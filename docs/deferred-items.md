# Deferred Items

Non-blocking a11y, design, and UX items identified during development. Reviewed at
each retrospective.

| Item | Category | Source | Severity | Status |
|------|----------|--------|----------|--------|
| Wire up `user_facing` error message for too-long inbox titles (422 shows no explanation) | UX | Task #15 (Kent, Luca, Heydon) | Refinement | Open |
| `InboxRecord.into_domain()` uses `.expect()` — could panic on corrupted DB data | Engineering | Task #15 (Luca) | Refinement | Open |
| `htmx_response_with_announce` uses `.unwrap()` on HeaderValue — panics on non-ASCII | Engineering | Task #15 (Luca) | Refinement | Open |
| Quick capture `hx-swap="none"` may not fire HX-Trigger for aria-live announcements | A11y | Task #15 (Luca) | Refinement | Open |
| E2e inbox tests use CSS selectors instead of `getByRole`/`getByLabel` convention | Engineering | Task #15 (Luca) | Refinement | Open |
| Extract `register_and_login` test helper to `tests/api/helpers.rs` (duplicated 3x) | Engineering | Task #15 (Kent, Luca) | Refinement | Open |
| Rename `TodoTitle` to `ItemTitle` (shared by todo and inbox items) | Domain | Task #15 (Scott) | Refinement | Open |
| CSS gaps: `todo-add__error`, `nav-logout`, `auth-form__message :empty`, `auth-form__cancel` lack styles | Design | Task #11 (Steve Schoger) | Refinement | Open |
| Password toggle: confirm field removal needs hidden input mirror for no-JS fallback | A11y | Task #11 (Heydon) | Refinement | Open |
| Focus management after HTMX delete (inbox + contexts): focus should move to a logical target after item removal | A11y | Step 3 (Heydon) | Refinement | Open |
| Add optional `project_id` FK to `waiting_for_items` to link Waiting For items to Projects | Domain | Step 8 (Scott) | Refinement | Open |
| Convert Waiting For item to Next Action (US-GTD-5: "User can convert a Waiting For item to a Next Action") | Feature | Step 8 (Marty) | Refinement | Open |
