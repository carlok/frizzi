ALTER TABLE event_slots ADD CONSTRAINT valid_time_range CHECK (end_time IS NULL OR end_time > start_time);
ALTER TABLE event_slots ADD CONSTRAINT slot_event_pair UNIQUE (id, event_id);
ALTER TABLE events DROP CONSTRAINT fk_confirmed_slot;
ALTER TABLE events ADD CONSTRAINT fk_confirmed_slot FOREIGN KEY (confirmed_slot_id, id)
    REFERENCES event_slots (id, event_id) ON DELETE SET NULL (confirmed_slot_id);
ALTER TABLE events ADD CONSTRAINT confirmed_status CHECK ((status = 'CONFIRMED') = (confirmed_slot_id IS NOT NULL));

-- Delivery is committed with the event/vote, then attempted after commit and by the retry worker.
CREATE TABLE email_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient VARCHAR(255) NOT NULL,
    subject TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    locked_until TIMESTAMPTZ,
    attempts INTEGER NOT NULL DEFAULT 0,
    sent_at TIMESTAMPTZ
);
CREATE INDEX idx_outbox_pending ON email_outbox (available_at) WHERE sent_at IS NULL;
CREATE TABLE request_limits (
    key TEXT NOT NULL,
    hour TIMESTAMPTZ NOT NULL DEFAULT date_trunc('hour', now()),
    count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (key, hour)
);
