ALTER TABLE magmas ADD COLUMN right_cancellative INTEGER;

CREATE INDEX IF NOT EXISTS idx_magmas_right_cancellative ON magmas(right_cancellative);
