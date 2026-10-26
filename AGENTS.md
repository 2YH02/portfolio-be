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
├── migrations.rs    # Transactional startup migrations and schema compatibility checks
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
├── migrations.rs    # Isolated database migration safety tests (require CREATEDB)
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

- `src/main.rs` enables CORS for the why2log local frontend origin: `http://localhost:5777` and permits the guestbook client header `X-Guestbook-Token`.
- Production why2log origins are allowed by default: `https://www.why2log.com`, `https://why2log.com`
- Additional or replacement origins can be configured with comma-separated `CORS_ALLOWED_ORIGINS`
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
| `PG_HOST_PORT` | Docker Compose host port for Postgres; container connections remain on 5432 | `5432` |
| `PG__USER` | Postgres user | — |
| `PG__PASSWORD` | Postgres password | — |
| `PG__DBNAME` | Postgres database name | — |
| `PG__POOL__MAX_SIZE` | Connection pool size | — |
| `JWT_SECRET` | JWT signing secret for admin cookie auth | `change-me-in-production` |
| `COOKIE_SECURE` | Mark admin auth cookie as Secure and SameSite=None | `false` |
| `GUESTBOOK_MODERATION_RULES` | Private JSON keyword lists; overrides the rules file | empty |
| `GUESTBOOK_MODERATION_RULES_FILE` | Local keyword JSON path, resolved from working directory | `moderation-rules.json` |
| `CORS_ALLOWED_ORIGINS` | Comma-separated allowed CORS origins | `http://localhost:5777,https://www.why2log.com,https://why2log.com` |

Copy `.env.example` (if present) or create `.env` with the above for local dev.

---

## Running Locally

If another local Postgres instance already uses port 5432, set `PG_HOST_PORT=5433` in `.env` before starting Docker Compose. The containerized API continues to use `db:5432`; when running `cargo run` on the host, set `PG__PORT` to the selected host port as well.

```bash
# Start DB only
docker compose up db

# Run server
RUST_LOG=debug cargo run

# Run tests (requires running DB)
cargo test
```

The travel schema is mounted as `sql/travel_schema.sql`. Docker only runs init scripts when the Postgres volume is first created; run `docker compose down -v` before `docker compose up db` if a local DB volume was created before the travel tables existed.

## Guestbook API

The shared anonymous guestbook is implemented in `src/guestbook/` and uses `sql/guestbook_schema.sql`.

- `GET /guestbook/{scope}` lists approved entries for a reusable scope such as `castmorrow`.
- `POST /guestbook/{scope}` accepts `nickname`, `message`, and an optional honeypot `website` field. Content is normalized and classified as visible, pending, or blocked; a hashed client token and database rate limit prevent rapid repeat submissions.
- Messages containing links are placed in pending review before keyword normalization to avoid false positives from joined URL components; they are never published automatically.
- `GET /admin/guestbook?status=pending` and `POST /admin/guestbook/{id}/moderate` are admin-only moderation endpoints. `DELETE /admin/guestbook/{id}` soft-deletes an entry.

The server applies pending database migrations before listening for requests. Add every client origin that uses the API to `CORS_ALLOWED_ORIGINS`; environment values override the local default.

## Database migrations

- `src/migrations.rs` contains the ordered migration registry. Version 1 embeds `sql/guestbook_schema.sql`; this SQL must remain immutable after deployment. Add a new SQL file and a higher version for subsequent changes.
- `public.portfolio_schema_migrations` records version, name, SHA-256 checksum, and application time. Changed known versions, unknown historical versions, and missing earlier versions stop startup. Newer versions are preserved with a warning to permit older applications to restart after backward-compatible additive migrations; this does not prove arbitrary future SQL is compatible.
- All pending migrations and history writes run in one transaction under a PostgreSQL transaction advisory lock. Failure rolls back the batch and prevents the new server from listening. Multiple replicas serialize migration execution.
- Transaction-local `lock_timeout = 5s`, `statement_timeout = 60s`, and `search_path = public, pg_catalog` bound DB lock/statement waits and fix the schema used for DDL. The statement timeout is per SQL statement, not a whole-batch or network connection deadline. Transaction settings do not leak into pooled API connections.
- Existing guestbook tables can be adopted by version 1 through `IF NOT EXISTS`. Required column names, types, lengths, and nullability are checked before commit and on subsequent startup; incompatible columns fail without recording a successful migration. Additional columns are allowed. This check does not repair tables or verify every default, index, or constraint.
- `cargo run -- --migrate-only` applies migrations without starting HTTP or loading moderation rules. The normal Docker command also runs migrations at startup, using the existing Railway `PG__*` settings; no production DB credentials are needed in the GitHub runner.
- Never register `sql/schema.sql` as an operational migration: it drops the public schema and contains development seed data. Compose initialization remains local/test-only.
- Use additive schema changes compatible with the previous running application. Application rollback does not undo committed database migrations.
- `tests/migrations.rs` creates uniquely named temporary databases using a dedicated local/CI role with `CREATEDB`, and removes those databases after successful tests. It never resets the configured database. Failed tests can leave their test databases behind in the disposable PostgreSQL instance. Tests cover a truly fresh database, repeat runs with preserved content, failed-DDL rollback and retry, concurrent runners, newer additive history, checksum mismatch, missing history, and lock timeout. Do not point the test suite at production.

Moderation keywords are private configuration, never Rust constants or test fixtures. The ignored local `moderation-rules.json` contains `profanity`, `sexual`, and `politics` string arrays; see `moderation-rules.example.json` for harmless sample values. Existing local keywords have been preserved in the ignored file. For a fresh checkout, copy the example and replace the samples with private rules.

Rules load once at startup: non-empty `GUESTBOOK_MODERATION_RULES` takes priority over `GUESTBOOK_MODERATION_RULES_FILE`. Railway should receive the JSON through its environment variable; Docker Compose forwards that variable too. The local rules file is excluded from both Git and Docker build context. Restart after edits. Missing files or empty rule sets send submissions to pending review (`rules_unavailable`); malformed JSON, unknown categories, or keywords that normalize to empty fail startup without logging rule contents. Unit tests use synthetic keywords and do not require private configuration. Previously committed keyword lists remain in Git history.

---

## Testing

- `tests/guestbook.rs` uses a disposable database (requires `CREATEDB`) to verify POST persistence, pending visibility, HTTP 429, and cooldown expiry. The cooldown SQL explicitly binds seconds as `bigint` to match Rust `i64`.

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
