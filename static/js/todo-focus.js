/**
 * Todo list focus management and empty state -- progressive enhancement for HTMX.
 *
 * Focus management:
 * After an HTMX delete swap removes a todo item from the DOM, focus
 * would otherwise be lost (sent to <body>). This script moves focus
 * to the logical next element:
 *   1. Next sibling todo item (if one exists)
 *   2. Previous sibling todo item (if deleted was last in list)
 *   3. The "New todo" input (if the list is now empty)
 *
 * Empty state transitions:
 * When the last item is deleted, shows the "No todos yet." message
 * and hides the empty list. When the first item is added to an empty
 * list, hides the message and shows the list.
 *
 * A11y: Focus management after DOM removal is required by WCAG 2.4.3
 * (Focus Order) and is part of our HTMX Focus Management Protocol.
 */
(function () {
  "use strict";

  // Track the focus target before the swap removes the element
  var pendingFocusTarget = null;
  var pendingDeleteSwap = false;

  document.body.addEventListener("htmx:beforeSwap", function (e) {
    var target = e.detail.target;

    // Only handle delete swaps (empty response that removes an element)
    if (!target || !target.classList || !target.classList.contains("todo-item")) {
      return;
    }

    // If the response body is empty, this is a delete operation
    var responseText = e.detail.xhr ? e.detail.xhr.responseText : "";
    if (responseText.trim() !== "") {
      return;
    }

    pendingDeleteSwap = true;

    // Determine focus target before the element is removed
    var nextItem = target.nextElementSibling;
    var prevItem = target.previousElementSibling;

    if (nextItem && nextItem.classList.contains("todo-item")) {
      pendingFocusTarget = nextItem;
    } else if (prevItem && prevItem.classList.contains("todo-item")) {
      pendingFocusTarget = prevItem;
    } else {
      // List will be empty after this deletion -- focus the add input
      pendingFocusTarget = document.getElementById("title");
    }
  });

  document.body.addEventListener("htmx:afterSettle", function () {
    // Handle empty state visibility after delete
    if (pendingDeleteSwap) {
      pendingDeleteSwap = false;
      updateEmptyState();
    }

    if (!pendingFocusTarget) {
      return;
    }

    var el = pendingFocusTarget;
    pendingFocusTarget = null;

    // Make the target focusable if it isn't already
    if (!el.hasAttribute("tabindex") && el.tagName !== "INPUT" && el.tagName !== "BUTTON" && el.tagName !== "A") {
      el.setAttribute("tabindex", "-1");
    }

    el.focus();
  });

  // After adding a todo, hide empty state and show the list
  document.body.addEventListener("htmx:afterRequest", function (e) {
    var elt = e.detail.elt;
    // Only handle the add-todo form's successful POST
    if (!elt || !elt.classList || !elt.classList.contains("todo-add")) {
      return;
    }
    if (!e.detail.successful) {
      return;
    }
    updateEmptyState();
  });

  function updateEmptyState() {
    var todoList = document.querySelector(".todo-list");
    var emptyMsg = document.getElementById("todo-empty");
    if (!todoList || !emptyMsg) {
      return;
    }

    var hasItems = todoList.querySelector(".todo-item") !== null;

    if (hasItems) {
      todoList.removeAttribute("hidden");
      emptyMsg.setAttribute("hidden", "");
    } else {
      todoList.setAttribute("hidden", "");
      emptyMsg.removeAttribute("hidden");
    }
  }
})();
