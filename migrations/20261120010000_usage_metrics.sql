CREATE TABLE usage_metrics (
    day DATE NOT NULL,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,
    total BIGINT NOT NULL DEFAULT 1,
    PRIMARY KEY (day, action, outcome)
);
