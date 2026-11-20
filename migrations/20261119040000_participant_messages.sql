CREATE TABLE participant_messages (
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    request_id UUID NOT NULL,
    recipients INTEGER NOT NULL,
    PRIMARY KEY (event_id, request_id)
);
