DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;

CREATE TABLE public.posts (
  id               SERIAL           PRIMARY KEY,
  title            VARCHAR(255)     NOT NULL,
  description      TEXT             NOT NULL DEFAULT '',
  body             TEXT             NOT NULL,
  tags             TEXT[]           NOT NULL DEFAULT '{}',
  thumbnail        TEXT             NOT NULL DEFAULT '',
  created_at       TIMESTAMP        NOT NULL DEFAULT NOW()
);
