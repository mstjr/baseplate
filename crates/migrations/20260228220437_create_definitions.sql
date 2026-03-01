-- Add migration script here
CREATE TABLE definitions (
    id UUID PRIMARY KEY,
    api_name TEXT UNIQUE NOT NULL,
    singular_name TEXT NOT NULL,
    plural_name TEXT NOT NULL,
    description TEXT,
    title_field UUID NOT NULL,
    quick_view_fields UUID [] NOT NULL DEFAULT '{}',
    fields JSONB NOT NULL,
    hidden BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);