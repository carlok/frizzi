ALTER TABLE events ADD COLUMN tiebreak_slot_id UUID;

ALTER TABLE events ADD CONSTRAINT fk_tiebreak_slot
    FOREIGN KEY (tiebreak_slot_id, id) REFERENCES event_slots(id, event_id)
    ON DELETE SET NULL (tiebreak_slot_id);
