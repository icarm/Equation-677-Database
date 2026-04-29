CREATE TABLE IF NOT EXISTS display_reorder_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  magma_id INTEGER NOT NULL REFERENCES magmas(id) ON DELETE CASCADE,
  user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
  display_reorder TEXT,             -- NULL = identity
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_display_reorder_log_magma ON display_reorder_log(magma_id, id);

-- Prepopulate: for every magma whose current reorder is non-null,
-- insert an identity entry first so it's the earliest log entry.
INSERT INTO display_reorder_log (magma_id, user_id, display_reorder)
SELECT id, NULL, NULL FROM magmas WHERE display_reorder IS NOT NULL;

-- Prepopulate: insert the current reorder value for every magma.
INSERT INTO display_reorder_log (magma_id, user_id, display_reorder)
SELECT id, NULL, display_reorder FROM magmas;
