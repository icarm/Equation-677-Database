ALTER TABLE magmas ADD COLUMN idempotent INTEGER;

CREATE INDEX IF NOT EXISTS idx_magmas_idempotent ON magmas(idempotent);
