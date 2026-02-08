# Team Agreements

*Formed: Day 0 -- Team Formation Session*
*Facilitator: Kent Beck*
*Status: APPROVED -- Consensus reached*

---

## 1. Working Agreements

### Core Principles
- **Outcomes over output**: We measure success by user problems solved, not features
  shipped. Before starting any feature, the team articulates the user problem it solves
  and how we will know it is working (success criteria).
- **MVP discipline is sacred**: Any idea beyond the defined MVP scope gets acknowledged,
  written on the Future List (`docs/future-ideas.md`), and deferred. The answer to
  "wouldn't it be cool if..." is "Yes -- in v2." Scope expansion requires team consensus,
  and Marty will push back hard unless there is a compelling discovery that changes our
  understanding of the user problem.
- **User stories are conversations, not contracts**: User stories guide our work but are
  owned by the whole team. Anyone can propose changes to acceptance criteria with good
  reasoning.

### Communication
- We communicate openly, directly, and respectfully.
- Every team member's perspective is valued equally -- the accessibility specialist's
  concerns carry the same weight as the lead engineer's.
- When raising a concern, propose an alternative. "I don't like X" is less useful than
  "I'd prefer Y because Z."
- We ask questions rather than make assumptions -- especially across disciplines.

### Mob/Ensemble Programming
- **All production code is written by the mob.** No solo commits to production code.
- **Roles rotate every 10-15 minutes**: Driver (types), Navigator (directs at intent
  level), Mob (contributes ideas, catches issues, thinks ahead).
- **Everyone participates**: Designers, PM, accessibility specialist, domain architect --
  everyone is in the mob. Different eyes catch different things.
- **The navigator describes intent, not keystrokes.** "Let's add a handler for the
  completion endpoint" not "type `async fn`..."
- **Respect the driver.** The driver translates intent into code. Give them space to
  think. Don't dictate syntax.
- **It's okay to say "I don't understand."** The mob is a learning environment.
- **Take breaks.** Mobbing is intense. Short breaks every 45-60 minutes.

### Feature Workflow
Each feature follows this lightweight process in the mob:

1. **Discovery (5 min)**: Who is the user? What is the problem? What does "working"
   look like? Identify edge cases (empty state, error state, 1000 items, special
   characters). Marty leads.
2. **Type sketch (5 min)**: What domain types and states are involved? Quick sketch of
   enums, newtypes, and workflows. Scott leads.
3. **Design intent (5 min)**: What is the visual hierarchy? What states need designing
   (empty, loading, error, success)? Steve Schoger describes layout, spacing, and
   typography in concrete terms.
4. **TDD cycle**: Red-Green-Refactor. The compiler is our first test. Take the smallest
   step possible. Kent facilitates.
5. **Continuous review**: Accessibility checked by Heydon, usability checked by Steve
   Krug, design checked by Steve Schoger, hypermedia patterns checked by Carson -- all
   in real time, not as a separate phase.

### Decision Cadence
- Day-to-day coding decisions (naming, small refactors, local patterns) are made
  naturally in the mob -- no formal process needed.
- Significant decisions (architecture, patterns, library choices, UX patterns, design
  system changes) use the consensus protocol defined in Section 4.

---

## 2. Definition of Done

A feature is **done** when ALL of the following are true:

### Product
- [ ] The user problem is clearly solved
- [ ] All acceptance criteria from the user story are met
- [ ] A first-time user can accomplish the task without hesitating (Trunk Test)
- [ ] Edge cases are handled gracefully (not just the happy path)
- [ ] The feature is deployable -- not just merged, but shippable to real users

### Engineering
- [ ] Tests are written first (TDD: Red-Green-Refactor) and all pass
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo test` passes (unit + integration)
- [ ] Error cases return meaningful HTML responses with structured logging
- [ ] Tracing spans cover the new functionality
- [ ] Database migrations (if any) are committed and `sqlx-data.json` is updated

### Accessibility (WCAG 2.2 AA)
- [ ] HTML is semantic and uses correct elements (`<button>`, `<label>`, `<ul>`, etc.)
- [ ] All form inputs have visible `<label>` elements (no placeholder-only inputs)
- [ ] All interactive elements are fully keyboard operable
- [ ] Screen reader testing (VoiceOver, NVDA, or equivalent) confirms: interactive
      elements are announced correctly, state changes are communicated, and dynamic
      content updates are announced via live regions
- [ ] ARIA used only when native HTML semantics are insufficient (with comments explaining why)
- [ ] Color contrast meets WCAG AA (4.5:1 for normal text, 3:1 for large text/UI)
- [ ] Color is never the only indicator of state (always a secondary visual cue)
- [ ] Focus management is correct after HTMX swaps (see HTMX Focus Protocol below)
- [ ] Custom `:focus-visible` styles are present with sufficient contrast
- [ ] `prefers-reduced-motion` is respected for all animations/transitions

### Design
- [ ] Visual design follows the established design token system
- [ ] All states are designed: empty, loading, error, success
- [ ] Visual hierarchy is clear -- one primary focal point per screen
- [ ] Spacing is consistent and uses the defined scale

### Frontend
- [ ] Core functionality works via standard HTML forms and links (progressive
      enhancement -- HTMX and TypeScript enhance but are not required for basic operation)
- [ ] HTMX attributes follow locality of behavior (self-describing elements)
- [ ] TypeScript (if any) is non-critical and has accessible alternatives

### Process
- [ ] The mob has continuously reviewed code, design, usability, and accessibility
      during development
- [ ] Labels and copy follow the Copy Standards (see Section 3)

---

## 3. Code Conventions and Patterns

### Project Structure

```
src/
  main.rs              -- Startup, configuration loading, server binding
  lib.rs               -- Re-exports for integration test access
  configuration.rs     -- Config structs, environment-aware loading
  startup.rs           -- Application builder (Router, DB pool, middleware)
  routes/              -- HTTP handlers (thin: extract, delegate, respond)
    mod.rs
    health_check.rs
  domain/              -- Pure domain types and logic (NO I/O)
    mod.rs
  services/            -- Application services (orchestrate domain + infra)
    mod.rs
  infrastructure/      -- Database queries, external service clients
    mod.rs
  templates/           -- Askama HTML templates
tests/
  api/                 -- Integration tests against the full HTTP stack
    mod.rs
    helpers.rs         -- TestApp, spawn_app(), shared fixtures
    health_check.rs
```

Dependencies flow inward: routes -> services -> domain + infrastructure.
Domain depends on nothing. This makes the domain layer trivially testable.

### Rust Conventions

**Type-driven domain modeling:**
- Newtypes for all domain identifiers and constrained values: `UserId(Uuid)`,
  `Email(String)`, `Title(String)`, `TodoItemId(Uuid)`
- Enums for state machines -- make illegal states unrepresentable:
  ```rust
  // GOOD: the type system prevents invalid states
  enum TodoItem {
      Pending { title: Title, created_at: DateTime<Utc> },
      Completed { title: Title, created_at: DateTime<Utc>, completed_at: DateTime<Utc> },
  }
  ```
- **Parse, don't validate**: Instead of `fn validate(input: &str) -> bool`, write
  `fn parse(input: String) -> Result<Email, ValidationError>`. Validation that returns
  a boolean throws away information. Parsing produces a typed value that carries the
  proof of validity. Validate once at the boundary; never re-validate inside the system.

**Workflows as type-safe pipelines:**
```
UnvalidatedInput -> ValidatedInput -> DomainEvent -> PersistenceResult
```
Each step's output type is the next step's input. You cannot skip validation or
persist before validating.

**Error handling (three layers):**
- **Domain errors**: Custom enum with `thiserror`. Domain terms only -- no HTTP concepts,
  no infrastructure types. A `TodoError::TitleEmpty` must never contain a `sqlx::Error`.
  Domain errors and infrastructure errors are separate types, translated at the boundary.
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum TodoError {
      #[error("Todo title cannot be empty")]
      TitleEmpty,
      #[error("Todo title too long (max {max}, got {actual})")]
      TitleTooLong { max: usize, actual: usize },
      #[error("Todo not found")]
      NotFound { id: TodoItemId },
      #[error(transparent)]
      Unexpected(#[from] anyhow::Error),
  }
  ```
- **Service errors**: Map domain errors to actionable responses. Add context.
- **Route errors**: Implement `IntoResponse` for error types. Return meaningful HTML
  (not raw 500s). Every error path produces a user-facing response.
- **Always add context** when propagating errors. A bare `?` that loses context is a
  code smell.
- **Log at error boundaries**: Every error crossing an architectural boundary gets
  logged with structured `tracing` fields.

**Idiomatic Rust:**
- Prefer iterators over loops, pattern matching over conditionals
- `?` operator for error propagation with context
- `#[must_use]` on important return types
- Compile-time guarantees: SQLx `query!`/`query_as!` macros, Askama templates

**Testing strategy (TDD):**
- **Unit tests** (`#[cfg(test)] mod tests`): Pure domain logic, validation, type
  conversions. Fast, no I/O, no mocking. **Litmus test: if you need a mock to test
  domain logic, I/O has leaked into the domain layer. Push it back to the boundary.**
- **Integration tests** (`tests/` directory): Full HTTP stack with real database.
  `TestApp` struct with `spawn_app()` helper. Unique database per test run
  (random name, run migrations, drop after).
- **No mocks for the database**: Use real PostgreSQL via Docker. Mocks give false
  confidence.
- **Test names as behavior specifications**:
  ```rust
  #[tokio::test]
  async fn registration_returns_422_when_email_is_missing() { ... }
  ```
- **Acceptance tests as usability statements**: "When I type a todo and press Enter,
  it appears in my list." Tests describe what the user expects to happen.
- **Property-based tests** with `proptest` where appropriate.
- **Accessibility testing** (four layers):
  - *Automated*: axe-core (or equivalent) in the test suite for missing labels,
    contrast violations, and structural issues
  - *Keyboard*: Manual keyboard navigation testing for every interactive flow
  - *Screen reader*: Manual VoiceOver/NVDA testing for every feature
  - *Structural*: HTML validation for semantic correctness
  Tests can assert on roles, labels, and ARIA attributes -- documenting the expected
  screen reader experience as living specifications.

### Initial Crate List

Proposed starting dependencies (open to mob discussion):

| Crate | Purpose |
|-------|---------|
| `axum` | Web framework |
| `tokio` (full features) | Async runtime |
| `sqlx` (postgres, migrate, runtime-tokio-rustls) | Database |
| `askama` + `askama_axum` | Templates |
| `serde` + `serde_json` | Serialization |
| `tracing` + `tracing-subscriber` (env-filter) | Observability |
| `thiserror` | Domain error types |
| `anyhow` | Ad-hoc error context |
| `argon2` | Password hashing |
| `tower-sessions` + `tower-sessions-sqlx-store` | Session management |
| `uuid` | Identifiers |

Note: Input validation will be handled through custom newtypes with validation in
constructors (Scott's approach), not a validation framework. The types carry the
proof of validity.

### HTML / HTMX Conventions

**The Hypermedia Contract:**
- **Return HTML, not JSON.** Every UI endpoint returns HTML. No JSON API layer.
- **Semantic HTML first**: `<button>` for actions, `<a>` for navigation, `<form>` for
  data submission, `<ul>`/`<ol>` for lists. Never `<div onclick>`.
- **Form accessibility**: Every input has a visible `<label>` (never placeholder-only).
  Validation errors are associated with inputs via `aria-describedby` and announced
  via `aria-live="polite"` regions. Use `aria-live="assertive"` only for critical errors.
- **Locality of behavior**: Every HTMX-enhanced element is self-describing. You
  understand what it does by looking at it alone.
- **No client-side state**: The DOM is the state, the server is the source of truth.
  No client-side data stores. The only acceptable client-side state is transient UI
  state for progressive enhancement (e.g., drag position).

**Server endpoint conventions:**
- **Full page requests** (no `HX-Request` header): Return complete HTML page with
  layout, nav, landmarks.
- **HTMX requests** (with `HX-Request` header): Return just the HTML fragment for
  the swap.
- **`HX-Trigger` response header** for cross-component updates (e.g., updating a
  todo count after deletion).
- **Same URL, same handler**: One handler checks the `HX-Request` header to decide
  whether to return a full page or a fragment. No separate `/api/` routes.
- **HTTP methods used semantically**: GET = read, POST = create, PUT/PATCH = update,
  DELETE = delete.
- **Status codes**: 200 for success with HTML, 422 for validation errors with error
  HTML, 200 with empty body for deletion (allows HTMX swap to remove element).

**Progressive enhancement (non-negotiable):**
- The app MUST work without JavaScript. Forms submit, links navigate, content is
  readable with HTML alone.
- HTMX enhances the experience (partial updates, no full page reloads).
- TypeScript is only for things neither HTML nor HTMX can do.

**HTMX Focus Management Protocol:**
After every HTMX swap:
- If an element was **removed**: Move focus to the logical next element.
- If content was **updated in place**: Keep focus where it was.
- Use `aria-live="polite"` regions to announce state changes to screen readers.
- Test every HTMX interaction with Tab key before considering it done.

**Page structure (every page):**
- Skip link to main content (first focusable element)
- Landmark regions: `<main>`, `<nav>`, `<header>`, `<footer>`
- Logical heading hierarchy: one `<h1>` per page, no skipped levels
- Descriptive `<title>` element

### CSS Conventions

**Progressive enhancement layers:**
1. **Layer 1 (HTML)**: Functional and readable with zero CSS.
2. **Layer 2 (CSS)**: Beautiful -- layout, typography, color, transitions, focus styles.
3. **Layer 3 (TypeScript)**: Delightful -- drag-and-drop, advanced keyboard shortcuts.

**CSS organization with cascade layers:**
```css
@layer reset, tokens, base, layout, components, utilities;
```
- **reset**: Normalize/reset styles
- **tokens**: Custom properties (design tokens)
- **base**: Element-level styles (body, headings, links, forms)
- **layout**: Page-level layout (CSS Grid)
- **components**: Component-specific styles (todo-item, auth-form)
- **utilities**: Helper classes (visually-hidden, etc.)
- No `!important` unless in the utilities layer for intentional overrides.
- CSS nesting is fine for component-scoped styles but avoid nesting deeper than
  2 levels.

**Design tokens (CSS custom properties on `:root`):**
- **Spacing scale** (4px base): 4, 8, 12, 16, 24, 32, 48, 64, 96 -- encoded as
  `--space-xs` through `--space-3xl`
- **Color palette**: oklch color space for perceptual uniformity. One primary color,
  neutral gray palette (50-900), semantic colors (danger, success, warning). Most UI
  is grayscale; color is for emphasis and interaction.
- **Typography**: System font stack (`system-ui, sans-serif`). Type scale from xs to
  3xl with defined line-height. Tighter for headings, looser for body.
- **Shadows**: Three levels (sm, md, lg). Never combine shadows and borders on the
  same element.
- **Border radius**: Three sizes (sm, md, lg).
- **All values reference tokens -- no hardcoded hex values or pixel values in
  component styles.**

**Design tokens are a prerequisite**: The token system (spacing, colors, typography,
shadows, radii) must be established and agreed upon before any feature UI work begins.
This is foundational -- without it, inconsistent one-off values accumulate from day one.

**Design token governance**: Adding a new color, spacing value, or typography level
goes through the mob (Steve Schoger approves design token changes).

**Design token flow**: Steve Schoger defines the visual design -> Lea encodes as CSS
custom properties -> Components reference only tokens -> Themes/dark mode reassign
properties. No build step. No JSON-to-CSS pipeline.

**Modern CSS features:**
- `:has()` for parent-based styling (e.g., `li:has(input:checked)` for completed todos)
- Container queries where components need to respond to container size
- Logical properties (`margin-inline`, `padding-block`) for future RTL support
- `clamp()` for fluid typography
- View Transitions API for smooth HTMX swap animations
- `@supports` for graceful degradation of newer features

**Accessibility in CSS:**
- Custom `:focus-visible` styles with at least 3:1 contrast. Never `outline: none`
  without a replacement.
- `prefers-reduced-motion`: All animations/transitions wrapped in a motion query.
  Reduced or no motion as the alternative.
- `prefers-color-scheme`: Supported if/when the team decides to add dark mode.
- Color is never the only state indicator.

**Responsive design**: Mobile-first. Default styles target the smallest viewport;
enhancements added via `min-width` media queries. Breakpoints: mobile (default),
desktop (~1024px). A todo list should work beautifully on a phone.

**Browser targets**: Last 2 versions of evergreen browsers (Chrome, Firefox, Safari,
Edge). No IE11. `@supports` for graceful degradation. No polyfills unless necessary.

### TypeScript Conventions
- **No framework**: Vanilla TypeScript with DOM APIs.
- **Progressive enhancement pattern**: Feature works without JS, TypeScript enhances.
- **Event delegation**: Listeners on parent containers, not individual elements.
- **State via data attributes**: TypeScript toggles `data-*` attributes or classes;
  CSS handles all visual changes based on those.
- **No CSS-in-JS**: Styles live in CSS files. TypeScript never sets inline styles
  (except computed values like drag position).
- **Thin layer**: If a TypeScript file is getting large, we are probably doing
  something CSS or HTMX could handle.

### Copy Standards
- **Labels**: 1-2 words when possible.
- **Error messages**: Plain language. Say what went wrong, suggest what to do next.
  "That email isn't registered -- want to create an account?" not "Error: User not found."
- **Button labels**: Describe the action ("Add todo", "Sign in"), not vague labels
  ("Submit", "OK").
- **No happy talk**: Every word earns its place or gets cut. No introductory paragraphs
  that say nothing useful.
- **Design for scanning, not reading**: Users don't read pages, they scan. Clear
  headings, short text, prominent calls to action, no walls of text.
- **Cut the text, then cut it again**: If you wrote a paragraph, ask "can I say this
  in half the words?"

### Domain Language and Modeling Conventions
- Code uses the ubiquitous language of the domain. "Complete a task" ->
  `complete_task()`, not `update_status()`.
- Domain types are explicit -- no primitive obsession. Every domain concept gets its
  own type, even if it is "just a string" or "just a UUID" underneath.
- Newtypes are constructed only through validated factory methods (the `parse`
  pattern). You cannot create a `TodoTitle` from an arbitrary string without going
  through validation.
- Every `is_*` boolean flag is a candidate for an enum. Ask: "what states does this
  represent?" and model them explicitly.
- Maintain a domain glossary in `docs/glossary.md` mapping business terms to types:

  | Domain Term | Rust Type | Notes |
  |-------------|-----------|-------|
  | User | `User` | Authenticated user with valid credentials |
  | Email | `ValidatedEmail` | Constructed via `parse`, not `new` |
  | Todo item | `TodoItem` (enum) | `Pending` or `Completed` variant |
  | Todo title | `TodoTitle` | Non-empty, max length, validated at construction |

### Observability

**Structured logging with `tracing`:**
- Request ID (UUID) attached to every request via middleware.
- Span fields: `request_id`, `method`, `uri`, `status_code`.
- Application-level spans for significant operations: `register_user`,
  `authenticate`, `create_todo`, etc.
- Use `#[tracing::instrument]` for service functions.
- Structured fields on errors: what failed, why, relevant IDs.

**Log level conventions:**
| Level | When to use |
|-------|-------------|
| `ERROR` | Something is broken, needs human attention |
| `WARN` | Unexpected but handled (e.g., invalid login attempt) |
| `INFO` | Significant business events (user registered, todo created) |
| `DEBUG` | Technical details useful for debugging |
| `TRACE` | Very verbose (request/response bodies, SQL queries) |

### Database Conventions
- **Migrations**: SQLx migrations (`sqlx migrate`), numbered SQL files, version controlled.
- **Compile-time checked queries**: `sqlx::query!` and `sqlx::query_as!` macros.
- **Offline mode for CI**: `sqlx-data.json` generated via `cargo sqlx prepare`,
  committed to version control.
- **Connection pool**: `PgPool` shared via Axum state.
- **UUIDs for primary keys**: `uuid` type in PostgreSQL, `Uuid` in Rust.
- **Timestamps**: `timestamptz` in PostgreSQL, `chrono::DateTime<Utc>` or
  `time::OffsetDateTime` in Rust.
- **Naming conventions**: snake_case for tables and columns. Plural table names
  (`users`, `todos`).

### Configuration Management
- Layered configuration: `base.yaml` (defaults) -> `local.yaml` (gitignored overrides)
  -> environment variables (12-factor style).
- **Fail fast**: Missing required configuration at startup panics with a clear message.
  No silent defaults for critical settings (database URL, secret keys).
- Separate configuration structs: `DatabaseSettings`, `ApplicationSettings`,
  `AuthSettings`.

### CI/CD Quality Gates

Every commit must pass (in order):
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test` (unit + integration)
4. `cargo audit` (dependency vulnerabilities)

### Dependency Management

Before adding a new crate:
- Is it well-maintained? (Recent commits, responsive maintainers)
- Is it widely used? (Downloads, reverse dependencies)
- Compatible license? (MIT/Apache-2.0 preferred)
- Can we avoid it? (Prefer stdlib or existing dependencies)

### Documentation
- **Code comments**: Only where "why" is not obvious from the code.
- **ADRs**: For significant technical decisions. Format: context, decision,
  consequences. Stored in `docs/adr/`.
- **Domain glossary**: `docs/glossary.md` mapping business terms to code.
- **README**: Practical -- how to set up, run tests, run the app.

---

## 4. Disagreement Resolution

### In the Mob (Day-to-Day)
1. State the disagreement clearly: "I think X, you think Y."
2. Each side explains their reasoning briefly (< 2 minutes each).
3. If possible, **try both and let the code decide** -- write a quick test or prototype.
4. If one approach is simpler, prefer it (smallest viable decision).
5. If still stuck after 5 minutes, the mob votes. Majority wins for day-to-day decisions.
   The dissenter can say "I disagree but won't block."

### For Significant Decisions (Consensus Protocol)
1. Any team member raises the decision point with a proposal and reasoning.
2. Each team member shares their perspective (concerns, alternatives, support).
3. Discussion continues until:
   - **Consensus**: Everyone agrees or consents ("I can live with this").
   - **Compromise**: A synthesized solution addresses all concerns.
   - **Stand-aside**: "I disagree but won't block" -- recorded and respected.
4. If no consensus after **10 rounds** of substantive discussion, **escalate to the
   project owner** with:
   - Options considered
   - Each member's position
   - Trade-offs
   - Team recommendation (if possible)

### When to Involve the Project Owner
- Consensus deadlock after 10 rounds
- Any proposed change to MVP scope
- Questions about target user or business context the team cannot answer
- Major architectural pivots that significantly change the timeline
- Brief status updates at milestones (keep the owner informed, don't go dark)

### Principles
- Disagree with the idea, not the person.
- Once decided, **disagree and commit** -- no undermining, no "I told you so."
- Revisit decisions when new information emerges (from tests, users, or production).
- **Write it down**: Major decisions recorded as Architecture Decision Records (ADRs)
  in `docs/adr/`.

---

## 5. Architectural Principles

1. **Simplicity first**: Choose the simplest approach that could work. Add complexity
   only when tests or user feedback demand it. "Could this be simpler?" is always a
   valid question.
2. **Server-side state**: The server is the source of truth for both data and
   presentation. The browser renders what the server sends. No client-side state
   management.
3. **Pure domain core**: Domain logic is pure functions and types with no I/O. Side
   effects live at the boundaries. This makes the domain trivially testable.
4. **Make illegal states unrepresentable**: Use the type system to prevent bugs at
   compile time. Newtypes, enums, and validated types carry proof of correctness.
5. **Production-ready from day one**: Observability, error handling, and structured
   logging from the first line of code -- not bolted on later.
6. **Accessibility is a quality, not a feature**: Every feature is accessible from the
   moment it is built. Inaccessible is broken. ~15-20% of users have some form of
   disability -- that is not an edge case.
7. **Progressive enhancement**: HTML works -> CSS makes it beautiful -> JS makes it
   delightful. Each layer enhances without being required by the layers above.
8. **Conventions over cleverness**: Follow established patterns. A login form should
   look like every other login form. Don't make the user think. Design for scanning,
   not reading. Cut unnecessary words. When in doubt, simplify.
9. **Continuous testing**: TDD keeps us honest. The compiler is our first test. We
   never skip the red step. Tests read like behavior specifications.
10. **Incremental design**: Let the design emerge from tests and feedback. Don't
    over-design upfront. Make the code a little better before adding a feature (Tidy
    First), but don't redesign the universe. Ship early, learn fast.
11. **Walking skeleton first**: Our first milestone is the thinnest possible slice
    through the entire stack -- a user can register, log in, see an empty todo list,
    add one item, and log out. Everything else builds on top.
12. **Validate at the boundary, trust the types**: Input validation happens once, at
    the system boundary. It produces typed values that carry proof of validity.
    The interior of the system trusts its own types.

---

## 6. Retrospective Cadence

All team members are AI agents operating continuously. Time-based retros (weekly,
bi-weekly) do not apply. Instead, retros are triggered by events.

- **After each feature**: Retro immediately after completing a feature. What worked?
  What didn't? What should we change?
- **After significant incidents**: If a build breaks, a major refactor goes sideways,
  or the team gets stuck for an extended period, retro after resolution.
- **Format**: Simple Start/Stop/Continue.
- **Action items**: Every retro produces at most 2-3 concrete action items. These are
  tracked and checked at the next retro.
- **Agreements are living**: These team agreements can be updated at any retro if the
  team reaches consensus.
- **Usability testing**: Aim for informal testing -- three users, 20 minutes each --
  at natural breakpoints. Even watching one person use the app for 5 minutes reveals
  more than a week of discussion.

---

## 7. Tooling & Repository

### Source Control
- **Repository**: Public GitHub repository under the `jwilger` account
- **License**: MIT
- **Branching**: Work on `main` for now. Feature branches if/when the team decides
  they are needed.

### CI/CD
- **GitHub Actions**: CI runs on every push and pull request to `main`.
- **Pipeline stages** (in order):
  1. `cargo fmt --check`
  2. `cargo clippy -- -D warnings`
  3. `cargo test`
- Rust nightly toolchain (matching the project's `rust-toolchain.toml`)
- CI must pass before merging any pull request.

### Work Tracking
- **GitHub Issues**: For tracking features, bugs, and tasks visible to the project owner.
- **Claude Task tool**: For in-progress coordination between team members during a session.
- Issues and tasks should reference each other when applicable.

---

## 8. Team Composition Assessment

### Current Team (9 members)
| Name | Role | Domain |
|------|------|--------|
| Kent Beck | TDD Coach & Dev Practice Lead | TDD, XP, refactoring |
| Scott Wlaschin | Domain Architect | DDD, type-driven design, FP |
| Luca Palmieri | Lead Rust Engineer | Axum, SQLx, production Rust |
| Carson Gross | Hypermedia Architect & Frontend Lead | HTMX, hypermedia systems |
| Lea Verou | Frontend Engineer | CSS, web standards |
| Steve Schoger | UI Designer | Visual design, design systems |
| Steve Krug | UX Specialist | Usability, user advocacy |
| Heydon Pickering | Accessibility Specialist | WCAG, inclusive design |
| Marty Cagan | Product Manager | Product discovery, MVP |

### Assessment
The current team covers all necessary disciplines for this MVP:
- **Product**: Marty Cagan
- **Design**: Steve Schoger (visual) + Steve Krug (UX) + Heydon Pickering (a11y)
- **Architecture**: Scott Wlaschin (domain) + Carson Gross (hypermedia) + Luca
  Palmieri (Rust/infra)
- **Frontend**: Carson Gross (HTMX) + Lea Verou (CSS/HTML) + Heydon Pickering
  (a11y review)
- **Backend**: Luca Palmieri + Scott Wlaschin
- **Process**: Kent Beck (TDD, mob facilitation)

### Potential Gaps to Monitor
- **DevOps/Infrastructure**: No dedicated ops person. Luca has deployment experience
  but if CI/CD or infrastructure becomes complex, we may want to recruit.
- **Security**: Auth is in scope. Luca has experience but if we encounter complex
  security requirements, consider a security specialist.
- **QA/Testing beyond TDD**: The team practices TDD rigorously, but exploratory
  testing and load testing may need attention for production readiness.
- **Analytics/Measurement**: If we are serious about "outcomes over output," we will
  eventually need to measure whether the product solves the user's problem. Structured
  logging (`tracing`) is sufficient for MVP. Analytics instrumentation is a post-MVP gap.

**Recommendation**: The current 9-person team is sufficient for the MVP. Revisit at
the first retrospective after the core features are built.

---

## Signatures (Consensus Record)

By agreeing to these terms, each team member commits to following these agreements
and raising concerns through the established process when they feel the agreements
need updating.

- [x] Kent Beck
- [x] Scott Wlaschin
- [x] Luca Palmieri
- [x] Carson Gross
- [x] Lea Verou
- [x] Steve Schoger
- [x] Steve Krug
- [x] Heydon Pickering
- [x] Marty Cagan
