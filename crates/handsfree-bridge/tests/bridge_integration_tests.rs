use futures_util::{SinkExt, StreamExt};
use handsfree_bridge::{
    auth::BridgeAuthenticator,
    browser_server::BrowserBridgeServer,
    protocol::{BridgeCommand, BridgeEnvelope, PageSnapshot, SemanticElement},
};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_browser_bridge_auth_and_snapshot() {
    let authenticator = BridgeAuthenticator::with_token("test-secret-token-123");
    let server = BrowserBridgeServer::new(authenticator);

    // Start server on ephemeral port 18765
    server
        .start("127.0.0.1", 18765)
        .await
        .expect("server starts");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connect client
    let url = "ws://127.0.0.1:18765";
    let (ws_stream, _) = connect_async(url).await.expect("client connects");
    let (mut write, mut read) = ws_stream.split();

    // 1. Send auth command
    let auth_cmd = BridgeEnvelope::new(BridgeCommand::Auth {
        token: "test-secret-token-123".to_string(),
    });
    write
        .send(Message::Text(
            serde_json::to_string(&auth_cmd).unwrap().into(),
        ))
        .await
        .expect("auth sent");

    // Expect pong back
    if let Some(Ok(Message::Text(resp_text))) = read.next().await {
        let envelope: BridgeEnvelope = serde_json::from_str(&resp_text).expect("valid json");
        assert_eq!(envelope.command, BridgeCommand::Pong);
    } else {
        panic!("Did not receive auth response");
    }

    // 2. Send page snapshot
    let snapshot = PageSnapshot {
        generation_id: 1001,
        timestamp: chrono::Utc::now(),
        url: "http://127.0.0.1/test_page.html".to_string(),
        title: "Test Page".to_string(),
        elements: vec![SemanticElement {
            id: "hf-1".to_string(),
            role: "button".to_string(),
            name: "Search Catalog".to_string(),
            tag_name: "button".to_string(),
            placeholder: None,
            is_clickable: true,
            is_input: false,
            is_disabled: false,
        }],
    };

    let snap_cmd = BridgeEnvelope::new(BridgeCommand::Snapshot(snapshot.clone()));
    write
        .send(Message::Text(
            serde_json::to_string(&snap_cmd).unwrap().into(),
        ))
        .await
        .expect("snapshot sent");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify snapshot stored in server
    let stored = server.get_latest_snapshot().expect("snapshot exists");
    assert_eq!(stored.title, "Test Page");
    assert_eq!(stored.elements.len(), 1);
    assert_eq!(stored.elements[0].name, "Search Catalog");
}
