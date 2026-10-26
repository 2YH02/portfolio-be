CREATE TABLE IF NOT EXISTS guestbook_entries (
  id                SERIAL PRIMARY KEY,
  scope             VARCHAR(64) NOT NULL,
  nickname          VARCHAR(16) NOT NULL,
  message           VARCHAR(120) NOT NULL,
  status            VARCHAR(16) NOT NULL DEFAULT 'visible',
  moderation_reason VARCHAR(64),
  client_hash       CHAR(64) NOT NULL,
  created_at        TIMESTAMP NOT NULL DEFAULT NOW(),
  CONSTRAINT guestbook_entries_status_check CHECK (status IN ('pending', 'visible', 'blocked', 'deleted'))
);

CREATE INDEX IF NOT EXISTS idx_guestbook_entries_scope_status_created
  ON guestbook_entries(scope, status, created_at DESC, id DESC);

CREATE TABLE IF NOT EXISTS guestbook_rate_limits (
  client_hash       CHAR(64) PRIMARY KEY,
  last_submitted_at TIMESTAMP NOT NULL DEFAULT NOW()
);
