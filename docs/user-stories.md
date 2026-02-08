# User Stories -- MVP

These user stories define the MVP scope. They are conversations, not contracts -- the
whole team owns them and anyone can propose changes with good reasoning.

Priority order follows the walking skeleton approach: get the thinnest slice working
end-to-end first, then layer on features.

---

## US-1: User Registration (Must Have)

**As a** new user,
**I want to** create an account with my email and password,
**So that** I have a personal, private space to manage my todos.

### Acceptance Criteria

- [ ] User can navigate to a registration page from the login page
- [ ] Registration form has fields: email, password, password confirmation
- [ ] **Open question for mob**: Consider replacing password confirmation with a password
  visibility toggle (show/hide). The toggle is better UX (see what you typed) but
  requires JS. Proposal: keep confirm field for no-JS baseline, add visibility toggle
  as progressive enhancement that hides the confirm field when JS is available.
- [ ] Every form input has a visible `<label>` element
- [ ] Submit button label is "Create account" (action words, not "Submit" or "Register")
- [ ] Successful registration creates a new user account and redirects to the todo list
  (user is automatically logged in)
- [ ] Email must be a valid email format
- [ ] Password must be at least 8 characters
- [ ] Password and password confirmation must match (unless mob decides on toggle approach)
- [ ] If email is already registered, show a secure, helpful error message that does NOT
  reveal whether the email exists (account enumeration prevention): "Something went
  wrong. If you already have an account, try signing in instead." with "signing in" as
  a link to the login page
- [ ] Validation errors are shown inline next to the relevant field
- [ ] Validation errors are associated with inputs via `aria-describedby`
- [ ] Password is hashed with Argon2 before storage (never stored in plaintext)
- [ ] Registration event is logged with structured tracing (no PII in logs)
- [ ] Form works without JavaScript (standard HTML form submission)
- [ ] HTMX enhances with inline validation and no full-page reload

### Edge Cases

- Email with unusual but valid format (e.g., `user+tag@example.com`)
- Very long email or password (enforce reasonable max lengths)
- SQL injection attempts in email/password fields (parameterized queries prevent this)
- Concurrent registration with the same email
- Empty form submission
- Whitespace-only inputs
- Password exactly at minimum length

### Success Metric

- A user can register and immediately start using their todo list in under 30 seconds.

---

## US-2: User Login (Must Have)

**As a** registered user,
**I want to** log in with my email and password,
**So that** I can access my personal todo list.

### Acceptance Criteria

- [ ] Login page is the default landing page for unauthenticated users
- [ ] Login form has fields: email, password
- [ ] Every form input has a visible `<label>` element
- [ ] Successful login redirects to the user's todo list
- [ ] Submit button label is "Sign in" (action words, not "Submit" or "Login")
- [ ] Invalid credentials show a friendly generic error: "That email or password didn't
  work. Try again." (do NOT reveal which field is wrong)
- [ ] Error message is announced to screen readers via `aria-live` region
- [ ] Login page has a link to the registration page
- [ ] "Forgot password?" link is present below the form. For MVP, it leads to a
  placeholder page explaining the feature is coming soon. (Password reset is out of
  MVP scope but users expect to see the link.)
- [ ] Session is created server-side on successful login
- [ ] Failed login attempts are logged with structured tracing (no PII)
- [ ] Form works without JavaScript
- [ ] HTMX enhances with no full-page reload

### Edge Cases

- User who has not registered tries to log in
- Correct email, wrong password
- Case sensitivity in email (emails should be case-insensitive)
- Brute force attempts (rate limiting is a v2 concern, but log failed attempts)
- Session fixation (generate new session ID on login)
- User tries to access todo list URL directly without being logged in (redirect to login)

### Success Metric

- A returning user can log in and see their todo list in under 10 seconds.

---

## US-3: User Logout (Must Have)

**As a** logged-in user,
**I want to** log out of my account,
**So that** my todo list is protected when I'm done using the app.

### Acceptance Criteria

- [ ] Logout button/link is visible on every authenticated page
- [ ] Clicking logout destroys the server-side session
- [ ] After logout, user is redirected to the login page
- [ ] After logout, the back button does not show authenticated content (no cache leaks)
- [ ] Logout works without JavaScript (standard link or form submission)
- [ ] Logout event is logged with structured tracing

### Edge Cases

- Double-clicking logout
- Logout when session has already expired
- Logout from multiple tabs (session destroyed on server; other tabs redirect on next request)

### Success Metric

- User can log out with a single click and is confident their session is ended.

---

## US-4: View Todo List (Must Have)

**As a** logged-in user,
**I want to** see my todo list when I log in,
**So that** I know what I need to do.

### Acceptance Criteria

- [ ] Todo list is the main screen after login
- [ ] Page has a clear heading (e.g., "My Todos") as the `<h1>`
- [ ] Todo items are displayed in a semantic list (`<ul>` or `<ol>`)
- [ ] Each item shows its title and completion status
- [ ] Completed items are visually distinct from pending items (not just color --
  a secondary indicator like strikethrough text)
- [ ] **Empty state**: When the user has no todos, show a friendly, helpful message
  (e.g., "No todos yet. Add one above to get started.") -- not a blank screen
- [ ] Page title (`<title>`) is descriptive (e.g., "My Todos -- Todo List")
- [ ] The page includes proper landmark regions (`<main>`, `<nav>`, `<header>`)
- [ ] Skip link to main content is the first focusable element
- [ ] Each user can only see their own todos (authorization check)
- [ ] Page works without JavaScript

### Edge Cases

- User with zero todos (empty state)
- User with one todo
- User with many todos (100+) -- still performant and usable
- Todo titles with special characters, HTML entities, or very long text
- Accessing another user's todo list URL (must return 403 or redirect)

### Success Metric

- A user can see their entire todo list within 1 second of logging in. The list is
  scannable -- the user's eye goes directly to what needs doing.

---

## US-5: Add a Todo Item (Must Have)

**As a** logged-in user,
**I want to** add a new todo item to my list,
**So that** I can track a task I need to do.

### Acceptance Criteria

- [ ] Input field for the todo title is prominently placed (above or at the top of the list)
- [ ] Input has a visible `<label>` (e.g., "New todo") and helpful placeholder text
  (e.g., "What do you need to do?")
- [ ] Submitting the form adds the item to the list immediately
- [ ] New items are added in the `Pending` state
- [ ] After adding, the input field is cleared and focus returns to it (ready for the
  next item)
- [ ] Empty or whitespace-only submissions are silently ignored (no error message --
  an accidental Enter press is not an intentional submission)
- [ ] Title has a maximum length of 300 characters (agreed with domain architect)
- [ ] Titles exceeding max length show a clear inline validation error
- [ ] Form works without JavaScript (full page reload, item appears in list)
- [ ] HTMX enhances: item is appended to the list without page reload
- [ ] New item creation is logged with structured tracing
- [ ] `aria-live` region announces "Todo added" (or similar) to screen readers after
  HTMX swap

### Edge Cases

- Adding an item with only whitespace (silently ignored)
- Pressing Enter with empty input (silently ignored)
- Adding an item with very long text (at or beyond max length -- show error)
- Adding an item with special characters, HTML, or emoji
- Rapid successive additions (no duplicate submissions)
- Adding when disconnected (graceful failure)

### Success Metric

- Adding a todo takes fewer than 5 seconds from thought to captured item. The flow
  feels as fast as jotting something on a sticky note.

---

## US-6: Complete a Todo Item (Must Have)

**As a** logged-in user,
**I want to** mark a todo item as complete,
**So that** I can see what I've accomplished and focus on what's left.

### Acceptance Criteria

- [ ] Each pending todo item has a mechanism to mark it complete (e.g., a checkbox or button)
- [ ] The mechanism is keyboard accessible
- [ ] Completing an item visually updates it immediately (e.g., strikethrough, dimmed,
  moved to a "completed" section)
- [ ] Completed items remain visible in the list (not hidden)
- [ ] A visual indicator beyond color alone marks completed items (e.g., strikethrough + check)
- [ ] Completion can be undone (mark as pending again) -- toggling state
- [ ] State change works without JavaScript (form submission)
- [ ] HTMX enhances: in-place update without page reload
- [ ] State change is announced to screen readers via `aria-live` region
- [ ] Completion event is logged with structured tracing
- [ ] Focus remains on or near the toggled item after the update

### Edge Cases

- Completing an already-completed item (toggle back to pending)
- Completing an item that was deleted by another tab (graceful error)
- Rapid toggling (no race conditions)

### Success Metric

- Completing a todo is a single action (one click or keypress). The visual feedback
  is immediate and satisfying.

---

## US-7: Delete a Todo Item (Must Have)

**As a** logged-in user,
**I want to** delete a todo item I no longer need,
**So that** my list stays clean and focused.

### Acceptance Criteria

- [ ] Each todo item has a delete action (button, not a link -- this is a destructive action)
- [ ] Delete button has an accessible label (e.g., "Delete: Buy groceries")
- [ ] Deletion removes the item from the list immediately
- [ ] Deletion is permanent (no undo in MVP -- but confirm before deleting)
- [ ] Confirmation mechanism before deletion (e.g., "Are you sure?" or visually distinct
  confirm/cancel) -- keep it lightweight, not a modal dialog
- [ ] After deletion, focus moves to the next item in the list (or to the "add" input
  if the list is now empty)
- [ ] Deletion works without JavaScript (form submission, page reload)
- [ ] HTMX enhances: item is removed from DOM without page reload
- [ ] Screen reader announcement: "Todo deleted" via `aria-live` region
- [ ] Deletion event is logged with structured tracing
- [ ] Deleting someone else's todo is not possible (authorization)

### Edge Cases

- Deleting the last item in the list (transitions to empty state)
- Deleting an item that was already deleted (e.g., in another tab)
- Confirmation dismissed (item should not be deleted)

### Success Metric

- Deleting a todo is quick but safe. Users never accidentally lose a todo item.

---

## US-8: Edit a Todo Item Title (Should Have)

**As a** logged-in user,
**I want to** edit the title of an existing todo item,
**So that** I can fix typos or update the description without deleting and re-creating.

### Acceptance Criteria

- [ ] Each todo item has a visible edit button (pencil icon) as the primary affordance
  (click-on-title as an optional progressive enhancement for power users, but the icon
  is what new users will discover)
- [ ] Edit mode replaces the title with an input field pre-filled with the current title
- [ ] Input has a visible `<label>` (may be visually hidden if the context is clear,
  but must be present for screen readers)
- [ ] Save and cancel actions are available (e.g., Enter to save, Escape to cancel, or
  explicit Save/Cancel buttons)
- [ ] Same validation as adding: non-empty, max length, no whitespace-only
- [ ] Validation errors shown inline
- [ ] After saving, the item returns to display mode with the updated title
- [ ] After canceling, the original title is restored
- [ ] Edit works without JavaScript (navigate to an edit page/form, save redirects back)
- [ ] HTMX enhances: inline editing without page navigation
- [ ] Focus management: on entering edit mode, focus moves to the input; on saving/canceling,
  focus returns to the item
- [ ] Screen reader: state change announced via `aria-live` region
- [ ] Edit event is logged with structured tracing

### Edge Cases

- Editing to an empty title (validation error)
- Editing while another tab has deleted the item
- Editing a completed item (should work the same way)
- Very long replacement text
- Pressing Escape after making changes (discard changes)
- Saving with no changes (no-op, no error)

### Success Metric

- Fixing a typo in a todo item takes under 5 seconds. The inline editing feels natural
  and doesn't interrupt the user's flow.

---

## Priority and Implementation Order

Following the walking skeleton principle, the implementation order is:

1. **Project scaffolding** (Task #1) -- Cargo.toml, project structure, docker-compose, config
2. **Walking skeleton** (Task #2) -- Health check endpoint with integration test
3. **Design tokens** (Task #3) -- CSS custom properties foundation
4. **US-1: Registration** (Task #4) -- First real feature, proves the full stack works
5. **US-2: Login** (Task #5) -- Requires registration; completes the auth flow
6. **US-3: Logout** (Task #5) -- Bundled with login (same session management)
7. **US-4: View todo list + US-5: Add todo** (Task #6) -- Core loop: see list, add items
8. **US-6: Complete todo** (Task #7) -- State change
9. **US-7: Delete todo** (Task #8) -- Removal
10. **US-8: Edit todo title** (Task #9) -- Polish (should have)

This order delivers value incrementally. After step 7, we have a usable product. Steps
8-10 are enhancements on the core.

---

## Product Success Criteria (MVP)

The MVP is successful when:

1. A new user can sign up, log in, manage todos (add, complete, delete), and log out
2. The experience loads fast (< 1 second for any page)
3. The app is fully accessible (WCAG 2.2 AA)
4. The app works without JavaScript (progressive enhancement)
5. The architecture cleanly supports future iteration
6. The codebase has comprehensive test coverage via TDD

We will know this is working when a first-time user can accomplish the complete
add-complete-delete flow without hesitation or confusion.
