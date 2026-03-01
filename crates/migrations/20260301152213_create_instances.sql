-- Add migration script here
CREATE TABLE instances (
    id UUID PRIMARY KEY,
    definition_id UUID NOT NULL REFERENCES definitions (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE instance_fields (
    instance_id UUID NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    field_id UUID NOT NULL, -- Definition field ID
    value JSONB NOT NULL, -- Contains the field type and value
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (instance_id, field_id)
);