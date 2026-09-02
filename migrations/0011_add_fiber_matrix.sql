-- Properties of the right-multiplication fiber matrix F, where
--   F[z][x] = #{ y : y ◇ x = z }
-- (an isomorphism invariant of the magma, up to simultaneous permutation of
-- rows and columns). All three are NULL for rows not yet backfilled.
ALTER TABLE magmas ADD COLUMN fiber_matrix_symmetric INTEGER;  -- F = Fᵀ
ALTER TABLE magmas ADD COLUMN fiber_matrix_normal INTEGER;     -- F·Fᵀ = Fᵀ·F
ALTER TABLE magmas ADD COLUMN fiber_matrix_rank INTEGER;       -- rank of F over ℚ

CREATE INDEX IF NOT EXISTS idx_magmas_fiber_matrix_symmetric ON magmas(fiber_matrix_symmetric);
CREATE INDEX IF NOT EXISTS idx_magmas_fiber_matrix_normal ON magmas(fiber_matrix_normal);
CREATE INDEX IF NOT EXISTS idx_magmas_fiber_matrix_rank ON magmas(fiber_matrix_rank);
