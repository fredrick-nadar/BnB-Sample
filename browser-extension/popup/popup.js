document.addEventListener("DOMContentLoaded", () => {
  const statusBadge = document.getElementById("status-badge");
  const tokenInput = document.getElementById("token-input");
  const saveBtn = document.getElementById("save-btn");

  // Load existing status and token
  chrome.storage.local.get(["handsfree_token", "bridge_connected"], (res) => {
    if (res.handsfree_token) {
      tokenInput.value = res.handsfree_token;
    }
    if (res.bridge_connected) {
      statusBadge.textContent = "Connected";
      statusBadge.className = "status-badge status-connected";
    } else {
      statusBadge.textContent = "Disconnected";
      statusBadge.className = "status-badge status-disconnected";
    }
  });

  saveBtn.addEventListener("click", () => {
    const token = tokenInput.value.trim();
    if (token) {
      chrome.runtime.sendMessage({ action: "update_token", token: token }, () => {
        statusBadge.textContent = "Connecting...";
        statusBadge.className = "status-badge status-disconnected";
      });
    }
  });
});
