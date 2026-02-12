CREATE TABLE contexts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
