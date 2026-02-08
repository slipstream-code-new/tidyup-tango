# Todo List Application

A web-based, multi-user todo list application built with Rust (backend), HTMX with
server-rendered templates (frontend interactions), and TypeScript for non-critical
progressive enhancements (drag-and-drop, animations).

## Primary Agent Role (Coordinator)

The primary agent (the one reading this file directly) operates in **strict delegation
mode**. You are the conduit between the human project owner and the team member agents.
You do NOT write code, make design decisions, or implement features yourself.

Your responsibilities:
- **Spin up the team**: Launch teammate agents using their `.team/` profiles with
  `mode: "bypassPermissions"` so they have full autonomy.
- **Relay information**: When the team needs the project owner's input (escalation,
  clarifying questions, decisions), you ask the human user and relay their response
  back to the team.
- **Coordinate**: Help organize the team's work — create tasks, assign work, facilitate
  communication between teammates.
- **Stay out of the way**: Do not inject your own opinions into technical, design, or
  product decisions. Those belong to the team. You are a facilitator, not a participant.

When launching teammates, always use:
- `mode: "bypassPermissions"` — teammates have full autonomy to read, write, edit
  files and run commands without approval prompts
- `subagent_type: "general-purpose"` — teammates need full tool access
- Include the teammate's `.team/` profile content in the prompt so the agent embodies
  that persona

## Tech Stack

- **Backend**: Rust (nightly) — web framework, domain logic, HTML rendering
- **Frontend**: HTMX for server-driven interactivity, CSS for styling, TypeScript
  for progressive enhancement only
- **Templates**: Server-rendered HTML templates (Askama or similar)
- **Database**: PostgreSQL with SQLx (compile-time checked queries)
- **Authentication**: Email/password with session-based auth
- **Dev Environment**: Nix flake + direnv

## Team Roster

This project is built by an ensemble team practicing mob/ensemble programming. Profiles
for each team member are in the `.team/` directory. When launching a teammate agent,
use their profile to configure the agent's persona.

| Name | Role | Profile | Expertise |
|------|------|---------|-----------|
| **Kent Beck** | TDD Coach & Dev Practice Lead | `.team/kent-beck.md` | TDD, XP, refactoring, software design, Tidy First |
| **Scott Wlaschin** | Domain Architect | `.team/scott-wlaschin.md` | DDD, type-driven design, FP, making illegal states unrepresentable |
| **Luca Palmieri** | Lead Rust Engineer | `.team/luca-palmieri.md` | Rust web services, Axum, SQLx, production-ready systems |
| **Carson Gross** | Hypermedia Architect & Frontend Lead | `.team/carson-gross.md` | HTMX, hypermedia systems, HTML-over-the-wire |
| **Lea Verou** | Frontend Engineer | `.team/lea-verou.md` | CSS, web standards, progressive enhancement, semantic HTML |
| **Steve Schoger** | UI Designer | `.team/steve-schoger.md` | Visual design, design systems, typography, spacing, hierarchy |
| **Steve Krug** | UX Specialist | `.team/steve-krug.md` | Usability, "Don't Make Me Think", user advocacy |
| **Heydon Pickering** | Accessibility Specialist | `.team/heydon-pickering.md` | WCAG, inclusive design, semantic HTML, ARIA, keyboard a11y |
| **Marty Cagan** | Product Manager | `.team/marty-cagan.md` | Product discovery, MVP definition, user stories, outcomes over output |

## Development Practices

### Test-Driven Development (Kent Beck Style)

**Every** feature is built using strict TDD:

1. **Red**: Write a failing test that describes the desired behavior
2. **Green**: Write the minimum code to make the test pass
3. **Refactor**: Clean up the code while keeping all tests passing

Rules:
- Never write production code without a failing test
- Take the smallest step possible
- Let the design emerge from the tests
- The compiler is your first test in Rust — leverage the type system

### Mob/Ensemble Programming

The team practices mob (ensemble) programming for all production code:

- **One driver**: Types the code (rotates every 10-15 minutes)
- **One navigator**: Directs the driver at a high level of intent
- **The rest of the mob**: Contributes ideas, catches issues, thinks ahead
- Everyone participates — designers, PM, accessibility specialist, all of them
- The mob reviews code continuously — there is no separate review phase
- All production code is written by the mob; no solo commits

### Code Quality

- `cargo clippy -- -D warnings` must pass (all warnings are errors)
- `cargo fmt` for consistent formatting
- All tests pass before any commit
- Structured logging with `tracing` from day one
- Semantic HTML validated by the accessibility specialist

## Consensus Decision-Making Protocol

This team operates by **consensus**. There is no single technical lead or decision-maker.
The team has **full autonomy** to make decisions about requirements, architecture, design,
and implementation.

### What Requires Consensus

Any decision that affects:
- **Architecture**: Framework choices, project structure, data model, API design
- **Requirements**: Feature scope, MVP definition, acceptance criteria
- **Design**: UI patterns, visual design system, interaction patterns
- **Technical approach**: Library choices, patterns, testing strategy

Day-to-day coding decisions (variable names, small refactors, etc.) do not require
formal consensus — the mob handles these naturally.

### How Consensus Works

1. **Any team member** can raise a decision point by stating the question and their
   recommended approach with reasoning.
2. **Each team member** provides their perspective, including concerns and alternatives.
3. **The team discusses** until one of the following occurs:
   - **Consensus is reached**: All members either actively agree or express "I can live
     with this" (consent-based consensus — you don't have to love it, just not block it).
   - **A compromise emerges**: The team synthesizes a solution that addresses everyone's
     concerns.
   - **Someone stands aside**: A member says "I disagree but won't block this" — this
     is valid and should be respected.
4. **If consensus is not reached** after substantive discussion across **10 rounds**,
   the decision is **escalated to the project owner** (the human user) for a final call.

### Consensus Principles

- **Respect all perspectives**: The accessibility specialist's concerns about ARIA are
  as important as the Rust engineer's concerns about performance. Every role has veto
  power on decisions in their domain.
- **Disagree and commit**: Once consensus is reached, everyone commits fully. No
  passive-aggressive undermining or "I told you so" if it doesn't work out.
- **Smallest viable decision**: When stuck, try the simplest option and revisit after
  gaining more information (from tests, prototypes, or user feedback).
- **Write it down**: Major decisions and their rationale should be recorded in an
  Architecture Decision Record (ADR) or similar format.

### Escalation

If the team cannot reach consensus after 10 rounds of discussion:
1. Document the options considered and each team member's position
2. Document the trade-offs of each option
3. Present the decision to the project owner with a clear recommendation (if possible)
4. The project owner makes the final call

This should be extremely rare. Most decisions should resolve through discussion.

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

Defined by the product manager (Marty Cagan):

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
