ALTER TABLE agents.runs
ADD COLUMN routing_requested_band TEXT,
ADD COLUMN routing_selected_band TEXT,
ADD COLUMN routing_effort TEXT,
ADD COLUMN routing_approval_required BOOLEAN,
ADD constraint RUNS_ROUTING_DECISION_CHECK CHECK (
    (
        routing_requested_band IS NULL
        AND routing_selected_band IS NULL
        AND routing_effort IS NULL
        AND routing_approval_required IS NULL
    )
    OR (
        routing_requested_band IN (
            'xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max'
        )
        AND (
            routing_selected_band IS NULL
            OR routing_selected_band IN (
                'xtra-low', 'low', 'medium', 'high', 'xtra-high', 'max'
            )
        )
        AND routing_effort IS NOT NULL
        AND btrim(routing_effort) <> ''
        AND routing_approval_required IS NOT NULL
    )
);
