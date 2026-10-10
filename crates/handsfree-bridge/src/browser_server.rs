use crate::auth::BridgeAuthenticator;
use crate::protocol::{BridgeCommand, BridgeEnvelope, PageSnapshot};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct BrowserBridgeServer {
    authenticator: BridgeAuthenticator,
    latest_snapshot: Arc<RwLock<Option<PageSnapshot>>>,
    outgoing_tx: Arc<RwLock<Option<mpsc::UnboundedSender<Message>>>>,
}

impl BrowserBridgeServer {
    pub fn new(authenticator: BridgeAuthenticator) -> Self {
        Self {
            authenticator,
            latest_snapshot: Arc::new(RwLock::new(None)),
            outgoing_tx: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_latest_snapshot(&self) -> Option<PageSnapshot> {
        self.latest_snapshot.read().clone()
    }

    pub fn is_connected(&self) -> bool {
        self.outgoing_tx.read().is_some()
    }

    pub fn send_command(&self, command: BridgeCommand) -> anyhow::Result<()> {
        let envelope = BridgeEnvelope::new(command);
        let serialized = serde_json::to_string(&envelope)?;
        if let Some(tx) = &*self.outgoing_tx.read() {
            tx.send(Message::Text(serialized.into()))?;
            Ok(())
        } else {
            anyhow::bail!("Browser extension is not connected")
        }
    }

    pub async fn start(&self, host: &str, port: u16) -> anyhow::Result<()> {
        // Enforce loopback-only binding
        if host != "127.0.0.1" && host != "localhost" {
            anyhow::bail!(
                "Security violation: Browser bridge must bind only to loopback 127.0.0.1"
            );
        }

        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
        let listener = TcpListener::bind(addr).await?;
        info!("Browser bridge WebSocket server listening on {}", addr);

        let server_clone = self.clone();
        tokio::spawn(async move {
            while let Ok((stream, client_addr)) = listener.accept().await {
                tracing::debug!("Incoming bridge connection from {}", client_addr);
                let server = server_clone.clone();
                tokio::spawn(async move {
                    if let Err(e) = server.handle_connection(stream).await {
                        let err_str = e.to_string();
                        if err_str.contains("Handshake not finished")
                            || err_str.contains("Connection reset")
                            || err_str.contains("connection closed")
                        {
                            tracing::debug!(
                                "Browser bridge probe/disconnect from {}: {}",
                                client_addr,
                                err_str
                            );
                        } else {
                            error!("Bridge connection error: {}", e);
                        }
                    }
                });
            }
        });

        Ok(())
    }

    async fn handle_connection(&self, stream: TcpStream) -> anyhow::Result<()> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        *self.outgoing_tx.write() = Some(tx);

        // Spawn outgoing message forwarder
        let writer_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() {
                    break;
                }
            }
        });

        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    let parsed: Result<BridgeEnvelope, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(envelope) => match envelope.command {
                            BridgeCommand::Auth { .. } => {
                                info!("Browser extension connected and authenticated (tokenless loopback)");
                                let _ = self.send_command(BridgeCommand::Pong);
                            }
                            BridgeCommand::Snapshot(snapshot) => {
                                info!(
                                    "Received page snapshot for '{}' ({} elements)",
                                    snapshot.title,
                                    snapshot.elements.len()
                                );
                                *self.latest_snapshot.write() = Some(snapshot);
                            }
                            BridgeCommand::Ping => {
                                let _ = self.send_command(BridgeCommand::Pong);
                            }
                            BridgeCommand::Pong => {}
                            BridgeCommand::ActionResult {
                                id,
                                success,
                                message,
                            } => {
                                info!("Action {:?} result: {} ({:?})", id, success, message);
                            }
                            BridgeCommand::SemanticAction { .. } => {}
                        },
                        Err(e) => {
                            warn!("Malformed envelope received: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Browser extension disconnected");
                    break;
                }
                Ok(Message::Ping(_)) => {
                    let _ = self.send_command(BridgeCommand::Pong);
                }
                Err(e) => {
                    error!("WebSocket stream read error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        *self.outgoing_tx.write() = None;
        writer_task.abort();
        Ok(())
    }
}
