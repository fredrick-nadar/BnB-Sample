/**
 * Handsfree Semantic DOM Extractor
 * Extracts accessibility properties (role, accessible name, state)
 * Assigns short-lived element IDs and excludes sensitive password/payment fields.
 */

(function () {
  let elementCounter = 0;

  function getAccessibleName(el) {
    if (el.getAttribute("aria-label")) {
      return el.getAttribute("aria-label").trim();
    }
    if (el.getAttribute("aria-labelledby")) {
      const labelEl = document.getElementById(el.getAttribute("aria-labelledby"));
      if (labelEl) return labelEl.innerText.trim();
    }
    if (el.labels && el.labels.length > 0) {
      return el.labels[0].innerText.trim();
    }
    if (el.placeholder) {
      return el.placeholder.trim();
    }
    if (el.title) {
      return el.title.trim();
    }
    if (el.innerText && el.innerText.trim().length > 0 && el.innerText.trim().length < 60) {
      return el.innerText.trim();
    }
    return el.name || el.id || "";
  }

  function extractSemanticElements() {
    const interactiveSelectors = [
      "button",
      "a[href]",
      "input:not([type='hidden'])",
      "select",
      "textarea",
      "[role='button']",
      "[role='link']",
      "[role='checkbox']",
      "[role='menuitem']",
      "[tabindex]:not([tabindex='-1'])"
    ];

    const elements = document.querySelectorAll(interactiveSelectors.join(","));
    const snapshotList = [];

    elements.forEach((el) => {
      // Ignore hidden or zero-size elements
      const rect = el.getBoundingClientRect();
      if (rect.width === 0 || rect.height === 0 || window.getComputedStyle(el).visibility === "hidden") {
        return;
      }

      // Privacy: Never inspect password fields
      if (el.type === "password") {
        return;
      }

      // Assign stable, short-lived hf-id
      let hfId = el.getAttribute("data-hf-id");
      if (!hfId) {
        elementCounter++;
        hfId = `hf-${elementCounter}`;
        el.setAttribute("data-hf-id", hfId);
      }

      const role = el.getAttribute("role") || el.tagName.toLowerCase();
      const accessibleName = getAccessibleName(el);

      snapshotList.push({
        id: hfId,
        role: role,
        name: accessibleName,
        tag_name: el.tagName.toLowerCase(),
        placeholder: el.placeholder || null,
        is_clickable: typeof el.click === "function",
        is_input: el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable,
        is_disabled: el.disabled || el.getAttribute("aria-disabled") === "true",
      });
    });

    return snapshotList;
  }

  // Listen for extraction requests from background service worker
  chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.action === "extract_snapshot") {
      const elements = extractSemanticElements();
      sendResponse({
        url: window.location.href,
        title: document.title,
        elements: elements,
        generation_id: Date.now(),
      });
    }
  });

  // Automatically request snapshot update when DOM loads or changes
  window.addEventListener("load", () => {
    chrome.runtime.sendMessage({ action: "page_ready" });
  });
})();
