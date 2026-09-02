export const MAX_SIZE = 1000
export const COMMENT_MAX = 4096

// Canonical magma URLs use this many hex chars (half of the full sha256). The
// underlying isomorphism-class hash is still the full 64-char sha256; this is
// purely a URL-length compromise. 128 bits of identifier remains far more than
// enough to make collisions impossible in practice.
export const CANONICAL_PREFIX_LEN = 32
export const canonicalPrefix = (hash) => hash.slice(0, CANONICAL_PREFIX_LEN)

// Parse a Cayley table into a 2D int array.
// Accepts either:
//   - eq677's native whitespace/comma-separated text (n lines of n integers), or
//   - a JSON array of arrays of integers, e.g. [[0,1],[1,0]] (sniffed via leading '[').
// Returns { table } on success or { error } on failure.
export function parseText(text) {
  if (typeof text !== 'string') return { error: 'body must be text' }
  const trimmed = text.trim()
  if (trimmed.length === 0) return { error: 'body is empty' }
  if (trimmed.startsWith('[')) return parseJsonTable(trimmed)
  const cleaned = trimmed.replace(/,/g, ' ')
  const rows = cleaned.split('\n').map((line) => line.trim()).filter((line) => line.length > 0)
  const n = rows.length
  if (n === 0) return { error: 'no rows found' }
  if (n > MAX_SIZE) return { error: `size ${n} exceeds cap of ${MAX_SIZE}` }
  const table = []
  for (let i = 0; i < n; i++) {
    const parts = rows[i].split(/\s+/)
    if (parts.length !== n) {
      return { error: `row ${i} has ${parts.length} entries, expected ${n}` }
    }
    const row = new Array(n)
    for (let j = 0; j < n; j++) {
      const s = parts[j]
      if (s === '-') {
        return { error: `row ${i}, col ${j}: partial magmas ('-') are not accepted` }
      }
      if (!/^\d+$/.test(s)) {
        return { error: `row ${i}, col ${j}: ${JSON.stringify(s)} is not a non-negative integer` }
      }
      const v = Number(s)
      if (!Number.isInteger(v) || v < 0 || v >= n) {
        return { error: `row ${i}, col ${j}: ${s} is not in [0, ${n})` }
      }
      row[j] = v
    }
    table.push(row)
  }
  return { table }
}

function parseJsonTable(raw) {
  let parsed
  try {
    parsed = JSON.parse(raw)
  } catch (e) {
    return { error: `JSON parse error: ${e.message}` }
  }
  if (!Array.isArray(parsed)) return { error: 'JSON body must be an array of arrays' }
  const n = parsed.length
  if (n === 0) return { error: 'no rows found' }
  if (n > MAX_SIZE) return { error: `size ${n} exceeds cap of ${MAX_SIZE}` }
  const table = new Array(n)
  for (let i = 0; i < n; i++) {
    const row = parsed[i]
    if (!Array.isArray(row)) return { error: `row ${i} is not an array` }
    if (row.length !== n) {
      return { error: `row ${i} has ${row.length} entries, expected ${n}` }
    }
    const out = new Array(n)
    for (let j = 0; j < n; j++) {
      const v = row[j]
      if (typeof v !== 'number' || !Number.isInteger(v) || v < 0 || v >= n) {
        return { error: `row ${i}, col ${j}: ${JSON.stringify(v)} is not in [0, ${n})` }
      }
      out[j] = v
    }
    table[i] = out
  }
  return { table }
}

// Idempotent: ∀ x. x ◇ x = x.
export function isIdempotent(table) {
  const n = table.length
  for (let i = 0; i < n; i++) {
    if (table[i][i] !== i) return false
  }
  return true
}

// Right-cancellative: ∀ a b c. b ◇ a = c ◇ a → b = c.
// Equivalent to: every column of the Cayley table has distinct entries.
export function isRightCancellative(table) {
  const n = table.length
  const seen = new Uint8Array(n)
  for (let j = 0; j < n; j++) {
    seen.fill(0)
    for (let i = 0; i < n; i++) {
      const v = table[i][j]
      if (seen[v]) return false
      seen[v] = 1
    }
  }
  return true
}

// Right-multiplication fiber matrix F, flattened row-major into an Int32Array:
//   F[z*n + x] = #{ y : y ◇ x = z }
// i.e. column x is the fiber-size profile of R_x. Relabeling the magma by a
// permutation conjugates F by the corresponding permutation matrix, so any
// conjugation-invariant property of F is an isomorphism invariant of the magma.
export function fiberMatrix(table) {
  const n = table.length
  const F = new Int32Array(n * n)
  for (let y = 0; y < n; y++) {
    const row = table[y]
    for (let x = 0; x < n; x++) F[row[x] * n + x]++
  }
  return F
}

// Primes just below 2^26. Products of two residues stay below 2^52, so
// modular Gaussian elimination is exact in doubles.
const RANK_PRIMES = [67108859, 67108837]

// Rank of an n×n non-negative integer matrix (flat, row-major) over GF(p).
// Forward elimination only; row-echelon form suffices for rank.
function rankModP(F, n, p) {
  const M = new Float64Array(n * n)
  for (let i = 0; i < n * n; i++) M[i] = F[i] % p
  let rank = 0
  for (let c = 0; c < n && rank < n; c++) {
    let piv = -1
    for (let r = rank; r < n; r++) {
      if (M[r * n + c] !== 0) { piv = r; break }
    }
    if (piv < 0) continue
    if (piv !== rank) {
      for (let k = c; k < n; k++) {
        const t = M[piv * n + k]
        M[piv * n + k] = M[rank * n + k]
        M[rank * n + k] = t
      }
    }
    // Scale the pivot row so the pivot is 1 (inverse via Fermat).
    let inv = 1
    let base = M[rank * n + c]
    let e = p - 2
    while (e > 0) {
      if (e & 1) inv = (inv * base) % p
      base = (base * base) % p
      e >>= 1
    }
    for (let k = c; k < n; k++) M[rank * n + k] = (M[rank * n + k] * inv) % p
    for (let r = rank + 1; r < n; r++) {
      const f = M[r * n + c]
      if (f === 0) continue
      for (let k = c; k < n; k++) {
        let v = (M[r * n + k] - f * M[rank * n + k]) % p
        if (v < 0) v += p
        M[r * n + k] = v
      }
    }
    rank++
  }
  return rank
}

// Properties of the fiber matrix F (see fiberMatrix):
//   symmetric: F = Fᵀ, i.e. #{y : y ◇ x = z} = #{y : y ◇ z = x} for all x, z.
//   normal:    F·Fᵀ = Fᵀ·F.
//   rank:      rank of F over ℚ. Computed modulo two ~2^26 primes and maxed;
//              rank mod p never exceeds rank over ℚ, and undercounting would
//              require both primes to divide every maximal nonzero minor.
// For a right-cancellative magma every R_x is a bijection, so F is the
// all-ones matrix and the answer is (true, true, 1) with no O(n³) work.
export function fiberMatrixProperties(table, rightCancellative = isRightCancellative(table)) {
  const n = table.length
  if (rightCancellative) return { symmetric: true, normal: true, rank: n > 0 ? 1 : 0 }
  const F = fiberMatrix(table)

  let symmetric = true
  outer: for (let z = 1; z < n; z++) {
    for (let x = 0; x < z; x++) {
      if (F[z * n + x] !== F[x * n + z]) { symmetric = false; break outer }
    }
  }

  // F·Fᵀ = Σ_x (column x)(column x)ᵀ and Fᵀ·F = Σ_z (row z)ᵀ(row z); accumulate
  // over the nonzero support only. Entries are ≤ n³ ≤ 1e9, exact in doubles.
  let normal = true
  if (!symmetric) {
    const FFt = new Float64Array(n * n)
    const FtF = new Float64Array(n * n)
    const idx = new Int32Array(n)
    const val = new Int32Array(n)
    for (let x = 0; x < n; x++) {
      let m = 0
      for (let z = 0; z < n; z++) {
        const v = F[z * n + x]
        if (v !== 0) { idx[m] = z; val[m] = v; m++ }
      }
      for (let a = 0; a < m; a++) {
        const base = idx[a] * n
        const va = val[a]
        for (let b = 0; b < m; b++) FFt[base + idx[b]] += va * val[b]
      }
    }
    for (let z = 0; z < n; z++) {
      let m = 0
      for (let x = 0; x < n; x++) {
        const v = F[z * n + x]
        if (v !== 0) { idx[m] = x; val[m] = v; m++ }
      }
      for (let a = 0; a < m; a++) {
        const base = idx[a] * n
        const va = val[a]
        for (let b = 0; b < m; b++) FtF[base + idx[b]] += va * val[b]
      }
    }
    for (let i = 0; i < n * n; i++) {
      if (FFt[i] !== FtF[i]) { normal = false; break }
    }
  }

  let rank = 0
  for (const p of RANK_PRIMES) rank = Math.max(rank, rankModP(F, n, p))
  return { symmetric, normal, rank }
}

// eq677: ∀ x y. x = y ◇ (x ◇ ((y ◇ x) ◇ y))
export function satisfies677(table) {
  const n = table.length
  for (let x = 0; x < n; x++) {
    for (let y = 0; y < n; y++) {
      const yx = table[y][x]
      const yxy = table[yx][y]
      const xyxy = table[x][yxy]
      const yxyxy = table[y][xyxy]
      if (yxyxy !== x) return { ok: false, x, y }
    }
  }
  return { ok: true }
}

// display_reorder is stored as a comma-separated permutation σ of [0, n):
// the k-th displayed element corresponds to canonical element σ(k).
// Returns { sigma } on success or { error } on failure.
export function parseReorder(str, n) {
  if (typeof str !== 'string') return { error: 'reorder must be a string' }
  const parts = str.split(',').map((s) => s.trim()).filter((s) => s.length > 0)
  if (parts.length !== n) {
    return { error: `reorder has ${parts.length} entries, expected ${n}` }
  }
  const sigma = new Array(n)
  const seen = new Uint8Array(n)
  for (let k = 0; k < n; k++) {
    if (!/^\d+$/.test(parts[k])) return { error: `reorder entry ${k}: ${JSON.stringify(parts[k])} is not a non-negative integer` }
    const v = Number(parts[k])
    if (!Number.isInteger(v) || v < 0 || v >= n) return { error: `reorder entry ${k}: ${v} is not in [0, ${n})` }
    if (seen[v]) return { error: `reorder is not a permutation: ${v} repeated` }
    seen[v] = 1
    sigma[k] = v
  }
  return { sigma }
}

// Relabel `table` by permutation σ: out[i][j] = σ⁻¹(table[σ(i)][σ(j)]).
// Produces an isomorphic magma whose elements have been renamed via σ.
export function applyReorder(table, sigma) {
  const n = table.length
  const inv = new Array(n)
  for (let k = 0; k < n; k++) inv[sigma[k]] = k
  const out = new Array(n)
  for (let i = 0; i < n; i++) {
    const row = new Array(n)
    const src = table[sigma[i]]
    for (let j = 0; j < n; j++) row[j] = inv[src[sigma[j]]]
    out[i] = row
  }
  return out
}

export async function sha256Hex(str) {
  const buf = new TextEncoder().encode(str)
  const hash = await crypto.subtle.digest('SHA-256', buf)
  const bytes = new Uint8Array(hash)
  let hex = ''
  for (const b of bytes) hex += b.toString(16).padStart(2, '0')
  return hex
}
