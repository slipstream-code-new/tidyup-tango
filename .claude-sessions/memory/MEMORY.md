# Project Memory

## Project Overview
- Web-based multi-user todo list app
- Rust (nightly) backend + HTMX + server-rendered templates + TypeScript for progressive enhancement
- Email/password auth, each user gets one todo list (MVP)
- Nix flake + direnv dev environment

## Team Setup (Driver-Reviewer Mob Model — adopted after retro)
- 9-person ensemble team in `.team/` directory (see CLAUDE.md for full roster)
- **Driver-Reviewer model**: 1 Driver (sole writer) + 8 Reviewers (read-only + messages)
- Driver rotates by task based on expertise needed
- All 9 must reach consensus before a task is marked complete
- Atomic Green Step pipeline: code -> fmt -> clippy -> test -> commit+push -> CI green -> consensus
- Coordinator verifies clean working tree before/after each task
- **Team persistence**: Keep reviewers alive between tasks; only respawn outgoing/incoming driver on rotation
- Strict TDD (Kent Beck style: Red-Green-Refactor)
- Team profiles: kent-beck, scott-wlaschin, luca-palmieri, carson-gross, lea-verou, steve-schoger, steve-krug, heydon-pickering, marty-cagan

## Tech Stack (Confirmed)
- Framework: Axum 0.8
- DB: PostgreSQL 16 + SQLx 0.8 (compile-time checked queries)
- Templates: Askama 0.13 (templates/ dir at project root, no askama_axum needed)
- Auth: Session-based with Argon2, tower-sessions 0.14 + tower-sessions-sqlx-store 0.15
- Logging: tracing + tracing-subscriber with env-filter
- Static files: tower-http ServeDir from static/ directory

## Completed Tasks (MVP + POLISH DONE)
- Task #1-2: Scaffolding + walking skeleton (Cargo.toml, docker-compose, health check)
- Task #3: CSS design tokens (main.css with cascade layers, oklch colors, fluid type scale)
- Task #4: Base HTML template + static serving (Askama base.html, skip-link, landmarks)
- Task #5: User registration (GET/POST /register, domain types, Argon2)
- Task #6: Login/logout (GET/POST /login, POST /logout, session mgmt)
- Task #7: View todo list + add todo item (GET/POST /todos, empty state, auth)
- Task #8: Complete a todo item (POST /todos/{id}/toggle)
- Task #9: Delete a todo item (POST /todos/{id}/delete, details/summary confirmation)
- Task #20: Edit a todo item title (GET/POST /todos/{id}/edit)
- Polish: a11y always-in-DOM error elements (login, todos, edit_todo templates)
- Polish: Forgot password placeholder page + link on login (GET /forgot-password)
- Polish: HTMX progressive enhancement (add/toggle/delete without page reload, HX-Request fragment responses)
- Polish: aria-live announcements for HTMX dynamic updates (HX-Trigger + #live-region)
- Polish: Password visibility toggle (TypeScript progressive enhancement on register)
- All 60 integration tests pass, all CI green
- Fix: Index page was dead-end (no nav links), added session-aware routing + nav
- Added Playwright e2e testing (3 browser tests covering full user journey)
- Updated Definition of Done with e2e user journey requirements
- README.md added to project root
- All 97 Rust tests + 3 Playwright e2e tests pass, CI green
- Fix: Welcome page nav links visible to sighted users (added visually-hidden class)
- Fix: Invisible checkbox characters on todo list (base button `color: white` not overridden)
- Fix: `--color-primary` contrast ratio was 4.34:1 (below WCAG AA 4.5:1), adjusted oklch lightness 0.55→0.52
- Fix: `.todo-item__toggle` border lost in Windows High Contrast Mode (forced-colors fix)
- Integrated axe-core for automated WCAG scanning (index + todos pages, pending + completed states)
- Improved e2e tests to test quality over implementation (6 Playwright tests total)
- All 97 Rust tests + 6 Playwright e2e tests pass, CI green

## Lessons Learned
- COMMIT AFTER EVERY GREEN: All code was lost due to uncommitted working directory reset
- DATABASE_URL env var needed for SQLx compile-time checking (use .env file)
- sqlx-cli now in devshell (flake.nix updated)
- tests/api/main.rs is the integration test crate root (not tests/api.rs with mod api;)
- Axum 0.8: `Redirect::to()` is 303 See Other (not `see_other()` which doesn't exist)
- Askama 0.13: Use `template.render()` + `Html()`, no separate askama_axum crate
- Concurrent teammate edits can revert files -- always re-read before editing
- Auth form CSS pattern: `.auth-form` centered, `.auth-form__group` with `:has([aria-invalid])` for error borders
- tower-sessions 0.14 required for Axum 0.8 compatibility (0.13 uses axum-core 0.4)
- tower-sessions-sqlx-store: call PostgresStore::migrate() for auto-schema creation
- CI needs DATABASE_URL with PostgreSQL service for SQLx compile-time checks
- Askama HTML entity encoding: uses `&#39;` not `&#x27;` for apostrophes
- Team discipline issues: teammates repeatedly skip ahead, mark tasks complete prematurely, accumulate uncommitted work. New commit discipline rules adopted to address this.
- A11y: error elements should ALWAYS be in DOM (empty when no error) with role="alert" so aria-describedby resolves
- HTMX fragment responses: check HX-Request header, return fragment vs full page from same handler
- HTMX aria-live: use HX-Trigger response header with "announce" event, client listener updates #live-region
- Concurrent teammate edits STILL happen despite rules -- need to actively monitor and revert non-driver changes
- Password toggle: when confirm field removed from DOM, form won't send password_confirmation -- needs hidden input mirror (known follow-up)
- Steve Schoger flagged CSS gaps: todo-add__error, nav-logout, auth-form__message :empty, auth-form__cancel have no styles
- Dead-end index page shipped because no test verified user could navigate from `/` — fixed with Playwright e2e tests
- Playwright setup: package.json + playwright.config.ts + e2e/ dir; NixOS needs system Chrome (findSystemChrome helper)
- Playwright config uses webServer to auto-build/start Rust app, waits for /health_check
- Playwright locators: use getByRole/getByLabel (a11y-correct), not CSS selectors
- E2e tests handle password-toggle progressive enhancement via networkidle wait + visibility check
- Atomic Green Step pipeline now includes `npx playwright test` as step 5 (before commit)
- Teammates can have stale context about task status — always explicitly update them when tasks are pushed/completed
- Landing page CSS: `.landing-hero` component scoped, centered 24rem max-width, matches auth-form pattern
- Empty nav landmarks are a11y failures — always populate `{% block nav %}` in templates
- Base button styles set `color: white` — any button with `background: none` MUST explicitly set `color` or text becomes invisible
- Duplicate nav links (nav + hero CTAs) should use `visually-hidden` class, not be removed, to preserve screen reader nav landmarks
- Playwright `not.toBeVisible()` does NOT work for `clip-path: inset(50%)` elements — they're still considered "visible"; use `toHaveClass(/visually-hidden/)` instead
- axe-core integration: `@axe-core/playwright` with `AxeBuilder`, use `.withTags(["wcag2a","wcag2aa","wcag21a","wcag21aa","wcag22aa"])` for full WCAG 2.2 AA coverage
- axe-core caught real contrast bug in `--color-primary` that manual color checks missed — validates the approach
- Always use `JSON.stringify(results.violations, null, 2)` as second arg to `expect()` for readable axe failure diagnostics
- Test both pending AND completed todo states with axe — different styling means different contrast characteristics
- Test quality over implementation: prefer `toBeAttached()` + behavioral assertions over checking bounding box sizes or specific CSS values
- `@media (forced-colors: active)` rules needed in components layer when `border: none` overrides base-layer forced-colors borders
- Teammates sometimes go idle without executing commit commands — may need multiple explicit messages to get them to run git operations
