CREATE TABLE IF NOT EXISTS comments_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  magma_id INTEGER NOT NULL REFERENCES magmas(id) ON DELETE CASCADE,
  user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
  content TEXT NOT NULL,            -- '' represents a "clear" edit
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_comments_log_magma ON comments_log(magma_id, id);

ALTER TABLE magmas ADD COLUMN current_comment_id INTEGER REFERENCES comments_log(id);
