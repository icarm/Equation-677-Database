CREATE TABLE IF NOT EXISTS magmas (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  canonical_hash TEXT UNIQUE NOT NULL,
  size INTEGER NOT NULL,
  satisfies_255 INTEGER NOT NULL,
  r2_key TEXT NOT NULL,
  submitted_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  submitted_by TEXT
);

CREATE INDEX IF NOT EXISTS idx_magmas_size ON magmas(size);
CREATE INDEX IF NOT EXISTS idx_magmas_satisfies_255 ON magmas(satisfies_255);
