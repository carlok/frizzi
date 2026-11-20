CREATE TYPE event_status AS ENUM ('OPEN', 'CONFIRMED', 'CANCELLED');

CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_token VARCHAR(32) UNIQUE NOT NULL,
    admin_token VARCHAR(32) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    location VARCHAR(255) NOT NULL,
    organizer_name VARCHAR(100) NOT NULL,
    organizer_email VARCHAR(255) NOT NULL,
    organizer_locale VARCHAR(10) NOT NULL DEFAULT 'it',
    allow_multiple_choices BOOLEAN NOT NULL DEFAULT true, -- true: più giorni selezionabili, false: un solo giorno
    status event_status NOT NULL DEFAULT 'OPEN',
    confirmed_slot_id UUID,
    timezone VARCHAR(64) NOT NULL DEFAULT 'Europe/Rome',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Regola: un singolo orario/fascia oraria per ciascun giorno proposto
CREATE TABLE event_slots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    slot_date DATE NOT NULL,
    start_time TIME NOT NULL,
    end_time TIME, -- Opzionale (es. 20:00 oppure fascia 20:00 - 22:30)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (event_id, slot_date) -- Garantisce un singolo orario per giorno per lo stesso evento
);

ALTER TABLE events
    ADD CONSTRAINT fk_confirmed_slot
    FOREIGN KEY (confirmed_slot_id) REFERENCES event_slots(id) ON DELETE SET NULL;

CREATE TABLE participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    edit_token VARCHAR(32) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL, -- Obbligatoria
    locale VARCHAR(10) NOT NULL DEFAULT 'it',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (event_id, email) -- Evita doppie registrazioni con la stessa email sullo stesso evento
);

-- Preferenza puramente binaria: la presenza della riga indica "Disponibile / Presente"
CREATE TABLE votes (
    participant_id UUID NOT NULL REFERENCES participants(id) ON DELETE CASCADE,
    slot_id UUID NOT NULL REFERENCES event_slots(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (participant_id, slot_id)
);

CREATE INDEX idx_events_public_token ON events(public_token);
CREATE INDEX idx_participants_edit_token ON participants(edit_token);
