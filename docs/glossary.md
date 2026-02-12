# Domain Glossary

Maps business terms to Rust types. This is the ubiquitous language of our GTD (Getting
Things Done) application. Code should read like these definitions -- if the business
says "capture to inbox," the code says `capture_to_inbox()`.

Maintained by the mob. Changes reviewed by the domain architect (Scott Wlaschin).

---

## Core Domain Types

| Domain Term | Rust Type | Module | Notes |
|-------------|-----------|--------|-------|
| User | `User` | `domain` | An authenticated user with valid credentials |
| User ID | `UserId(Uuid)` | `domain` | Newtype wrapper; uniquely identifies a user |
| Email address | `ValidatedEmail` | `domain` | Constructed via `ValidatedEmail::parse()`; validated format, normalized to lowercase (case-insensitive per US-2) |
| Password | (not stored as domain type) | -- | Hashed with Argon2 at the boundary; the domain never sees raw passwords |
| Todo item | `TodoItem` (enum) | `domain` | Sum type: `Pending` or `Completed` variant. No boolean flags. |
| Todo item ID | `TodoItemId(Uuid)` | `domain` | Newtype wrapper; uniquely identifies a todo item |
| Todo title | `TodoTitle` | `domain` | Non-empty, max 300 chars (per US-5), trimmed; constructed via `TodoTitle::parse()` |
| Pending todo | `TodoItem::Pending { ... }` | `domain` | A todo that has not been completed; has title and created_at |
| Completed todo | `TodoItem::Completed { ... }` | `domain` | A todo that has been completed; has title, created_at, and completed_at |

## Domain Actions (Ubiquitous Language)

| Business Action | Rust Function / Method | Notes |
|-----------------|----------------------|-------|
| Register a user | `register_user()` | Service: creates user with validated email and hashed password |
| Log in | `authenticate_user()` | Service: verifies credentials; route stores user_id in session |
| Log out | `post_logout()` | Route: flushes session, redirects to login |
| Add a todo | `add_todo()` | Service: parses title, creates `TodoItem::Pending`, persists |
| View todos | `get_todos()` | Service: fetches all todos for authenticated user |
| Toggle completion | `toggle_todo_completion()` | Service: looks up todo, verifies ownership, toggles state |
| Complete a todo | `TodoItem::complete()` | Domain method: Pending -> Completed; records completion time |
| Uncomplete a todo | `TodoItem::uncomplete()` | Domain method: Completed -> Pending; drops completion time |
| Delete a todo | `delete_todo()` | Service: verifies ownership, permanently removes from DB |
| Edit a todo title | `update_todo_title()` | Service: verifies ownership, parses new title, persists |

## Domain Errors

| Error Condition | Rust Type | Notes |
|-----------------|-----------|-------|
| Title is empty | `TodoTitleError::Empty` | Todo title cannot be blank |
| Title too long | `TodoTitleError::TooLong { max, actual }` | Exceeds maximum allowed length |
| Todo not found (toggle) | `ToggleTodoError::NotFound` | Referenced todo does not exist |
| Not authorized (toggle) | `ToggleTodoError::Unauthorized` | User does not own the todo |
| Todo not found (delete) | `DeleteTodoError::NotFound` | Referenced todo does not exist |
| Not authorized (delete) | `DeleteTodoError::Unauthorized` | User does not own the todo |
| Email invalid | `RegistrationError::InvalidEmail(EmailValidationError)` | Email fails format validation |
| Duplicate email | `RegistrationError::DuplicateEmail` | Registration attempted with existing email |
| Invalid credentials | `AuthenticationError::InvalidCredentials` | Login failed (generic -- no info leak) |
| Invalid title (add) | `AddTodoError::InvalidTitle(TodoTitleError)` | Title validation failure in add_todo service |
| Todo not found (edit) | `UpdateTitleError::NotFound` | Referenced todo does not exist |
| Not authorized (edit) | `UpdateTitleError::Unauthorized` | User does not own the todo |
| Invalid title (edit) | `UpdateTitleError::InvalidTitle(TodoTitleError)` | Title validation failure in update_todo_title service |
| Context name empty | `ContextNameError::Empty` | Context name cannot be blank |
| Context name too long | `ContextNameError::TooLong { max, actual }` | Context name exceeds 50 character limit |
| Invalid title (add next action) | `AddNextActionError::InvalidTitle(TodoTitleError)` | Title validation failure in add_next_action service |
| Next action not found (complete) | `CompleteNextActionError::NotFound` | Referenced next action does not exist |
| Not authorized (complete next action) | `CompleteNextActionError::Unauthorized` | User does not own the next action |
| Next action not found (delete) | `DeleteNextActionError::NotFound` | Referenced next action does not exist |
| Not authorized (delete next action) | `DeleteNextActionError::Unauthorized` | User does not own the next action |
| Next action not found (edit) | `UpdateNextActionTitleError::NotFound` | Referenced next action does not exist |
| Not authorized (edit next action) | `UpdateNextActionTitleError::Unauthorized` | User does not own the next action |
| Invalid title (edit next action) | `UpdateNextActionTitleError::InvalidTitle(TodoTitleError)` | Title validation failure in update_next_action_title service |
| Inbox item not found (clarify) | `ClarifyAsNextActionError::NotFound` | Referenced inbox item does not exist |
| Not authorized (clarify) | `ClarifyAsNextActionError::Unauthorized` | User does not own the inbox item |
| Invalid title (add project) | `AddProjectError::InvalidTitle(TodoTitleError)` | Title validation failure in add_project service |
| Project not found (complete) | `CompleteProjectError::NotFound` | Referenced project does not exist |
| Not authorized (complete project) | `CompleteProjectError::Unauthorized` | User does not own the project |
| Project not found (delete) | `DeleteProjectError::NotFound` | Referenced project does not exist |
| Not authorized (delete project) | `DeleteProjectError::Unauthorized` | User does not own the project |
| Project not found (edit) | `UpdateProjectTitleError::NotFound` | Referenced project does not exist |
| Not authorized (edit project) | `UpdateProjectTitleError::Unauthorized` | User does not own the project |
| Invalid title (edit project) | `UpdateProjectTitleError::InvalidTitle(TodoTitleError)` | Title validation failure in update_project_title service |
| Project not found (add action) | `AddProjectNextActionError::ProjectNotFound` | Referenced project does not exist |
| Not authorized (add project action) | `AddProjectNextActionError::Unauthorized` | User does not own the project |

## Error Copy Convention

Domain error messages (`#[error("...")]`) describe **what invariant was violated** -- they
are written for developers and logs. User-facing error messages are **instructions** that
tell the user what to fix -- they are written for humans scanning a form.

The route/template layer translates domain errors into user-facing copy. Templates never
call `error.to_string()` on a domain error.

| Domain Error | User-Facing Copy |
|-------------|-----------------|
| `EmailValidationError::Empty` | "Enter your email address" |
| `EmailValidationError::MissingAtSymbol` | "Enter a valid email address, like name@example.com" |
| `EmailValidationError::TooLong` | "That email address is too long" |
| `PasswordError::Empty` | "Enter a password" |
| `PasswordError::TooShort` | "Your password needs at least 8 characters" |
| `PasswordError::TooLong` | "That password is too long" |
| `RegistrationError::DuplicateEmail` | "Unable to create account. If you already have an account, try signing in." |
| `AuthenticationError::InvalidCredentials` | "That email or password didn't work. Try again." |
| `TodoTitleError::Empty` (add) | (silently ignored per US-5 -- empty submissions are not errors) |
| `TodoTitleError::Empty` (edit) | "Title cannot be empty" |
| `TodoTitleError::TooLong` | "That title is too long (max 300 characters)" |

| `TodoTitleError::Empty` (add next action) | (silently ignored -- empty submissions are not errors) |
| `TodoTitleError::Empty` (edit next action) | "Title cannot be empty" |
| `TodoTitleError::TooLong` (next action) | "That title is too long (max 300 characters)" |

*Copy reviewed by Steve Krug. Update this table when adding new error types.*

---

## Type Design Principles

These principles govern how we model the domain:

1. **Parse, don't validate**: `TodoTitle::parse(input) -> Result<TodoTitle, TodoError>`
   not `fn is_valid_title(input: &str) -> bool`. Parsing produces a typed value that
   carries proof of validity. Validate once at the boundary; never re-validate inside.

2. **Make illegal states unrepresentable**: A `TodoItem` is either `Pending` or
   `Completed` -- never both, never neither. The enum enforces this at compile time.
   No boolean `is_completed` flag with an `Option<completed_at>` that could be
   inconsistent.

3. **Newtypes for everything**: `UserId` is not a bare `Uuid`. `ValidatedEmail` is
   not a bare `String`. The type system prevents mixing them up. These are zero-cost
   abstractions in Rust.

4. **Pure domain core**: Everything in `src/domain/` is pure functions and types. No
   database imports, no HTTP types, no I/O. If you need a mock to test domain logic,
   I/O has leaked into the domain.

5. **Workflows as type-safe pipelines**:
   ```
   UnvalidatedInput -> ValidatedInput -> DomainResult
   ```
   Each step's output type is the next step's input. You cannot skip validation.

## State Machine: TodoItem

```
                 add_todo()
  [nothing] ────────────────> Pending
                                 │  ^
               item.complete()   │  │  item.uncomplete()
                                 │  │
                                 v  │
                             Completed
```

Both `Pending` and `Completed` variants can be deleted (`delete_todo()`).
The `update_todo_title()` action applies to both states.
Completion is **reversible** -- users can toggle between Pending and Completed (US-6).

---

## GTD Domain Types (V1 -- Approved)

*These types will replace the simple TodoItem model as we evolve to GTD. Approved by
team consensus. Final type signatures subject to domain architect guidance during
implementation.*

### Core GTD Concepts

| Domain Term | Proposed Rust Type | Notes |
|-------------|-------------------|-------|
| Inbox item | `InboxItem` | Raw, unclarified capture. Has title and created_at. No context, no project link. |
| Next action | `NextAction` (enum: Active, Completed) | A concrete, physical action ready to do. Has context and optional project link. **Implemented.** |
| Next action ID | `NextActionId(Uuid)` | Newtype wrapper; uniquely identifies a next action. **Implemented.** |
| Project | `Project` (enum: Active, Completed, Dropped) | An outcome requiring 2+ actions. Has title and linked next actions. **Implemented.** |
| Project ID | `ProjectId(Uuid)` | Newtype wrapper; uniquely identifies a project. **Implemented.** |
| Context | `Context` | Where/how an action can be performed. User-defined. Defaults: @computer, @home, @errands, @phone, @anywhere. **Implemented.** |
| Context ID | `ContextId(Uuid)` | Newtype wrapper; uniquely identifies a context. **Implemented.** |
| Waiting For item | `WaitingForItem` | Delegated/blocked item. Has title, waiting_on (text), and date added. |
| Someday/Maybe item | `SomedayMaybeItem` | Parked idea. Has title and created_at. Not committed to. |
| Item title | `ItemTitle` | Replaces `TodoTitle`. Non-empty, max 300 chars, trimmed. Same validation. |
| Context name | `ContextName` | Non-empty, max 50 chars, trimmed. Validated at construction via `ContextName::parse()`. **Implemented.** |

### GTD Actions (Ubiquitous Language)

| Business Action | Proposed Rust Function / Method | Notes |
|-----------------|-------------------------------|-------|
| Capture to inbox | `capture_to_inbox()` | Creates InboxItem from raw text |
| Clarify as next action | `clarify_as_next_action()` | InboxItem -> NextAction (requires context). **Implemented.** |
| Clarify as project | `clarify_as_project()` | InboxItem -> Project + first NextAction |
| Delegate | `delegate()` | InboxItem or NextAction -> WaitingForItem |
| Defer | `defer()` | InboxItem or NextAction -> SomedayMaybeItem |
| Trash | (delete) | InboxItem is permanently removed. **Implemented.** |
| Add next action | `add_next_action()` | Service: creates NextAction (Active) with title and context. **Implemented.** |
| Complete action | `NextAction::complete()` | Active -> Completed; records completion time. **Implemented.** |
| Delete next action | `delete_next_action()` | Service: verifies ownership, permanently removes. **Implemented.** |
| Edit next action title | `update_next_action_title()` | Service: verifies ownership, parses new title, persists. **Implemented.** |
| Add project | `add_project()` | Service: creates Project (Active) with title. **Implemented.** |
| Complete project | `Project::complete()` | Active -> Completed; records completion time. **Implemented.** |
| Drop project | `Project::drop_project()` | Active -> Dropped; project abandoned. **Implemented.** |
| Delete project | `delete_project()` | Service: verifies ownership, permanently removes. **Implemented.** |
| Edit project title | `update_project_title()` | Service: verifies ownership, parses new title, persists. **Implemented.** |
| Add next action to project | `add_next_action_to_project()` | Service: creates NextAction linked to project. **Implemented.** |
| Activate someday/maybe | `activate()` | SomedayMaybeItem -> InboxItem (for re-clarification) |
| Resolve waiting for | `resolve()` | WaitingForItem -> completed or moved to inbox |
| Add context | `add_context()` | Creates a new user-defined context |
| Start weekly review | `start_review()` | Initiates the three-phase review flow |

### GTD State Machines

**InboxItem Lifecycle:**
```
                capture()
  [nothing] ──────────────> InboxItem
                                │
                      clarify() │
              ┌────────┬────────┼────────┬────────┐
              v        v        v        v        v
         NextAction  Project  WaitingFor  Someday  [trash]
```

**NextAction Lifecycle:**
```
                  clarify_as_next_action()
  InboxItem ─────────────────────────────> NextAction (Active)
                                               │
                                   complete()  │
                                               v
                                         NextAction (Completed)
```

**Project Lifecycle:**
```
                    clarify_as_project()
  InboxItem ──────────────────────────> Project (Active)
                                           │  │
                              complete()   │  │  drop()
                                           v  v
                                    Completed / Dropped
```
