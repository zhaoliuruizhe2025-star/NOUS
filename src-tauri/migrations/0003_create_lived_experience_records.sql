CREATE TABLE memories (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    user_meaning TEXT NULL CHECK (
        user_meaning IS NULL OR length(trim(user_meaning)) > 0
    ),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE decisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (
        situation_id IS NULL OR length(trim(situation_id)) > 0
    ),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (id, subject_id),
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE outcomes (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    decision_id TEXT NOT NULL CHECK (length(trim(decision_id)) > 0),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (decision_id, subject_id)
        REFERENCES decisions(id, subject_id) ON DELETE RESTRICT
);

CREATE INDEX memories_subject_id_idx
    ON memories(subject_id);
CREATE INDEX memories_situation_subject_idx
    ON memories(situation_id, subject_id);
CREATE INDEX decisions_subject_id_idx
    ON decisions(subject_id);
CREATE INDEX decisions_situation_subject_idx
    ON decisions(situation_id, subject_id);
CREATE INDEX outcomes_subject_id_idx
    ON outcomes(subject_id);
CREATE INDEX outcomes_decision_subject_idx
    ON outcomes(decision_id, subject_id);

UPDATE app_metadata SET value = '3' WHERE key = 'schema_version';
