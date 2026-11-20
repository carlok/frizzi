CREATE TABLE confirmation_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    payload JSONB,
    pdf BYTEA
);

CREATE INDEX idx_confirmation_invitations_event ON confirmation_invitations(event_id);

ALTER TABLE email_outbox ADD COLUMN invitation_id UUID
    REFERENCES confirmation_invitations(id) ON DELETE SET NULL;

CREATE INDEX idx_outbox_invitation ON email_outbox(invitation_id) WHERE invitation_id IS NOT NULL;
