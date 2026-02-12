CREATE TABLE projects (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    title TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NULL,
    dropped_at TIMESTAMPTZ NULL
);

CREATE INDEX idx_projects_user_id ON projects(user_id);

-- Add foreign key from next_actions.project_id to projects.id
ALTER TABLE next_actions
    ADD CONSTRAINT fk_next_actions_project_id
    FOREIGN KEY (project_id) REFERENCES projects(id);

CREATE INDEX idx_next_actions_project_id ON next_actions(project_id);
