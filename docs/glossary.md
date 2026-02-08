# Domain Glossary

Maps business terms to Rust types. This is the ubiquitous language of our todo list
application. Code should read like these definitions -- if the business says "complete
a todo," the code says `complete_todo()`.

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
| Edit a todo title | `edit_todo_title()` | Updates the title on an existing todo item |

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
| Invalid title | `AddTodoError::InvalidTitle(TodoTitleError)` | Title validation failure in add_todo service |

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
| `TodoTitleError::Empty` | (silently ignored per US-5 -- empty submissions are not errors) |
| `TodoTitleError::TooLong` | "That title is too long (max 300 characters)" |

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
The `edit_todo_title()` action applies to both states.
Completion is **reversible** -- users can toggle between Pending and Completed (US-6).
