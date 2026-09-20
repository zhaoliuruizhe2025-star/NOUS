CREATE UNIQUE INDEX observations_id_subject_unique_idx
    ON observations(id, subject_id);
CREATE UNIQUE INDEX thoughts_id_subject_unique_idx
    ON thoughts(id, subject_id);
CREATE UNIQUE INDEX emotions_id_subject_unique_idx
    ON emotions(id, subject_id);
CREATE UNIQUE INDEX memories_id_subject_unique_idx
    ON memories(id, subject_id);
CREATE UNIQUE INDEX outcomes_id_subject_unique_idx
    ON outcomes(id, subject_id);
CREATE UNIQUE INDEX beliefs_id_subject_unique_idx
    ON beliefs(id, subject_id);
CREATE UNIQUE INDEX belief_revisions_id_belief_unique_idx
    ON belief_revisions(id, belief_id);
CREATE UNIQUE INDEX values_id_subject_unique_idx
    ON "values"(id, subject_id);
CREATE UNIQUE INDEX value_revisions_id_value_unique_idx
    ON value_revisions(id, value_id);

CREATE TABLE evidence_links (
    id TEXT PRIMARY KEY NOT NULL CHECK (length(trim(id)) > 0),
    subject_id TEXT NOT NULL CHECK (length(trim(subject_id)) > 0),
    relationship_kind TEXT NOT NULL CHECK (
        relationship_kind IN (
            'Supports',
            'Contradicts',
            'Complicates',
            'Contextualizes'
        )
    ),
    provenance TEXT NOT NULL CHECK (provenance = 'UserAuthored'),
    source_kind TEXT NOT NULL CHECK (
        source_kind IN (
            'Observation',
            'Thought',
            'Emotion',
            'Situation',
            'Memory',
            'Decision',
            'Outcome'
        )
    ),
    source_observation_id TEXT NULL CHECK (
        source_observation_id IS NULL OR length(trim(source_observation_id)) > 0
    ),
    source_thought_id TEXT NULL CHECK (
        source_thought_id IS NULL OR length(trim(source_thought_id)) > 0
    ),
    source_emotion_id TEXT NULL CHECK (
        source_emotion_id IS NULL OR length(trim(source_emotion_id)) > 0
    ),
    source_situation_id TEXT NULL CHECK (
        source_situation_id IS NULL OR length(trim(source_situation_id)) > 0
    ),
    source_memory_id TEXT NULL CHECK (
        source_memory_id IS NULL OR length(trim(source_memory_id)) > 0
    ),
    source_decision_id TEXT NULL CHECK (
        source_decision_id IS NULL OR length(trim(source_decision_id)) > 0
    ),
    source_outcome_id TEXT NULL CHECK (
        source_outcome_id IS NULL OR length(trim(source_outcome_id)) > 0
    ),
    target_kind TEXT NOT NULL CHECK (
        target_kind IN ('BeliefRevision', 'ValueRevision')
    ),
    target_belief_id TEXT NULL CHECK (
        target_belief_id IS NULL OR length(trim(target_belief_id)) > 0
    ),
    target_belief_revision_id TEXT NULL CHECK (
        target_belief_revision_id IS NULL OR length(trim(target_belief_revision_id)) > 0
    ),
    target_value_id TEXT NULL CHECK (
        target_value_id IS NULL OR length(trim(target_value_id)) > 0
    ),
    target_value_revision_id TEXT NULL CHECK (
        target_value_revision_id IS NULL OR length(trim(target_value_revision_id)) > 0
    ),
    user_note TEXT NULL CHECK (
        user_note IS NULL OR length(
            trim(user_note, char(9) || char(10) || char(11) || char(12) || char(13) || ' ')
        ) > 0
    ),
    created_at_ms INTEGER NOT NULL,
    CHECK (
        (
            source_kind = 'Observation'
            AND source_observation_id IS NOT NULL
            AND source_thought_id IS NULL
            AND source_emotion_id IS NULL
            AND source_situation_id IS NULL
            AND source_memory_id IS NULL
            AND source_decision_id IS NULL
            AND source_outcome_id IS NULL
        ) OR (
            source_kind = 'Thought'
            AND source_observation_id IS NULL
            AND source_thought_id IS NOT NULL
            AND source_emotion_id IS NULL
            AND source_situation_id IS NULL
            AND source_memory_id IS NULL
            AND source_decision_id IS NULL
            AND source_outcome_id IS NULL
        ) OR (
            source_kind = 'Emotion'
            AND source_observation_id IS NULL
            AND source_thought_id IS NULL
            AND source_emotion_id IS NOT NULL
            AND source_situation_id IS NULL
            AND source_memory_id IS NULL
            AND source_decision_id IS NULL
            AND source_outcome_id IS NULL
        ) OR (
            source_kind = 'Situation'
            AND source_observation_id IS NULL
            AND source_thought_id IS NULL
            AND source_emotion_id IS NULL
            AND source_situation_id IS NOT NULL
            AND source_memory_id IS NULL
            AND source_decision_id IS NULL
            AND source_outcome_id IS NULL
        ) OR (
            source_kind = 'Memory'
            AND source_observation_id IS NULL
            AND source_thought_id IS NULL
            AND source_emotion_id IS NULL
            AND source_situation_id IS NULL
            AND source_memory_id IS NOT NULL
            AND source_decision_id IS NULL
            AND source_outcome_id IS NULL
        ) OR (
            source_kind = 'Decision'
            AND source_observation_id IS NULL
            AND source_thought_id IS NULL
            AND source_emotion_id IS NULL
            AND source_situation_id IS NULL
            AND source_memory_id IS NULL
            AND source_decision_id IS NOT NULL
            AND source_outcome_id IS NULL
        ) OR (
            source_kind = 'Outcome'
            AND source_observation_id IS NULL
            AND source_thought_id IS NULL
            AND source_emotion_id IS NULL
            AND source_situation_id IS NULL
            AND source_memory_id IS NULL
            AND source_decision_id IS NULL
            AND source_outcome_id IS NOT NULL
        )
    ),
    CHECK (
        (
            target_kind = 'BeliefRevision'
            AND target_belief_id IS NOT NULL
            AND target_belief_revision_id IS NOT NULL
            AND target_value_id IS NULL
            AND target_value_revision_id IS NULL
        ) OR (
            target_kind = 'ValueRevision'
            AND target_belief_id IS NULL
            AND target_belief_revision_id IS NULL
            AND target_value_id IS NOT NULL
            AND target_value_revision_id IS NOT NULL
        )
    ),
    CHECK (
        (
            target_kind = 'BeliefRevision'
            AND (
                (
                    source_kind IN (
                        'Observation',
                        'Thought',
                        'Memory',
                        'Decision',
                        'Outcome'
                    )
                    AND relationship_kind IN (
                        'Supports',
                        'Contradicts',
                        'Complicates',
                        'Contextualizes'
                    )
                ) OR (
                    source_kind IN ('Situation', 'Emotion')
                    AND relationship_kind = 'Contextualizes'
                )
            )
        ) OR (
            target_kind = 'ValueRevision'
            AND (
                (
                    source_kind IN (
                        'Observation',
                        'Thought',
                        'Memory',
                        'Decision',
                        'Outcome'
                    )
                    AND relationship_kind IN (
                        'Supports',
                        'Contradicts',
                        'Complicates',
                        'Contextualizes'
                    )
                ) OR (
                    source_kind IN ('Situation', 'Emotion')
                    AND relationship_kind = 'Contextualizes'
                )
            )
        )
    ),
    FOREIGN KEY (subject_id)
        REFERENCES self_subjects(id) ON DELETE RESTRICT,
    FOREIGN KEY (source_observation_id, subject_id)
        REFERENCES observations(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (source_thought_id, subject_id)
        REFERENCES thoughts(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (source_emotion_id, subject_id)
        REFERENCES emotions(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (source_situation_id, subject_id)
        REFERENCES situations(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (source_memory_id, subject_id)
        REFERENCES memories(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (source_decision_id, subject_id)
        REFERENCES decisions(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (source_outcome_id, subject_id)
        REFERENCES outcomes(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (target_belief_id, subject_id)
        REFERENCES beliefs(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (target_belief_revision_id, target_belief_id)
        REFERENCES belief_revisions(id, belief_id) ON DELETE RESTRICT,
    FOREIGN KEY (target_value_id, subject_id)
        REFERENCES "values"(id, subject_id) ON DELETE RESTRICT,
    FOREIGN KEY (target_value_revision_id, target_value_id)
        REFERENCES value_revisions(id, value_id) ON DELETE RESTRICT
);

CREATE INDEX evidence_links_subject_id_idx
    ON evidence_links(subject_id);
CREATE INDEX evidence_links_source_observation_subject_idx
    ON evidence_links(source_observation_id, subject_id)
    WHERE source_observation_id IS NOT NULL;
CREATE INDEX evidence_links_source_thought_subject_idx
    ON evidence_links(source_thought_id, subject_id)
    WHERE source_thought_id IS NOT NULL;
CREATE INDEX evidence_links_source_emotion_subject_idx
    ON evidence_links(source_emotion_id, subject_id)
    WHERE source_emotion_id IS NOT NULL;
CREATE INDEX evidence_links_source_situation_subject_idx
    ON evidence_links(source_situation_id, subject_id)
    WHERE source_situation_id IS NOT NULL;
CREATE INDEX evidence_links_source_memory_subject_idx
    ON evidence_links(source_memory_id, subject_id)
    WHERE source_memory_id IS NOT NULL;
CREATE INDEX evidence_links_source_decision_subject_idx
    ON evidence_links(source_decision_id, subject_id)
    WHERE source_decision_id IS NOT NULL;
CREATE INDEX evidence_links_source_outcome_subject_idx
    ON evidence_links(source_outcome_id, subject_id)
    WHERE source_outcome_id IS NOT NULL;
CREATE INDEX evidence_links_belief_revision_target_idx
    ON evidence_links(target_belief_revision_id, target_belief_id)
    WHERE target_belief_revision_id IS NOT NULL;
CREATE INDEX evidence_links_value_revision_target_idx
    ON evidence_links(target_value_revision_id, target_value_id)
    WHERE target_value_revision_id IS NOT NULL;

UPDATE app_metadata SET value = '4' WHERE key = 'schema_version';
