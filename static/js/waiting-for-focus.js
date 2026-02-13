/**
 * Waiting For focus management and empty state
 * -- progressive enhancement for HTMX.
 *
 * Focus management:
 * After an HTMX complete or delete swap removes a waiting-for item from the DOM,
 * focus would otherwise be lost (sent to <body>). This script moves focus
 * to the logical next element:
 *   1. Next sibling waiting-for-item (if one exists)
 *   2. Previous sibling waiting-for-item (if deleted was last in list)
 *   3. The "What are you waiting for?" input (if the list is now empty)
 *
 * Inline edit focus:
 * Entering edit mode focuses the edit input. Saving or canceling
 * focuses the edit link on the restored item.
 *
 * Empty state transitions:
 * When the last item is removed, shows the empty state message
 * and hides the empty list. When items are added, hides the
 * message and shows the list.
 *
 * A11y: Focus management after DOM removal is required by WCAG 2.4.3
 * (Focus Order) and is part of our HTMX Focus Management Protocol.
 */
(function () {
  "use strict";

  var pendingFocusTarget = null;
  var pendingDeleteSwap = false;

  document.body.addEventListener("htmx:beforeSwap", function (e) {
    var target = e.detail.target;

    if (!target || !target.classList || !target.classList.contains("waiting-for-item")) {
      return;
    }

    var responseText = e.detail.xhr ? e.detail.xhr.responseText : "";

    // Empty response = complete or delete operation (item removed)
    if (responseText.trim() === "") {
      pendingDeleteSwap = true;

      var nextItem = target.nextElementSibling;
      var prevItem = target.previousElementSibling;

      if (nextItem && nextItem.classList.contains("waiting-for-item")) {
        pendingFocusTarget = { type: "element", el: nextItem };
      } else if (prevItem && prevItem.classList.contains("waiting-for-item")) {
        pendingFocusTarget = { type: "element", el: prevItem };
      } else {
        pendingFocusTarget = { type: "element", el: document.getElementById("waiting-for-title") };
      }
      return;
    }

    // Check if the response contains the edit form (entering edit mode)
    if (responseText.indexOf("waiting-for-item__edit-form") !== -1) {
      pendingFocusTarget = { type: "selector", selector: "#" + target.id + " .waiting-for-item__edit-input" };
      return;
    }

    // Check if the response is a normal item (exiting edit mode)
    if (responseText.indexOf("waiting-for-item__edit") !== -1 && responseText.indexOf("waiting-for-item__edit-form") === -1) {
      pendingFocusTarget = { type: "selector", selector: "#" + target.id + " .waiting-for-item__edit" };
      return;
    }
  });

  document.body.addEventListener("htmx:afterSettle", function () {
    if (pendingDeleteSwap) {
      pendingDeleteSwap = false;
      updateEmptyState();
    }

    if (!pendingFocusTarget) {
      return;
    }

    var pending = pendingFocusTarget;
    pendingFocusTarget = null;

    var el;
    if (pending.type === "element") {
      el = pending.el;
    } else if (pending.type === "selector") {
      el = document.querySelector(pending.selector);
    }

    if (!el) {
      return;
    }

    // Make the target focusable if it isn't already
    if (!el.hasAttribute("tabindex") && el.tagName !== "INPUT" && el.tagName !== "BUTTON" && el.tagName !== "A") {
      el.setAttribute("tabindex", "-1");
    }

    el.focus();
  });

  // After adding an item, hide empty state and show the list
  document.body.addEventListener("htmx:afterRequest", function (e) {
    var elt = e.detail.elt;
    if (!elt || !elt.classList || !elt.classList.contains("waiting-for__add")) {
      return;
    }
    if (!e.detail.successful) {
      return;
    }
    updateEmptyState();
  });

  function updateEmptyState() {
    var list = document.getElementById("waiting-for-list");
    var emptyState = document.getElementById("waiting-for-empty-state");
    if (!list || !emptyState) {
      return;
    }

    var hasItems = list.querySelector(".waiting-for-item") !== null;

    if (hasItems) {
      list.removeAttribute("hidden");
      emptyState.setAttribute("hidden", "");
    } else {
      list.setAttribute("hidden", "");
      emptyState.removeAttribute("hidden");
    }
  }
})();
