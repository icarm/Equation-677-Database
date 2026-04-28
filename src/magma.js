export const MAX_SIZE = 1000

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

export async function sha256Hex(str) {
  const buf = new TextEncoder().encode(str)
  const hash = await crypto.subtle.digest('SHA-256', buf)
  const bytes = new Uint8Array(hash)
  let hex = ''
  for (const b of bytes) hex += b.toString(16).padStart(2, '0')
  return hex
}
