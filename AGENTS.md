# AGENTS.md

Guide for AI agents working on this repository.

> Keep this file up to date. If you add a new module, endpoint, environment variable, or change a convention, update the relevant section in this file as part of the same task.

## Project Overview

Rust-based REST API backend for a personal portfolio blog.
Deployed on Railway, backed by PostgreSQL.

- **Framework**: actix-web 4
- **Database**: PostgreSQL 14 via `tokio-postgres` + `deadpool-postgres`
- **Config**: `confik` (env-based) + `dotenvy` (.env file)
- **Logging**: `env_logger` + `tracing`

---

## Directory Structure

```
src/
├── main.rs          # Server bootstrap, middleware, route mounting
├── lib.rs           # Module re-exports
├── config.rs        # AppConfig struct (env-bound)
├── db.rs            # Connection pool factory
├── errors.rs        # ServiceError enum + ResponseError impl
├── user/
│   ├── model.rs     # User, Role structs + auth parsing logic
│   ├── dto.rs       # MeRequest
│   ├── handlers.rs  # GET /me, POST /auth
│   └── routes.rs    # Route registration
└── blog/
    ├── model.rs     # Post struct (PostgresMapper)
    ├── dto.rs       # CreatePost, UpdatePost, PostListResponse, BlurRequest/Response
    ├── service.rs   # All DB queries + image blur logic
    ├── handlers.rs  # HTTP handlers for blog CRUD
    └── routes.rs    # Route registration

src/travel/
├── model.rs     # TravelComment, TravelReaction structs
├── dto.rs       # travel like/comment/reaction request/response shapes
├── service.rs   # travel DB queries + bcrypt password checks
├── handlers.rs  # HTTP handlers + cookie duplicate prevention
└── routes.rs    # Route registration

sql/
├── schema.sql        # Table definitions + seed data
└── travel_schema.sql # Travel likes/comments/reactions tables

tests/
├── blog.rs          # Blog integration tests (require live DB)
└── travel.rs        # Travel like/comment/reaction integration tests
```

---

## Module Conventions

Every domain (e.g. `blog`, `user`) follows this fixed structure:

| File | Responsibility |
|------|----------------|
| `model.rs` | DB row structs with `#[derive(PostgresMapper)]` |
| `dto.rs` | Request/response shapes (Serialize/Deserialize) |
| `service.rs` | All database operations and business logic |
| `handlers.rs` | Actix handler functions, auth checks, calls service |
| `routes.rs` | `pub fn init(cfg: &mut web::ServiceConfig)` wiring |

Do not mix these responsibilities. Handlers must not contain SQL. Service functions must not parse HTTP.

---

## Error Handling

All errors must go through `ServiceError` in `src/errors.rs`:

```rust
pub enum ServiceError {
    BadRequest(String),
    Unauthorized,
    NotFound,
    InternalServerError(String),
}
```

- Map DB errors to `ServiceError::InternalServerError(e.to_string())`
- Map missing rows to `ServiceError::NotFound`
- Use `.map_err(|e| ServiceError::InternalServerError(e.to_string()))?` — never `.unwrap()` in service functions
- Handlers return `e.error_response()` from the `ResponseError` trait

---

## Database Pattern

- Use `pool.get().await` to acquire a client from the pool
- Use `client.prepare(...)` + `client.query_one` / `client.query` / `client.execute`
- All queries use positional parameters (`$1`, `$2`, ...) — never string interpolation
- Row mapping: `Post::from_row_ref(&row).unwrap()` is currently used but should be replaced with proper error propagation when touching service functions

---

## Authentication

- Admin login uses `POST /auth` with `admin_user` / `admin_pass` from `AppConfig`.
- Successful admin login sets the HTTP-only `admin_token` cookie containing a JWT.
- JWT parsing is handled by `User::from_jwt(token, jwt_secret)` in `user/model.rs`.
- Admin-only handlers use the `Admin` extractor from `user/handlers.rs`.
- `travel/` endpoints are anonymous. Use cookies only for duplicate prevention.
- Travel comment deletion verifies the bcrypt hash stored in `travel_comments.password`.

---

## Logging Rules

- Use `tracing::info!`, `tracing::debug!`, `tracing::warn!` — never `println!`
- Never log credential values (`admin_user`, `admin_pass`, raw auth headers, passwords)
- Log outcomes, not secrets: `tracing::debug!("auth result: role={:?}", user.role)`
- Log level is controlled by the `RUST_LOG` env var (default: `info`)

---

## CORS

- `src/main.rs` enables CORS for the why2log local frontend origin: `http://localhost:5777`
- Credentials are supported so browser requests can send duplicate-prevention cookies
- why2log `fetch` calls that need cookies must use `credentials: "include"`

---

## Commit Message Convention

Follow Conventional Commits in Korean:

```
<type>: <Korean description>
```

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`
Scope is optional: `chore(ci): ...`

Examples from history:
```
fix: user 핸들러 println! 제거 및 크리덴셜 로그 노출 수정
chore(ci): 테스트 로그 출력 간소화
feat: 서버 에러 메시지 출력 추가
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_ADDR` | Bind address | `0.0.0.0:8080` |
| `ADMIN_USER` | Admin username | `guest` |
| `ADMIN_PASS` | Admin password | `secret` |
| `PG__HOST` | Postgres host | — |
| `PG__PORT` | Postgres port | — |
| `PG__USER` | Postgres user | — |
| `PG__PASSWORD` | Postgres password | — |
| `PG__DBNAME` | Postgres database name | — |
| `PG__POOL__MAX_SIZE` | Connection pool size | — |
| `JWT_SECRET` | JWT signing secret for admin cookie auth | `change-me-in-production` |
| `COOKIE_SECURE` | Mark admin auth cookie as Secure and SameSite=None | `false` |

Copy `.env.example` (if present) or create `.env` with the above for local dev.

---

## Running Locally

```bash
# Start DB only
docker compose up db

# Run server
RUST_LOG=debug cargo run

# Run tests (requires running DB)
cargo test
```

The travel schema is mounted as `sql/travel_schema.sql`. Docker only runs init scripts when the Postgres volume is first created; run `docker compose down -v` before `docker compose up db` if a local DB volume was created before the travel tables existed.

---

## Testing

- Tests live in `tests/` and are integration tests — they require a real DB connection
- Load env with `dotenv().ok()` at the top of each test
- Build the app with `test::init_service(...)` and call with `test::TestRequest`
- Assert both status code and deserialized response body
- Test file names mirror the module they test (e.g. `tests/blog.rs`)

---

## CI/CD

- Push to `main` → runs `cargo test` against a Docker Compose PostgreSQL → deploys to Railway
- All DB credentials and `ADMIN_USER`/`ADMIN_PASS` are stored as GitHub Actions secrets
- Deploy step uses `railway up --service portfolio-be`
