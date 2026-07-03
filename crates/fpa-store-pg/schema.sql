-- Agent-owned Project persistence (p6-c001). The Project aggregate is stored WHOLE
-- as a JSONB `body`; `id`/`name`/`schema_version` are denormalized for indexing and
-- cheap listing. This table is owned by the agent, not by forge or fabric.
CREATE TABLE IF NOT EXISTS fpa_projects (
    id             uuid PRIMARY KEY,
    name           text NOT NULL,
    schema_version integer NOT NULL,
    body           jsonb NOT NULL,
    updated_at     timestamptz NOT NULL DEFAULT now()
);
