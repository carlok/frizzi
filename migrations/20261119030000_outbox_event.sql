ALTER TABLE email_outbox ADD COLUMN event_id UUID REFERENCES events(id) ON DELETE CASCADE;

-- Associate pending messages from versions that only stored the private link.
UPDATE email_outbox q
SET event_id = e.id
FROM events e
WHERE q.sent_at IS NULL
  AND e.public_token = substring(q.body FROM '\n\nhttps?://[^\r\n]*/([A-Za-z0-9_-]{21})/(?:admin/)?[A-Za-z0-9_-]{21}\r?\n');

CREATE INDEX idx_outbox_event ON email_outbox(event_id) WHERE event_id IS NOT NULL;
