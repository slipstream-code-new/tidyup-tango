/**
 * Project detail focus management and empty state
 * -- progressive enhancement for HTMX.
 *
 * Focus management:
 * After adding an action via HTMX (beforeend swap into the list),
 * the empty state message and stalled notice are hidden.
 *
 * Completing actions triggers a full page reload (handled by
 * hx-on::after-request on the complete form), so focus is managed
 * naturally by the browser on reload. The server renders the correct
 * empty/stalled state.
 *
 * A11y: Focus management after DOM changes is required by WCAG 2.4.3
 * (Focus Order) and is part of our HTMX Focus Management Protocol.
 */
(function () {
  "use strict";

  document.body.addEventListener("htmx:afterRequest", function (e) {
    var elt = e.detail.elt;
    if (!elt || !elt.classList || !elt.classList.contains("project-detail__add-action")) {
      return;
    }
    if (!e.detail.successful) {
      return;
    }

    // Hide empty state and stalled notice after adding an action
    var emptyState = document.getElementById("project-empty-state");
    if (emptyState) {
      emptyState.setAttribute("hidden", "");
    }

    var stalledNotice = document.getElementById("project-stalled-notice");
    if (stalledNotice) {
      stalledNotice.setAttribute("hidden", "");
    }
  });
})();
