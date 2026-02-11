# User Stories

## Part 1: MVP (Completed)

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
- [ ] **Password progressive enhancement** (consensus: Marty, Steve Krug, Heydon):
  - **No-JS baseline**: Email + password + password confirmation. Confirm field catches
    typos. Both password fields use `autocomplete="new-password"`.
  - **JS enhancement**: Password visibility toggle replaces confirm field. Toggle is a
    `<button type="button">` with `aria-label` ("Show password" / "Hide password") and
    `aria-pressed`. When toggle is active, confirm field is removed from DOM (not just
    hidden). Input type changes between `password` and `text`.
  - This is the agreed approach -- no further discussion needed.
- [ ] Every form input has a visible `<label>` element
- [ ] Submit button label is "Create account" (action words, not "Submit" or "Register")
- [ ] Successful registration creates a new user account and redirects to the todo list
  (user is automatically logged in)
- [ ] Email must be a valid email format
- [ ] Password must be at least 8 characters
- [ ] Password and password confirmation must match (no-JS baseline); or password
  verified via visibility toggle (JS enhancement)
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
- [ ] Todo items are displayed in a semantic list (`<ul>` or `<ol>`) so screen readers
  automatically announce the item count (e.g., "list, 5 items")
- [ ] Each item shows its title and completion status
- [ ] Completed items are visually distinct from pending items (not just color --
  a secondary indicator like strikethrough text)
- [ ] **Empty state**: When the user has no todos, show a brief, friendly message
  (e.g., "No todos yet.") -- not a blank screen. The add-todo input above makes the
  next action obvious; don't over-explain.
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

## MVP Implementation Order (Completed)

All MVP features have been implemented and shipped:

1. Project scaffolding (Task #1)
2. Walking skeleton (Task #2)
3. Design tokens (Task #3)
4. US-1: Registration (Task #4-5)
5. US-2: Login + US-3: Logout (Task #5-6)
6. US-4: View todo list + US-5: Add todo (Task #7)
7. US-6: Complete todo (Task #8)
8. US-7: Delete todo (Task #9)
9. US-8: Edit todo title (Task #20)

---

## Product Success Criteria (MVP -- Achieved)

The MVP was successful:
- 97 Rust integration tests + 6 Playwright e2e tests all passing
- WCAG 2.2 AA accessible (axe-core verified)
- Works without JavaScript (progressive enhancement with HTMX)
- Full TDD coverage
- CI green

---

---

## Part 2: V1 GTD (Getting Things Done)

The product is evolving from a simple todo list into a full GTD system. These stories
define the V1 GTD scope. See `docs/gtd-product-discovery.md` for the complete product
discovery document.

### US-GTD-1: Capture to Inbox

**As a** GTD practitioner,
**I want to** quickly capture anything on my mind into my inbox,
**So that** I can get it out of my head and process it later.

#### Acceptance Criteria

- [ ] Inbox capture input is always accessible (prominent on the main page, and available
  via a persistent capture input on every authenticated page)
- [ ] Capture is fast: type and press Enter, item appears in inbox
- [ ] No categorization required at capture time (that happens during clarify)
- [ ] Inbox items show in a list ordered by capture time (oldest first -- process in
  FIFO order, like a physical inbox)
- [ ] Inbox count is visible in the navigation so users know how many items need processing
- [ ] Input has a visible `<label>` and helpful placeholder text
- [ ] After capturing, the input field is cleared and focus returns to it
- [ ] Empty or whitespace-only submissions are silently ignored
- [ ] Title has a maximum length of 300 characters
- [ ] Works without JavaScript (full page reload); HTMX enhances
- [ ] `aria-live` region announces "Captured to inbox" after HTMX swap
- [ ] Capture event is logged with structured tracing

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Capture input has a visible `<label>` associated via `for`/`id`
- [ ] Focus management: after capture, focus returns to the input
- [ ] Skip link bypasses header capture input to reach main content
- [ ] Screen reader announces inbox count change after capture

#### Empty State
- When the inbox is empty, show: "Inbox zero -- nothing to process. Capture something
  new or start your weekly review." with links to capture input and weekly review.

#### Edge Cases
- Adding with only whitespace (silently ignored)
- Rapid successive captures
- Very long text (at or beyond max length)
- Special characters, HTML, emoji

#### Success Metric
Capturing a thought takes under 3 seconds from thought to inbox.

---

### US-GTD-2: Clarify Inbox Items

**As a** GTD practitioner,
**I want to** process each inbox item through the GTD clarify workflow,
**So that** I can decide what each item means and where it belongs.

#### Acceptance Criteria

- [ ] User can process inbox items one at a time (process view for a single item)
- [ ] For each item, the user sees the original capture text and can choose:
  - **Next Action**: Specify action text (may keep or edit the captured text), select
    a context, optionally link to an existing or new project
  - **Project**: Create a new project with title and a first next action
  - **Waiting For**: Move to waiting for list with who/what information
  - **Someday/Maybe**: Park as a future idea
  - **Trash**: Delete permanently
- [ ] After clarifying, item leaves the inbox and appears in its destination list.
  Server redirects to the next unprocessed inbox item (`/inbox/{next-id}/clarify`)
  so the user stays in processing mode. If no items remain, redirect to an "Inbox
  clear!" success page with a link back to the dashboard.
- [ ] Processing a single item to Next Action requires a context selection
- [ ] User can skip an item and come back to it (returns to inbox list)
- [ ] "Back to inbox" link is always visible on the clarify page for users who want
  to exit processing mode
- [ ] Clarify page shows processing progress ("3 of 12 items processed") to create
  momentum toward inbox zero
- [ ] All form inputs have visible labels
- [ ] Clarify interaction pattern: a single form on `/inbox/{id}/clarify` with radio
  buttons for destination choice (Next Action, Project, Waiting For, Someday/Maybe,
  Trash). Selecting a destination reveals the relevant fields (context select for Next
  Action, who/what input for Waiting For, etc.) via progressive disclosure. One submit
  button processes the item. No multi-step wizard, no modals. Standard HTML form that
  works without JavaScript; HTMX enhances with inline field reveal.
- [ ] Works without JavaScript; HTMX enhances
- [ ] Screen reader announces the result of clarification via `aria-live` region

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Radio buttons for destination choice are in a `<fieldset>` with `<legend>`
- [ ] Conditionally revealed fields use `aria-describedby` to associate with their
  radio button context
- [ ] Focus moves to the next inbox item (or success message) after processing
- [ ] Form validation errors are associated with inputs via `aria-describedby`

#### Edge Cases
- Last inbox item processed (show success message, link back to dashboard)
- Clarifying to a context that was just created
- Editing the title during clarification
- Canceling mid-clarification (item stays in inbox)

#### UX Note
The clarify flow must feel fast -- like sorting mail, not filling out a form. The user
should be able to process an item in a few clicks. Radio buttons for the destination,
minimal fields revealed per choice, one submit. No friction.

#### Success Metric
Processing a single inbox item takes under 15 seconds.

#### Sub-Stories (for implementation slicing)
- **US-GTD-2a**: Clarify as Next Action (inbox item -> next action with context)
- **US-GTD-2b**: Clarify as Trash (inbox item -> deleted)
- **US-GTD-2c**: Clarify as Someday/Maybe (inbox item -> someday/maybe list)
- **US-GTD-2d**: Clarify as Project (inbox item -> new project + first next action)
- **US-GTD-2e**: Clarify as Waiting For (inbox item -> waiting for list with who/what)

---

### US-GTD-3: View and Work from Next Actions

**As a** GTD practitioner,
**I want to** see my next actions filtered by context,
**So that** I can quickly find what I can do right now based on where I am.

#### Acceptance Criteria

- [ ] Next Actions page shows all active (not completed) next actions
- [ ] Actions can be filtered by context (e.g., show only @computer actions)
- [ ] Default view shows all contexts with context headings as groupings
- [ ] Each action shows its title, context, and associated project name (if any)
- [ ] User can mark an action as complete (removes from active list)
- [ ] User can delete an action
- [ ] User can edit an action's title
- [ ] Context filter is a set of links/buttons, not a dropdown (scannable)
- [ ] Page has clear heading hierarchy
- [ ] Works without JavaScript; HTMX enhances for in-place updates
- [ ] Screen reader announces state changes via `aria-live` region

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Context filter links are within a `<nav>` landmark with `aria-label="Filter by context"`
- [ ] Active context filter is indicated with `aria-current="true"`
- [ ] Action list uses semantic `<ul>` so screen readers announce item count
- [ ] Complete/delete/edit actions have accessible labels including the action title
- [ ] Focus management after completion: focus moves to next item in list
- [ ] Result count announced via `aria-live` region after filtering

#### Empty State
- No actions at all: "No next actions yet. Process your inbox to find things to do."
  with link to inbox.
- No actions in filtered context: "Nothing to do @context right now." (no alarm, just
  informational).

#### Edge Cases
- No actions in a given context (show empty state per context)
- No actions at all (show helpful empty state)
- Filtering to a context with zero actions
- Action linked to a completed/dropped project

#### Success Metric
User can find and start working on a relevant action within 10 seconds.

---

### US-GTD-4: Manage Projects

**As a** GTD practitioner,
**I want to** see all my active projects and their next actions,
**So that** I can ensure every project is moving forward.

#### Acceptance Criteria

- [ ] Projects list shows all active projects
- [ ] Each project shows its title and the count of active next actions
- [ ] Stalled projects (zero next actions) are visually flagged
- [ ] User can view a project detail page showing all linked next actions (active and
  completed) and waiting-for items
- [ ] User can add a project directly (without going through inbox)
- [ ] User can add a next action directly to a project (with context selection)
- [ ] User can complete a project (marks as finished)
- [ ] User can drop a project (abandoned, different from completed)
- [ ] User can edit a project's title
- [ ] User can delete a project (with confirmation; warns about linked actions)
- [ ] Works without JavaScript; HTMX enhances

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Project list uses semantic `<ul>` so screen readers announce item count
- [ ] Stalled project flag is communicated via text (not color alone) -- e.g.,
  "No next actions" label visible alongside the project title
- [ ] Project detail page has clear heading hierarchy (`<h1>` project title,
  `<h2>` sections for next actions, completed actions, waiting for)
- [ ] Add/complete/delete/edit actions have accessible labels including the project title

#### Empty State
- No projects: "No projects yet. Projects are outcomes that need more than one step.
  You can create one here or during inbox processing." with link to add project form.

#### Edge Cases
- Project with no next actions (stalled -- flagged during weekly review)
- Deleting a project with active next actions (what happens to them?)
- Completing a project with remaining active next actions
- Very long project title

#### Success Metric
During weekly review, user can verify all projects have next actions in under 2 minutes.

---

### US-GTD-5: Waiting For List

**As a** GTD practitioner,
**I want to** track items I'm waiting on from others,
**So that** I can follow up at the right time and nothing falls through the cracks.

#### Acceptance Criteria

- [ ] Waiting For list shows all items the user is waiting on
- [ ] Each item shows: title, who/what it's waiting on, date added
- [ ] Items are sorted by date added (oldest first -- longest-waiting items at top)
- [ ] User can add items directly to Waiting For (with title and waiting-on text)
- [ ] User can resolve a Waiting For item (removes from list)
- [ ] User can convert a Waiting For item to a Next Action (with context selection)
- [ ] User can edit a Waiting For item's title and waiting-on text
- [ ] User can delete a Waiting For item
- [ ] User can optionally link a Waiting For item to a project
- [ ] Works without JavaScript; HTMX enhances

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Waiting For list uses semantic `<ul>` so screen readers announce item count
- [ ] Each item's "waiting on" metadata is associated with the item (not just visual proximity)
- [ ] Resolve/convert/delete actions have accessible labels including the item title
- [ ] Focus management after resolving: focus moves to next item or empty state

#### Empty State
- "Nothing pending from others. When you delegate something or need to follow up,
  add it here." with link to add form.

#### Edge Cases
- Empty Waiting For list (show helpful message)
- Very long waiting-on description
- Resolving the last Waiting For item linked to a project

#### Success Metric
User can see everything they are waiting on in one glance.

---

### US-GTD-6: Someday/Maybe List

**As a** GTD practitioner,
**I want to** park ideas and future possibilities,
**So that** they don't clutter my action lists but aren't forgotten either.

#### Acceptance Criteria

- [ ] Someday/Maybe list shows all parked ideas
- [ ] User can add items directly to Someday/Maybe
- [ ] User can activate an item (moves to inbox for re-clarification)
- [ ] User can delete items that are no longer interesting
- [ ] User can edit an item's title
- [ ] List is reviewed during the weekly review (Get Creative phase)
- [ ] Works without JavaScript; HTMX enhances

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Someday/Maybe list uses semantic `<ul>` so screen readers announce item count
- [ ] Activate/delete/edit actions have accessible labels including the item title
- [ ] Focus management after activating: focus moves to next item or empty state

#### Empty State
- "No ideas parked yet. When something's interesting but not actionable right now,
  put it here. You'll review this list weekly."

#### Edge Cases
- Empty Someday/Maybe list
- Activating an item (goes to inbox, not directly to Next Actions)

#### Success Metric
Ideas have a home that is not the user's head or their active lists.

---

### US-GTD-7: Weekly Review

**As a** GTD practitioner,
**I want to** be guided through a weekly review process,
**So that** I can keep my system current and trustworthy.

#### Acceptance Criteria

- [ ] Weekly Review is accessible from the main navigation
- [ ] Review follows three phases:
  1. **Get Clear**: Shows inbox count, prompts user to process inbox to zero. Links to
     inbox processing. Phase is complete when inbox is empty.
  2. **Get Current**: Shows each active project; user confirms next action exists or adds
     one. Shows Waiting For items; user confirms still waiting or resolves. Shows Next
     Actions; user confirms still relevant or removes/defers.
  3. **Get Creative**: Shows Someday/Maybe list for review. Prompts for new captures.
- [ ] Each phase has a clear completion state
- [ ] User can navigate between phases freely (not forced linear)
- [ ] Review progress is visible (which phases are done this session)
- [ ] Phase completion is announced to screen readers
- [ ] Works without JavaScript; HTMX enhances

#### Accessibility
- [ ] All interactive elements are fully keyboard operable
- [ ] Review phases use clear heading hierarchy (`<h1>` Weekly Review, `<h2>` per phase)
- [ ] Phase completion state is communicated via text, not color alone
- [ ] Links to relevant views (inbox, projects, etc.) have descriptive text
- [ ] Progress is perceivable by screen readers (e.g., "Phase 1 of 3: Get Clear -- complete")

#### Edge Cases
- Starting a review with empty inbox (phase 1 auto-completes)
- No active projects (phase 2 project section is empty)
- Empty Someday/Maybe list (phase 3 shows capture prompt only)

#### Success Metric
A weekly review can be completed in under 30 minutes.

---

### US-GTD-8: Navigation and Dashboard

**As a** GTD practitioner,
**I want to** navigate between all GTD lists and see a summary of my system's state,
**So that** I can access any part of my GTD system quickly and know what needs attention.

#### Acceptance Criteria

- [ ] Main navigation includes: Inbox (with count), Next Actions, Projects, Waiting For,
  Someday/Maybe, Weekly Review
- [ ] Dashboard/home page shows a summary:
  - Inbox count (items needing processing)
  - Stalled projects count (projects without next actions)
  - Total active next actions count
  - Waiting For count
- [ ] Navigation indicates the current page (aria-current="page")
- [ ] Navigation is accessible: keyboard operable, screen reader landmarks
- [ ] Logout remains accessible from every page
- [ ] Quick capture input is available on every authenticated page (in the header,
  above main content). Note: on the Inbox page, this header input and the inbox page's
  own capture input are the same form posting to the same endpoint -- avoid showing
  two capture inputs on the inbox page.
- [ ] Works without JavaScript

#### Accessibility
- [ ] Navigation uses `<nav>` landmark with `aria-label="GTD lists"`
- [ ] Current page indicated with `aria-current="page"` on the active nav link
- [ ] All interactive elements are fully keyboard operable
- [ ] Inbox count badge is announced by screen readers (not just visual)
- [ ] Skip link jumps past header (including capture input) to main content
- [ ] Dashboard summary counts use semantic markup (e.g., `<dl>` for label/value pairs)
- [ ] Clear focus indicators on all navigation links

#### Edge Cases
- All counts are zero (fresh user -- show getting started guidance)
- Very high counts (display doesn't break)

#### Success Metric
Any list is reachable in one click from any page.

---

## V1 GTD Priority and Implementation Order

Following the walking skeleton principle. Clarify is split into sub-stories (US-GTD-2a
through 2e) and interleaved with the lists it routes to, so each clarify path is built
immediately after its destination list exists:

1. **Navigation + Dashboard** (US-GTD-8) -- Multi-page structure, header nav, route skeleton
2. **Inbox Capture** (US-GTD-1) -- New entry point
3. **Contexts** -- Domain type + DB + default contexts for new users
4. **Next Actions with Contexts** (US-GTD-3) -- Core working list
5. **Clarify as Next Action + Trash** (US-GTD-2a, 2b) -- First clarify paths (inbox -> next action, inbox -> trash)
6. **Projects** (US-GTD-4) -- Multi-step outcome tracking
7. **Clarify as Project** (US-GTD-2d) -- Inbox -> new project + first next action
8. **Waiting For** (US-GTD-5) -- Delegation tracking
9. **Clarify as Waiting For** (US-GTD-2e) -- Inbox -> waiting for
10. **Someday/Maybe** (US-GTD-6) -- Idea parking
11. **Clarify as Someday/Maybe** (US-GTD-2c) -- Inbox -> someday/maybe
12. **Weekly Review** (US-GTD-7) -- The review process
13. **Data Migration + Cleanup** -- Migrate old todos, remove legacy code

Each step follows the Atomic Green Step pipeline. Each step delivers a usable increment.
The clarify workflow is built incrementally -- each clarify path ships as soon as its
destination list is available.

---

## Product Success Criteria (V1 GTD)

V1 GTD is successful when:

1. A user can capture items to their inbox and process them to zero
2. Clarified items land in the correct list (Next Actions, Projects, Waiting For,
   Someday/Maybe, or trash)
3. Next Actions are filterable by context
4. Every active project has at least one next action (stalled projects are flagged)
5. The Weekly Review guided flow keeps the system current
6. The entire system is accessible (WCAG 2.2 AA)
7. The system works without JavaScript (progressive enhancement)
8. A first-time GTD user can understand the system without external instructions
9. All existing auth functionality continues to work
10. All tests pass (Rust integration + Playwright e2e + axe-core a11y)

We will know this is working when a user can complete the full GTD loop (capture ->
clarify -> organize -> review -> engage) without hesitation.
