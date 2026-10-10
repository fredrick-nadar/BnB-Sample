/**
 * Handsfree Semantic Action Handler
 * Safely executes actions (click, fill) on elements identified by short-lived IDs.
 * Adds visual focus indicator for accessible confirmation.
 */

(function () {
  function highlightElement(el) {
    const originalOutline = el.style.outline;
    const originalBoxShadow = el.style.boxShadow;

    el.style.outline = "3px solid #0066cc";
    el.style.boxShadow = "0 0 10px rgba(0, 102, 204, 0.8)";

    setTimeout(() => {
      el.style.outline = originalOutline;
      el.style.boxShadow = originalBoxShadow;
    }, 1200);
  }

  chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.action === "execute_semantic_action") {
      const { target_id, action_type, value } = request;
      const el = document.querySelector(`[data-hf-id="${target_id}"]`);

      if (!el) {
        sendResponse({ success: false, message: `Target element ${target_id} not found or stale.` });
        return;
      }

      // Scroll element smoothly into view
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      highlightElement(el);

      try {
        if (action_type === "click") {
          el.focus();
          el.click();
          sendResponse({ success: true, message: `Clicked element ${target_id}` });
        } else if (action_type === "fill") {
          el.focus();
          el.value = value || "";
          el.dispatchEvent(new Event("input", { bubbles: true }));
          el.dispatchEvent(new Event("change", { bubbles: true }));
          sendResponse({ success: true, message: `Filled element ${target_id}` });
        } else {
          sendResponse({ success: false, message: `Unsupported action type ${action_type}` });
        }
      } catch (err) {
        sendResponse({ success: false, message: err.toString() });
      }
    }
  });
})();
