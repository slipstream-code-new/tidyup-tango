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
- Current state: 100 Rust tests + 9 Playwright e2e tests, all green

## Next Implementation Steps (from docs/gtd-product-discovery.md)
- Step 3: Contexts table + CRUD
- Step 4: Next Actions (US-GTD-2a — clarify to Next Action)
- Steps 5-13: Remaining clarify sub-stories, projects, waiting for, someday/maybe, weekly review, migration

## Key Process Rules (from Task #18 retro)
- **CI wait rule**: Wait for CI green before pushing next commit, never queue multiple CI runs
- **Refactor step not optional**: Check for duplication/naming before committing
- **Glossary compliance**: New domain types must match docs/glossary.md
- **Driver handoff**: Written summary + git log + full pipeline green + "ready" message
- **Reviewer coordination**: Check others' reviews before writing, "+1" for agreement, don't re-send acknowledged reviews
- **Mini-retro after each CI build**: 1-minute checkpoint
- **Deferred items**: Track in docs/deferred-items.md, review at retros and during discovery
- **Session transcripts excluded from CI**: paths-ignore in .github/workflows/ci.yml

## Coordinator Lessons Learned
- Driver agents frequently get stuck on completed tasks or terminate without doing work — need explicit, detailed spawn prompts focused on the NEW task
- Drivers need multiple nudges to execute git operations — known persistent issue
- When respawning driver, always verify clean working tree first
- Stale shutdown requests can be picked up by new sessions — instruct agents to reject unexpected shutdowns
- Keep reviewers alive between tasks to preserve context; only rotate driver
- Always broadcast review requests explicitly — don't assume agents will self-start
- Session transcript commits were triggering CI unnecessarily — now excluded via paths-ignore
