CREATE TABLE next_actions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    context_id UUID NOT NULL REFERENCES contexts(id),
    project_id UUID NULL,
    title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NULL
);

CREATE INDEX idx_next_actions_user_id ON next_actions(user_id);
CREATE INDEX idx_next_actions_context_id ON next_actions(context_id);
