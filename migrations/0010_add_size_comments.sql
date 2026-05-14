CREATE TABLE IF NOT EXISTS size_comments_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  size INTEGER NOT NULL,
  user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
  content TEXT NOT NULL,            -- '' represents a "clear" edit
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_size_comments_log_size ON size_comments_log(size, id);
