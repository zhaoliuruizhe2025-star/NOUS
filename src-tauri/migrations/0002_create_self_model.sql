CREATE TABLE self_subjects (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    created_at_ms INTEGER NOT NULL
);

CREATE TABLE person_references (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    relationship_label TEXT NOT NULL CHECK (length(trim(relationship_label)) > 0),
    context_notes TEXT NULL CHECK (context_notes IS NULL OR length(trim(context_notes)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE situations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    description TEXT NOT NULL CHECK (length(trim(description)) > 0),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (id, subject_id),
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE observations (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (situation_id IS NULL OR length(trim(situation_id)) > 0),
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id) REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE thoughts (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (situation_id IS NULL OR length(trim(situation_id)) > 0),
    content TEXT NOT NULL CHECK (length(trim(content)) > 0),
    confidence INTEGER NULL CHECK (confidence IS NULL OR confidence BETWEEN 0 AND 100),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id) REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE emotions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    situation_id TEXT NULL CHECK (situation_id IS NULL OR length(trim(situation_id)) > 0),
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    intensity INTEGER NOT NULL CHECK (intensity BETWEEN 0 AND 100),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (situation_id, subject_id) REFERENCES situations(id, subject_id) ON DELETE RESTRICT
);

CREATE TABLE beliefs (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE belief_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    belief_id TEXT NOT NULL CHECK (length(trim(belief_id)) > 0),
    revision_number INTEGER NOT NULL CHECK (revision_number > 0),
    proposition TEXT NOT NULL CHECK (length(trim(proposition)) > 0),
    endorsement INTEGER NULL CHECK (endorsement IS NULL OR endorsement BETWEEN 0 AND 100),
    change_note TEXT NULL CHECK (change_note IS NULL OR length(trim(change_note)) > 0),
    origin TEXT NOT NULL CHECK (origin IN ('InitialUserEntry', 'UserUpdate', 'UserCorrection')),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (belief_id, revision_number),
    CHECK ((revision_number = 1 AND origin = 'InitialUserEntry') OR (revision_number > 1 AND origin IN ('UserUpdate', 'UserCorrection'))),
    FOREIGN KEY (belief_id) REFERENCES beliefs(id) ON DELETE RESTRICT
);

CREATE TABLE "values" (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    created_at_ms INTEGER NOT NULL,
    FOREIGN KEY (subject_id) REFERENCES self_subjects(id) ON DELETE RESTRICT
);

CREATE TABLE value_revisions (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    value_id TEXT NOT NULL CHECK (length(trim(value_id)) > 0),
    revision_number INTEGER NOT NULL CHECK (revision_number > 0),
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    importance INTEGER NULL CHECK (importance IS NULL OR importance BETWEEN 0 AND 100),
    change_note TEXT NULL CHECK (change_note IS NULL OR length(trim(change_note)) > 0),
    origin TEXT NOT NULL CHECK (origin IN ('InitialUserEntry', 'UserUpdate', 'UserCorrection')),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (value_id, revision_number),
    CHECK ((revision_number = 1 AND origin = 'InitialUserEntry') OR (revision_number > 1 AND origin IN ('UserUpdate', 'UserCorrection'))),
    FOREIGN KEY (value_id) REFERENCES "values"(id) ON DELETE RESTRICT
);

CREATE INDEX person_references_subject_id_idx ON person_references(subject_id);
CREATE INDEX situations_subject_id_idx ON situations(subject_id);
CREATE INDEX observations_subject_id_idx ON observations(subject_id);
CREATE INDEX observations_situation_subject_idx ON observations(situation_id, subject_id);
CREATE INDEX thoughts_subject_id_idx ON thoughts(subject_id);
CREATE INDEX thoughts_situation_subject_idx ON thoughts(situation_id, subject_id);
CREATE INDEX emotions_subject_id_idx ON emotions(subject_id);
CREATE INDEX emotions_situation_subject_idx ON emotions(situation_id, subject_id);
CREATE INDEX beliefs_subject_id_idx ON beliefs(subject_id);
CREATE INDEX belief_revisions_belief_id_idx ON belief_revisions(belief_id);
CREATE INDEX values_subject_id_idx ON "values"(subject_id);
CREATE INDEX value_revisions_value_id_idx ON value_revisions(value_id);

UPDATE app_metadata SET value = '2' WHERE key = 'schema_version';
