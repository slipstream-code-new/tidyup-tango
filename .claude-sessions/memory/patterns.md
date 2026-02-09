# Code Patterns (from previous build, lost to uncommitted reset)

## Cargo.toml Dependencies
```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.8", features = ["postgres", "migrate", "runtime-tokio-rustls", "uuid", "chrono"] }
askama = "0.14"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "2"
anyhow = "1"
argon2 = "0.5"
rand_core = { version = "0.6", features = ["getrandom"] }
tower-sessions = "0.14"
tower-sessions-sqlx-store = { version = "0.14", features = ["postgres"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
config = "0.14"
tower-http = { version = "0.6", features = ["trace", "request-id", "fs"] }
tower = "0.5"

[dev-dependencies]
reqwest = { version = "0.12", features = ["cookies"] }
```

## Project Structure
```
src/
  main.rs              -- tokio main, tracing init, config load, app.run()
  lib.rs               -- pub mod {configuration, domain, infrastructure, routes, services, startup}
  configuration.rs     -- Settings, ApplicationSettings, DatabaseSettings, get_configuration()
  startup.rs           -- Application struct, build(), router(), run()
  routes/
    mod.rs             -- pub use handlers
    health_check.rs    -- GET /health_check -> 200
    register.rs        -- GET/POST /register with Askama template
    login.rs           -- GET/POST /login, POST /logout with session
  domain/
    mod.rs
    email.rs           -- ValidatedEmail::parse() with EmailValidationError
    password.rs        -- Password::parse() with Argon2, verify(), from_hash()
    user.rs            -- User, UserId(Uuid) newtype
  services/
    mod.rs
    registration.rs    -- register_user() -> Result<User, RegistrationError>
    authentication.rs  -- authenticate_user() -> Result<User, AuthenticationError>
  infrastructure/
    mod.rs
    user_repository.rs -- UserRepository::insert(), find_by_email()
templates/             -- at project root, not src/templates
  base.html            -- layout with skip link, landmarks, CSS link
  register.html        -- extends base, form with email+password+confirm
  login.html           -- extends base, form with email+password
tests/
  api/
    main.rs            -- mod helpers; mod health_check; mod register;
    helpers.rs          -- TestApp, spawn_app() with random DB per test
    health_check.rs
    register.rs
config/
  base.yaml            -- host, port, db settings
  local.yaml           -- gitignored overrides
migrations/
  20240101000000_create_users_table.sql
```

## Key Design Decisions
- Registration error for duplicate email uses GENERIC message (security)
- Password confirmation validation happens in route handler, not domain
- session.cycle_id() on login to prevent session fixation
- session.flush() on logout to destroy session
- Login returns 401 for invalid credentials
- Registration returns 422 for validation errors, 303 redirect on success
- Askama templates use {% extends "base.html" %} inheritance
- tower-sessions with PostgresStore for session management
- ServeDir for static files at /static/
