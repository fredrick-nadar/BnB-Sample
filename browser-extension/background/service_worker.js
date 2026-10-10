/**
 * Handsfree Background Service Worker (Manifest V3)
 * Manages WebSocket connection to local Rust core (ws://127.0.0.1:8765)
 * Forwards snapshots and relays semantic commands to content scripts.
 */

const BRIDGE_WS_URL = "ws://127.0.0.1:8765";
let ws = null;
let reconnectTimer = null;
let bridgeToken = "auto";

// Immediately connect to local Rust bridge on startup (tokenless loopback)
connectBridge();

function connectBridge() {
  if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
    return;
  }

  try {
    ws = new WebSocket(BRIDGE_WS_URL);

    ws.onopen = () => {
      console.log("[Handsfree Bridge] Connected to Rust core. Authenticating...");
      chrome.storage.local.set({ bridge_connected: true });
      ws.send(
        JSON.stringify({
          id: crypto.randomUUID(),
          version: 1,
          timestamp: new Date().toISOString(),
          command: {
            action: "auth",
            payload: { token: bridgeToken }
          }
        })
      );
      // Immediately extract snapshot of active tab
      triggerActiveTabSnapshot();
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        handleServerCommand(msg);
      } catch (err) {
        console.warn("[Handsfree Bridge] Parse error:", err);
      }
    };

    ws.onclose = () => {
      console.log("[Handsfree Bridge] Disconnected from Rust core. Retrying in 3s...");
      chrome.storage.local.set({ bridge_connected: false });
      ws = null;
      if (!reconnectTimer) {
        reconnectTimer = setTimeout(() => {
          reconnectTimer = null;
          connectBridge();
        }, 3000);
      }
    };

    ws.onerror = (err) => {
      console.warn("[Handsfree Bridge] WebSocket error:", err);
      ws.close();
    };
  } catch (err) {
    console.warn("[Handsfree Bridge] Connection init error:", err);
  }
}

function handleServerCommand(envelope) {
  if (!envelope || !envelope.command) return;

  const { action, payload } = envelope.command;
  if (action === "semantic_action") {
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      if (tabs.length === 0) return;
      chrome.tabs.sendMessage(
        tabs[0].id,
        {
          action: "execute_semantic_action",
          target_id: payload.target_id,
          action_type: payload.action_type,
          value: payload.value
        },
        (res) => {
          if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(
              JSON.stringify({
                id: crypto.randomUUID(),
                version: 1,
                timestamp: new Date().toISOString(),
                command: {
                  action: "action_result",
                  payload: {
                    id: payload.id,
                    success: res ? res.success : false,
                    message: res ? res.message : "No response from page content script"
                  }
                }
              })
            );
          }
        }
      );
    });
  } else if (action === "ping") {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(
        JSON.stringify({
          id: crypto.randomUUID(),
          version: 1,
          timestamp: new Date().toISOString(),
          command: { action: "pong" }
        })
      );
    }
  }
}

function triggerActiveTabSnapshot() {
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs.length === 0) return;
    chrome.tabs.sendMessage(tabs[0].id, { action: "extract_snapshot" }, (res) => {
      if (chrome.runtime.lastError || !res) return;
      if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(
          JSON.stringify({
            id: crypto.randomUUID(),
            version: 1,
            timestamp: new Date().toISOString(),
            command: {
              action: "page_snapshot",
              payload: {
                generation_id: res.generation_id,
                timestamp: new Date().toISOString(),
                url: res.url,
                title: res.title,
                elements: res.elements
              }
            }
          })
        );
      }
    });
  });
}

// Re-snapshot on tab change or updates
chrome.tabs.onActivated.addListener(() => {
  setTimeout(triggerActiveTabSnapshot, 400);
});

chrome.runtime.onMessage.addListener((request) => {
  if (request.action === "update_token") {
    bridgeToken = request.token;
    chrome.storage.local.set({ handsfree_token: bridgeToken }, () => {
      if (ws) ws.close();
      connectBridge();
    });
  } else if (request.action === "page_ready") {
    triggerActiveTabSnapshot();
  }
});
