# Todo List

A multi-user todo list web application built with Rust and HTMX. Users register with email/password, log in, and manage their personal todo list — adding, completing, editing, and deleting items. The UI is server-rendered with progressive enhancement via HTMX for seamless interactions without full page reloads.

## Tech Stack

- **Backend:** Rust (nightly) with [Axum](https://github.com/tokio-rs/axum) 0.8
- **Database:** PostgreSQL 16 with [SQLx](https://github.com/launchbadge/sqlx) 0.8 (compile-time checked queries)
- **Templates:** [Askama](https://github.com/djc/askama) 0.13 (type-safe Jinja2-like templates)
- **Frontend:** [HTMX](https://htmx.org/) + vanilla CSS + TypeScript for progressive enhancement
- **Auth:** Session-based with Argon2 password hashing, [tower-sessions](https://github.com/maxcountryman/tower-sessions)
- **Static files:** [tower-http](https://github.com/tower-rs/tower-http) ServeDir

## Prerequisites

This project uses [Nix flakes](https://nixos.org/) with [direnv](https://direnv.net/) for a reproducible dev environment. If you have both installed, all dependencies are handled automatically.

**Without Nix**, you will need:
- Rust nightly (see `rust-toolchain.toml` for the exact channel)
- PostgreSQL 16
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli) (`cargo install sqlx-cli --features postgres`)
- Docker (for running PostgreSQL via docker-compose)

## Getting Started

1. **Clone the repository:**

   ```sh
   git clone <repo-url>
   cd todo_list
   ```

2. **Set up the dev environment:**

   ```sh
   # With Nix + direnv (recommended):
   direnv allow

   # Or manually:
   nix develop
   ```

3. **Start PostgreSQL:**

   ```sh
   docker-compose up -d
   ```

4. **Create a `.env` file** in the project root:

   ```
   DATABASE_URL=postgres://todo_list:todo_list@127.0.0.1:5432/todo_list
   ```

5. **Run database migrations:**

   ```sh
   sqlx migrate run
   ```

6. **Start the application:**

   ```sh
   cargo run
   ```

   The app will be available at [http://127.0.0.1:8080](http://127.0.0.1:8080).

## Git Hooks Setup

This project uses [pre-commit](https://pre-commit.com/) to manage git hooks. After entering the dev environment:

```sh
pre-commit install
```

This installs a pre-commit hook that automatically:
- Stages Claude Code session transcripts (`.claude-sessions/`)
- Generates human-readable markdown chat logs from JSONL transcripts
- Stages the generated chat logs so they're included in every commit

The chat logs are written to `.claude-sessions/chat-logs/` and provide a readable history of AI-assisted development sessions.

## Running Tests

Tests require PostgreSQL to be running (step 3 above).

```sh
cargo test
```

## Quality Gates

```sh
cargo fmt --check    # Formatting
cargo clippy -- -D warnings  # Linting
cargo test           # Tests
```

## Project Structure

```
todo_list/
├── src/
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library root
│   ├── startup.rs         # App startup and server configuration
│   ├── configuration.rs   # Config loading
│   ├── domain/            # Domain types (User, TodoItem, etc.)
│   ├── routes/            # HTTP route handlers
│   ├── services/          # Business logic services
│   └── infrastructure/    # Database and external integrations
├── templates/             # Askama HTML templates
├── static/
│   ├── css/               # Stylesheets
│   └── js/                # TypeScript/JavaScript
├── migrations/            # SQLx database migrations
├── tests/
│   └── api/               # Integration tests
├── config/                # Application configuration files
├── docker-compose.yml     # PostgreSQL for local development
└── flake.nix              # Nix flake for dev environment
```

## License

[MIT](LICENSE)
