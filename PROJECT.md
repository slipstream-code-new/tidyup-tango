# Todo List Application

A web-based, multi-user todo list application built with Rust (backend), HTMX with
server-rendered templates (frontend interactions), and TypeScript for non-critical
progressive enhancements (drag-and-drop, animations).

> **This document contains project owner constraints.** The team must follow these rules.
> Changes to this document require project owner approval. For team-owned practices and
> conventions, see `TEAM_AGREEMENTS.md`.

## Tech Stack

- **Backend**: Rust (nightly) — web framework, domain logic, HTML rendering
- **Frontend**: HTMX for server-driven interactivity, CSS for styling, TypeScript
  for progressive enhancement only
- **Templates**: Server-rendered HTML templates (Askama or similar)
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Authentication**: Email/password with session-based auth
- **Dev Environment**: Nix flake + direnv

## Development Mandates

These are non-negotiable practices for the project:

- **Test-Driven Development**: Every feature is built using strict TDD
  (Red-Green-Refactor). No production code without a failing test.
- **Mob/Ensemble Programming**: All production code is written by the mob. No solo
  commits to production code.
- **Consensus Decision-Making**: The team operates by consensus. No single technical
  lead or decision-maker. If consensus is not reached after 10 rounds of substantive
  discussion, the decision is escalated to the project owner for a final call.
- **Code Quality Gates**: `cargo clippy -- -D warnings`, `cargo fmt`, and all tests
  must pass before any commit. Structured logging with `tracing` from day one.

## Environment & Tooling

### Nix

This project runs on NixOS with a `flake.nix` already in place. When the team needs
a utility that isn't currently installed:
- Use `nix shell nixpkgs#<package> --command <cmd>` for one-off commands
- For tools used regularly, add them to `buildInputs` in the devshell in `flake.nix`

### Agent Skills

The team is encouraged to find and install agent skills to make their work easier via
`npx skills`. The `find-skill` skill is already installed. When adding skills:
- Add them for **this project only** (not globally)
- Use support for **generic agent locations** (Codex, OpenCode, etc.) **and** Claude Code

### Custom Skills

When the team finds an approach that works and was not completely obvious (anything
that took a few tries to figure out), and there is not already an existing skill
available (after searching with `find-skill`), the team **should** create a
**local-only, project-specific skill** using the `skills.sh` format. This ensures
hard-won knowledge is captured and reusable.

### External Services (Docker)

If the team needs any external services running (e.g., a PostgreSQL database server),
use a `docker-compose.yml` file and Docker to run them. Do **not** run services
directly on the host machine. This keeps the dev environment reproducible and
self-contained.

## MVP Scope

### Must Have
- User registration (email/password)
- User login/logout
- View personal todo list
- Add a todo item
- Complete a todo item
- Delete a todo item

### Should Have
- Edit a todo item title

### Could Have
- Reorder todo items (drag-and-drop with keyboard alternative)

### Out of Scope (Future)
- Multiple lists / categories
- Due dates and reminders
- Sharing / collaboration
- Tags or labels
- Search and filter
- Mobile app (responsive web is sufficient)
- Social login (OAuth)
