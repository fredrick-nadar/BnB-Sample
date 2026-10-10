use handsfree_core::NormalizedEvent;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

pub struct WorkerIpcServer {
    event_sender: mpsc::UnboundedSender<NormalizedEvent>,
}

impl WorkerIpcServer {
    pub fn new(event_sender: mpsc::UnboundedSender<NormalizedEvent>) -> Self {
        Self { event_sender }
    }

    pub async fn start(&self, host: &str, port: u16) -> anyhow::Result<()> {
        if host != "127.0.0.1" && host != "localhost" {
            anyhow::bail!("Security violation: Worker IPC must bind only to loopback 127.0.0.1");
        }

        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
        let listener = TcpListener::bind(addr).await?;
        info!("Worker IPC server listening on {}", addr);

        let sender = self.event_sender.clone();
        tokio::spawn(async move {
            while let Ok((stream, client_addr)) = listener.accept().await {
                tracing::debug!("Worker connection initiated from {}", client_addr);
                let sender_clone = sender.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_worker_stream(stream, client_addr, sender_clone).await {
                        error!("Worker stream error from {}: {}", client_addr, e);
                    }
                });
            }
        });

        Ok(())
    }

    async fn handle_worker_stream(
        stream: TcpStream,
        client_addr: SocketAddr,
        sender: mpsc::UnboundedSender<NormalizedEvent>,
    ) -> anyhow::Result<()> {
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();
        let mut active = false;

        while let Some(line) = lines.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if !active {
                active = true;
                info!("Python worker actively streaming events from {}", client_addr);
            }

            match serde_json::from_str::<NormalizedEvent>(trimmed) {
                Ok(event) => {
                    let _ = sender.send(event);
                }
                Err(e) => {
                    warn!("Failed to parse normalized worker event: {}", e);
                }
            }
        }

        if active {
            info!("Worker disconnected from {}", client_addr);
        } else {
            tracing::debug!("Worker probe disconnected from {}", client_addr);
        }
        Ok(())
    }
}
