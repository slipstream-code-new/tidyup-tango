# Project Memory

## Project Overview
- Web-based multi-user todo list app evolving into GTD (Getting Things Done) system
- Rust (nightly) backend + HTMX + server-rendered templates + TypeScript for progressive enhancement
- Email/password auth, session-based with Argon2
- Nix flake + direnv dev environment
- V1 scope: Full GTD system (see docs/gtd-product-discovery.md)

## Team Setup (Driver-Reviewer Mob Model)
- 9-person ensemble team in `.team/` directory (see CLAUDE.md for full roster)
- **Driver-Reviewer model**: 1 Driver (sole writer) + 8 Reviewers (read-only + messages)
- Driver rotates by task based on expertise needed
- All 9 must reach consensus before a task is marked complete
- Atomic Green Step pipeline: code -> fmt -> clippy -> test -> refactor -> glossary check -> commit+push -> CI green -> mini-retro -> consensus
- **Post-retro additions**: CI wait rule, driver handoff protocol, reviewer coordination, deferred items tracking
- Team profiles: kent-beck, scott-wlaschin, luca-palmieri, carson-gross, lea-verou, steve-schoger, steve-krug, heydon-pickering, marty-cagan

## Tech Stack (Confirmed)
- Framework: Axum 0.8
- DB: PostgreSQL 16 + SQLx 0.8 (compile-time checked queries)
- Templates: Askama 0.13 (templates/ dir at project root)
- Auth: Session-based with Argon2, tower-sessions 0.14 + tower-sessions-sqlx-store 0.15
- Logging: tracing + tracing-subscriber with env-filter
- Static files: tower-http ServeDir from static/ directory
- HTMX: hx-boost on body, explicit attrs for in-page interactions, auth forms opt out

## Completed Tasks
- MVP: Tasks #1-9, #20 (scaffolding through edit todo) + polish
- GTD Task #1: Product discovery (docs/gtd-product-discovery.md, user stories, glossary)
- GTD Task #11: Navigation + Dashboard skeleton (gtd_nav, dashboard, placeholder pages)
- GTD Task #15: Inbox Capture (US-GTD-1) — inbox_items table, domain type, CRUD routes, HTMX, live badge
- GTD Task #18: Team Retrospective — 13 process improvements adopted (9/9 consensus)
- GTD Step 3: Contexts — ContextId, ContextName, CRUD, default seeding, management UI
- GTD Step 4: Next Actions with Contexts (US-GTD-3) — NextAction domain, CRUD, context filtering
- GTD Step 5: Clarify as Next Action (US-GTD-2a) — inbox → next action with DB transaction
- GTD Step 6: Projects (US-GTD-4) — Project domain (Active/Completed/Dropped), CRUD, stalled detection
- GTD Step 7: Clarify as Project — unified clarify form with radio buttons, CSS :has() disclosure
- GTD Step 8: Waiting For (US-GTD-5) — WaitingForItem (Active/Resolved), WaitingOn newtype, CRUD, dashboard
- GTD Step 9: Clarify as Waiting For — inbox → waiting for with waiting_on field, :has() disclosure
- Current state: 312 Rust tests (85 unit + 227 integration) + 11 Playwright e2e tests, all green

## Next Implementation Steps (from docs/gtd-product-discovery.md)
- Step 10: Someday/Maybe (US-GTD-6)
- Step 11: Clarify as Someday/Maybe
- Step 12: Weekly Review (US-GTD-7)
- Step 13: Data Migration + Cleanup

## Key Process Rules (from Task #18 retro)
- **CI wait rule**: Wait for CI green before pushing next commit, never queue multiple CI runs
- **Refactor step not optional**: Check for duplication/naming before committing
- **Glossary compliance**: New domain types must match docs/glossary.md
- **Driver handoff**: Written summary + git log + full pipeline green + "ready" message
- **Reviewer coordination**: Check others' reviews before writing, "+1" for agreement, don't re-send acknowledged reviews
- **Mini-retro after each CI build**: 1-minute checkpoint
- **Deferred items**: Track in docs/deferred-items.md, review at retros and during discovery
- **Session transcripts excluded from CI**: paths-ignore in .github/workflows/ci.yml

## Coordinator Hard Rules
- **NEVER perform project operations** — no git, cargo, file reads/writes, npm, etc. Only messaging and team management tools.
- **NEVER decide what the team works on next** — relay project owner's needs, but team decides priorities via consensus
- **NEVER run retrospectives** — mini-retros and full retros belong to the team; coordinator butts out
- **Mini-retro is part of the pipeline, not a shutdown ceremony** — the same team that did the work holds the retro within the same session after CI green, then continues to the next task. Never spawn a separate retro team.

## Coordinator Communication Rules
- **Send instructions ONCE.** Do not repeat yourself. If a teammate doesn't respond immediately, they are busy working — not ignoring you.
- **Idle notifications are automatic system events, NOT requests for action.** A teammate going idle means their turn ended; it does NOT mean they are stuck, confused, or need re-prompting. Do not react to idle notifications by re-sending instructions.
- **Never send the same instruction more than once** unless the teammate explicitly says they didn't receive it or asks for clarification.
- **Wait patiently.** Teammates process messages in order. If you send 5 copies of the same message, they waste time reading all 5. Silence from the coordinator is fine — it means "keep working."
- **Only follow up if the teammate explicitly asks a question or reports being blocked.** Time passing is not a reason to re-send.

## Coordinator Lessons Learned
- Driver agents frequently get stuck on completed tasks or terminate without doing work — need explicit, detailed spawn prompts focused on the NEW task
- Drivers may skip blocking review feedback items — always verify ALL blocking issues are addressed before re-requesting review
- When respawning driver, always verify clean working tree by asking the driver to check (not by running git yourself)
- Stale shutdown requests can be picked up by new sessions — instruct agents to reject unexpected shutdowns
- Keep reviewers alive between tasks to preserve context; only rotate driver
- Always broadcast review requests explicitly — don't assume agents will self-start
- Session transcript commits were triggering CI unnecessarily — now excluded via paths-ignore
- CSS is frequently omitted by backend-focused drivers — remind the team to check during review
