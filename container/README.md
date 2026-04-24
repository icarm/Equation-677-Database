# container

Produces the `canonicalize-server` image used by the Worker's
`CANONICALIZER` binding. Built and pushed to Cloudflare's managed registry
by `.github/workflows/container.yml`.

## Contents

- `eq677/` — vendored from <https://github.com/memoryleak47/eq677>. Small
  local patches:
  - Added `src/lib.rs` (upstream is binary-only) so the crate can be used
    as a library by `canonicalize-server`.
  - Replaced the nightly-only `become` tail calls in the DPLL solver files
    (`c_dpll/run.rs`, `tinv_dpll/run.rs`, `semitinv_dpll/run.rs`) with
    `return`, so the crate builds on stable Rust. Those code paths aren't
    on the canonicalization hot path.
  - Bumped `petgraph` to 0.8 to unify with the version `nauty-pet` links
    against.
  - Stubbed `build.rs` to emit an empty `DB_SOURCES` — the upstream
    `db/` fixtures are not shipped.
- `canonicalize-server/` — a small Axum HTTP server that exposes
  `POST /canonicalize` and `GET /health` and delegates to
  `eq677::MatrixMagma::canonicalize2` (nauty) + `is255`.
- `Dockerfile` — two-stage build (`rust:slim-bookworm` builder →
  `debian:bookworm-slim` runtime). Links against `libz3` and bundles
  nauty via the `bundled` feature of `nauty-Traces-sys`.
- `Cargo.toml`, `rust-toolchain.toml` — workspace root, pinned to stable.
