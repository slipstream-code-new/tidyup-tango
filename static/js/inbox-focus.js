/**
 * Inbox focus management and empty state transitions
 * -- progressive enhancement for HTMX.
 *
 * Focus management:
 * After an HTMX clarify or trash swap removes an inbox item from the DOM,
 * focus would otherwise be lost (sent to <body>). This script moves focus
 * to the logical next element:
 *   1. Next sibling inbox-item (if one exists)
 *   2. Previous sibling inbox-item (if removed was last in list)
 *   3. The "Capture" input (if the list is now empty)
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

    if (!target || !target.classList || !target.classList.contains("inbox-item")) {
      return;
    }

    var responseText = e.detail.xhr ? e.detail.xhr.responseText : "";

    // Empty response = clarify or trash operation (item removed)
    if (responseText.trim() === "") {
      pendingDeleteSwap = true;

      var nextItem = target.nextElementSibling;
      var prevItem = target.previousElementSibling;

      if (nextItem && nextItem.classList.contains("inbox-item")) {
        pendingFocusTarget = { type: "element", el: nextItem };
      } else if (prevItem && prevItem.classList.contains("inbox-item")) {
        pendingFocusTarget = { type: "element", el: prevItem };
      } else {
        pendingFocusTarget = { type: "element", el: document.getElementById("inbox-title") };
      }
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

  // After adding an inbox item, hide empty state and show the list
  document.body.addEventListener("htmx:afterRequest", function (e) {
    var elt = e.detail.elt;
    if (!elt || !elt.classList || !elt.classList.contains("inbox-capture")) {
      return;
    }
    if (!e.detail.successful) {
      return;
    }
    updateEmptyState();
  });

  function updateEmptyState() {
    var list = document.getElementById("inbox-list");
    var emptyState = document.getElementById("inbox-empty-state");
    if (!list || !emptyState) {
      return;
    }

    var hasItems = list.querySelector(".inbox-item") !== null;

    if (hasItems) {
      list.removeAttribute("hidden");
      emptyState.setAttribute("hidden", "");
    } else {
      list.setAttribute("hidden", "");
      emptyState.removeAttribute("hidden");
    }
  }
})();
