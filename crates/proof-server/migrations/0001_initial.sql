CREATE TABLE IF NOT EXISTS audit_entries (
    id        TEXT    PRIMARY KEY,
    timestamp TEXT    NOT NULL,
    spec_name TEXT    NOT NULL,
    data      TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_spec ON audit_entries (spec_name);
CREATE INDEX IF NOT EXISTS idx_audit_ts   ON audit_entries (timestamp DESC);

CREATE TABLE IF NOT EXISTS divergences (
    id          TEXT    PRIMARY KEY,
    detected_at TEXT    NOT NULL,
    spec_name   TEXT    NOT NULL,
    customer_id TEXT    NOT NULL,
    data        TEXT    NOT NULL,
    resolved    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_div_spec     ON divergences (spec_name);
CREATE INDEX IF NOT EXISTS idx_div_resolved ON divergences (resolved);

CREATE TABLE IF NOT EXISTS verification_events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id TEXT    NOT NULL,
    spec_name   TEXT    NOT NULL,
    event_type  TEXT    NOT NULL,
    ok          INTEGER NOT NULL,
    timestamp   TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_events_ts ON verification_events (id DESC);