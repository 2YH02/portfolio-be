CREATE TABLE IF NOT EXISTS public.project_likes (
    project_id INTEGER NOT NULL,
    client_hash CHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (project_id, client_hash)
);
