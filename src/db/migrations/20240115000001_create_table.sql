CREATE TYPE events_state AS ENUM ('Pending', 'Partial', 'Error', 'Processed');
CREATE TYPE destinations_state AS ENUM ('Pending', 'Error', 'Reached');

CREATE TABLE webhooks (
    id UUID PRIMARY KEY,
    secret VARCHAR(255) NOT NULL,
    name VARCHAR(256),
    description VARCHAR(256),
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE events (
    id UUID PRIMARY KEY,
    webhook_id UUID NOT NULL,
    data JSONB NOT NULL,
    state events_state NOT NULL DEFAULT 'Pending',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    CONSTRAINT events_webhook_fk FOREIGN KEY (webhook_id)
        REFERENCES webhooks(id) ON DELETE CASCADE
);

CREATE TABLE destinations (
    id UUID PRIMARY KEY,
    webhook_id UUID NOT NULL,
    url VARCHAR(255) NOT NULL,
    api_key VARCHAR(255),
    state destinations_state NOT NULL DEFAULT 'Pending',
    retry_count INT NOT NULL DEFAULT 0,
    next_retry_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    CONSTRAINT destinations_webhook_fk FOREIGN KEY (webhook_id)
        REFERENCES webhooks(id) ON DELETE CASCADE
);