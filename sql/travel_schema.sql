CREATE TABLE IF NOT EXISTS travel_likes (
  slug         VARCHAR(255) PRIMARY KEY,
  like_count   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS travel_comments (
  id           SERIAL PRIMARY KEY,
  slug         VARCHAR(255) NOT NULL,
  author       VARCHAR(50) NOT NULL,
  password     VARCHAR(255) NOT NULL,
  content      TEXT NOT NULL,
  created_at   TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_travel_comments_slug ON travel_comments(slug);

CREATE TABLE IF NOT EXISTS travel_reactions (
  id           SERIAL PRIMARY KEY,
  comment_id   INTEGER NOT NULL REFERENCES travel_comments(id) ON DELETE CASCADE,
  emoji        VARCHAR(10) NOT NULL,
  count        INTEGER NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_travel_reactions_unique
  ON travel_reactions(comment_id, emoji);
