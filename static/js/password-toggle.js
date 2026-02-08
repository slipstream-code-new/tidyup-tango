/**
 * Password visibility toggle -- progressive enhancement.
 *
 * No-JS baseline: password + confirm password fields work normally.
 * With JS: a toggle button replaces the confirm field. When password
 * is visible, the user can see what they typed, so confirmation is
 * unnecessary. When hidden again, confirm field returns.
 *
 * A11y:
 * - Toggle is a <button type="button"> (not a checkbox or link)
 * - aria-pressed communicates toggle state to screen readers
 * - aria-label describes the action ("Show password" / "Hide password")
 * - Focus remains on the toggle after activation
 */
(function () {
  "use strict";

  var passwordInput = document.getElementById("password");
  var confirmGroup = document.getElementById("confirm-group");

  // Only enhance if both elements exist (registration page)
  if (!passwordInput || !confirmGroup) {
    return;
  }

  // Store the confirm group's position in the DOM
  var confirmGroupParent = confirmGroup.parentNode;
  var confirmGroupNextSibling = confirmGroup.nextSibling;

  // Create a hidden input to mirror the password value when confirm field is removed.
  // Without this, the form won't send password_confirmation and the server rejects it.
  var hiddenConfirm = document.createElement("input");
  hiddenConfirm.type = "hidden";
  hiddenConfirm.name = "password_confirmation";

  // Create the toggle button
  var toggle = document.createElement("button");
  toggle.type = "button";
  toggle.className = "password-toggle";
  toggle.setAttribute("aria-label", "Show password");
  toggle.setAttribute("aria-pressed", "false");
  toggle.textContent = "Show password";

  // Insert toggle after the password input's parent group
  var passwordGroup = passwordInput.closest(".auth-form__group");
  if (!passwordGroup) {
    return;
  }
  passwordGroup.appendChild(toggle);

  // Remove the confirm group and add hidden mirror (JS is working, toggle replaces it)
  confirmGroup.remove();
  hiddenConfirm.value = passwordInput.value;
  passwordGroup.appendChild(hiddenConfirm);

  // Keep the hidden input in sync as the user types
  passwordInput.addEventListener("input", function () {
    if (hiddenConfirm.parentNode) {
      hiddenConfirm.value = passwordInput.value;
    }
  });

  toggle.addEventListener("click", function () {
    var isPressed = toggle.getAttribute("aria-pressed") === "true";

    if (isPressed) {
      // Hide password: restore confirm field, remove hidden mirror
      passwordInput.type = "password";
      toggle.setAttribute("aria-pressed", "false");
      toggle.setAttribute("aria-label", "Show password");
      toggle.textContent = "Show password";

      if (hiddenConfirm.parentNode) {
        hiddenConfirm.remove();
      }

      // Restore the confirm group
      if (confirmGroupNextSibling) {
        confirmGroupParent.insertBefore(confirmGroup, confirmGroupNextSibling);
      } else {
        confirmGroupParent.appendChild(confirmGroup);
      }
    } else {
      // Show password: remove confirm field, add hidden mirror
      passwordInput.type = "text";
      toggle.setAttribute("aria-pressed", "true");
      toggle.setAttribute("aria-label", "Hide password");
      toggle.textContent = "Hide password";

      // Remove the confirm group
      confirmGroup.remove();

      // Add hidden input mirroring the password value
      hiddenConfirm.value = passwordInput.value;
      passwordGroup.appendChild(hiddenConfirm);
    }
  });
})();
