CREATE TABLE situation_corrections (
    situation_id TEXT NOT NULL CHECK (length(trim(situation_id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    correction_sequence INTEGER NOT NULL CHECK (correction_sequence > 0),
    before_description TEXT NOT NULL CHECK (length(trim(before_description)) > 0),
    after_description TEXT NOT NULL CHECK (length(trim(after_description)) > 0),
    before_state_token TEXT NOT NULL CHECK (length(trim(before_state_token)) > 0),
    after_state_token TEXT NOT NULL CHECK (length(trim(after_state_token)) > 0),
    user_note TEXT NULL CHECK (
        user_note IS NULL OR length(
            trim(user_note, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')
        ) > 0
    ),
    recorded_at_ms INTEGER NOT NULL CHECK (recorded_at_ms >= 0),
    PRIMARY KEY (situation_id, correction_sequence),
    UNIQUE (before_state_token),
    UNIQUE (after_state_token),
    CHECK (before_state_token <> after_state_token),
    CHECK (before_description <> after_description),
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE observation_corrections (
    observation_id TEXT NOT NULL CHECK (length(trim(observation_id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    correction_sequence INTEGER NOT NULL CHECK (correction_sequence > 0),
    before_content TEXT NOT NULL CHECK (length(trim(before_content)) > 0),
    after_content TEXT NOT NULL CHECK (length(trim(after_content)) > 0),
    before_situation_id TEXT NULL CHECK (
        before_situation_id IS NULL OR length(trim(before_situation_id)) > 0
    ),
    after_situation_id TEXT NULL CHECK (
        after_situation_id IS NULL OR length(trim(after_situation_id)) > 0
    ),
    before_state_token TEXT NOT NULL CHECK (length(trim(before_state_token)) > 0),
    after_state_token TEXT NOT NULL CHECK (length(trim(after_state_token)) > 0),
    user_note TEXT NULL CHECK (
        user_note IS NULL OR length(
            trim(user_note, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')
        ) > 0
    ),
    recorded_at_ms INTEGER NOT NULL CHECK (recorded_at_ms >= 0),
    PRIMARY KEY (observation_id, correction_sequence),
    UNIQUE (before_state_token),
    UNIQUE (after_state_token),
    CHECK (before_state_token <> after_state_token),
    CHECK (
        before_content <> after_content
        OR before_situation_id IS NOT after_situation_id
    ),
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (observation_id, subject_id)
        REFERENCES observations(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (before_situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (after_situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE thought_corrections (
    thought_id TEXT NOT NULL CHECK (length(trim(thought_id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    correction_sequence INTEGER NOT NULL CHECK (correction_sequence > 0),
    before_content TEXT NOT NULL CHECK (length(trim(before_content)) > 0),
    after_content TEXT NOT NULL CHECK (length(trim(after_content)) > 0),
    before_situation_id TEXT NULL CHECK (
        before_situation_id IS NULL OR length(trim(before_situation_id)) > 0
    ),
    after_situation_id TEXT NULL CHECK (
        after_situation_id IS NULL OR length(trim(after_situation_id)) > 0
    ),
    before_confidence INTEGER NULL CHECK (
        before_confidence IS NULL OR before_confidence BETWEEN 0 AND 100
    ),
    after_confidence INTEGER NULL CHECK (
        after_confidence IS NULL OR after_confidence BETWEEN 0 AND 100
    ),
    before_state_token TEXT NOT NULL CHECK (length(trim(before_state_token)) > 0),
    after_state_token TEXT NOT NULL CHECK (length(trim(after_state_token)) > 0),
    user_note TEXT NULL CHECK (
        user_note IS NULL OR length(
            trim(user_note, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')
        ) > 0
    ),
    recorded_at_ms INTEGER NOT NULL CHECK (recorded_at_ms >= 0),
    PRIMARY KEY (thought_id, correction_sequence),
    UNIQUE (before_state_token),
    UNIQUE (after_state_token),
    CHECK (before_state_token <> after_state_token),
    CHECK (
        before_content <> after_content
        OR before_situation_id IS NOT after_situation_id
        OR before_confidence IS NOT after_confidence
    ),
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (thought_id, subject_id)
        REFERENCES thoughts(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (before_situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (after_situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE INDEX situation_corrections_subject_idx
    ON situation_corrections(subject_id);
CREATE INDEX observation_corrections_subject_idx
    ON observation_corrections(subject_id);
CREATE INDEX observation_corrections_before_situation_subject_idx
    ON observation_corrections(before_situation_id, subject_id)
    WHERE before_situation_id IS NOT NULL;
CREATE INDEX observation_corrections_after_situation_subject_idx
    ON observation_corrections(after_situation_id, subject_id)
    WHERE after_situation_id IS NOT NULL;
CREATE INDEX thought_corrections_subject_idx
    ON thought_corrections(subject_id);
CREATE INDEX thought_corrections_before_situation_subject_idx
    ON thought_corrections(before_situation_id, subject_id)
    WHERE before_situation_id IS NOT NULL;
CREATE INDEX thought_corrections_after_situation_subject_idx
    ON thought_corrections(after_situation_id, subject_id)
    WHERE after_situation_id IS NOT NULL;

UPDATE app_metadata SET value = '5' WHERE key = 'schema_version';
