-- display_reorder: optional permutation σ of [0, n) used only for visualization.
-- Encoded as a comma-separated list of n non-negative integers, e.g. "0,3,1,2,4",
-- where the k-th entry is σ(k): the k-th displayed element corresponds to
-- canonical element σ(k). When rendering, the canonical Cayley table is
-- relabeled by σ (rows, columns, and values all permuted consistently),
-- producing an isomorphic magma whose layout may be more visually structured.
-- NULL means use the canonical labeling as-is (identity permutation).
ALTER TABLE magmas ADD COLUMN display_reorder TEXT;
