export const MAX_SIZE = 1000
export const COMMENT_MAX = 4096

// Parse eq677's native whitespace/comma-separated format into a 2D int array.
// Returns { table } on success or { error } on failure.
export function parseText(text) {
  if (typeof text !== 'string') return { error: 'body must be text' }
  const cleaned = text.replace(/,/g, ' ').trim()
  if (cleaned.length === 0) return { error: 'body is empty' }
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
