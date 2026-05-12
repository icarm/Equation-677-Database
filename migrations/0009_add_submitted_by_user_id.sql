-- Add a proper FK from magmas to the submitting user. The legacy
-- `submitted_by TEXT` column is kept as a snapshot fallback for display when
-- the FK is NULL (deleted user, or backfill couldn't match a row).
ALTER TABLE magmas ADD COLUMN submitted_by_user_id INTEGER
  REFERENCES users(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_magmas_submitted_by_user_id
  ON magmas(submitted_by_user_id);

-- Backfill from display_name, but only when the name is unique among users.
UPDATE magmas
SET submitted_by_user_id = (
  SELECT u.id FROM users u
  WHERE u.display_name = magmas.submitted_by
    AND (SELECT COUNT(*) FROM users u2 WHERE u2.display_name = u.display_name) = 1
)
WHERE submitted_by IS NOT NULL
  AND submitted_by_user_id IS NULL;

-- Backfill from email when unique (covers the `user.email` branch of the
-- fallback chain previously used to build the `submitter` string).
UPDATE magmas
SET submitted_by_user_id = (
  SELECT u.id FROM users u
  WHERE u.email = magmas.submitted_by
    AND (SELECT COUNT(*) FROM users u2 WHERE u2.email = u.email) = 1
)
WHERE submitted_by IS NOT NULL
  AND submitted_by_user_id IS NULL;

-- Backfill from the synthetic `user-<id>` pattern (the final fallback used
-- when both display_name and email were absent at submission time).
UPDATE magmas
SET submitted_by_user_id = (
  SELECT u.id FROM users u WHERE 'user-' || u.id = magmas.submitted_by
)
WHERE submitted_by IS NOT NULL
  AND submitted_by_user_id IS NULL;
