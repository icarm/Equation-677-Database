import { Hono } from 'hono'
import { getContainer } from '@cloudflare/containers'
import { parseText, satisfies677, sha256Hex } from './magma.js'
import { magmaToPng, parseCanonicalText } from './png.js'
import {
  landingPage,
  browsePage,
  sizePage,
  magmaPage,
  notFoundPage,
} from './pages.js'

export { Canonicalizer } from './canonicalizer.js'

const HASH_RE = /^[0-9a-f]{64}$/

const app = new Hono()

app.post('/submit', async (c) => {
  const contentType = (c.req.header('content-type') || '').split(';')[0].trim().toLowerCase()
  if (contentType && contentType !== 'text/plain') {
    return c.json({ error: `content-type must be text/plain, got ${contentType}` }, 415)
  }
  const raw = await c.req.text()
  const parsed = parseText(raw)
  if (parsed.error) return c.json({ error: parsed.error }, 400)
  const table = parsed.table
  const n = table.length
  const submitter = (c.req.header('x-magma-submitter') || '').trim().slice(0, 256) || null
  const check = satisfies677(table)
  if (!check.ok) {
    return c.json(
      { error: 'table does not satisfy Equation 677', witness: { x: check.x, y: check.y } },
      422,
    )
  }

  const stub = getContainer(c.env.CANONICALIZER)
  const canonResp = await stub.fetch('http://container/canonicalize', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ table }),
  })
  if (!canonResp.ok) {
    const detail = await canonResp.text()
    return c.json({ error: 'canonicalizer failed', status: canonResp.status, detail }, 502)
  }
  const { canonical, is255 } = await canonResp.json()
  const canonicalHash = await sha256Hex(canonical)
  const r2Key = `magmas/${n}/${canonicalHash}.txt`

  const existing = await c.env.DB.prepare(
    'SELECT id, satisfies_255 FROM magmas WHERE canonical_hash = ?',
  )
    .bind(canonicalHash)
    .first()
  if (existing) {
    return c.json({
      id: existing.id,
      canonical_hash: canonicalHash,
      size: n,
      satisfies_255: Boolean(existing.satisfies_255),
      fresh: false,
    })
  }

  await c.env.BUCKET.put(r2Key, canonical, {
    httpMetadata: { contentType: 'text/plain; charset=utf-8' },
  })
  const result = await c.env.DB.prepare(
    'INSERT INTO magmas (canonical_hash, size, satisfies_255, r2_key, submitted_by) VALUES (?, ?, ?, ?, ?)',
  )
    .bind(canonicalHash, n, is255 ? 1 : 0, r2Key, submitter)
    .run()

  return c.json({
    id: result.meta.last_row_id,
    canonical_hash: canonicalHash,
    size: n,
    satisfies_255: Boolean(is255),
    fresh: true,
  })
})

app.get('/', (c) => c.html(landingPage()))

app.get('/browse', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT size, COUNT(*) AS count FROM magmas GROUP BY size ORDER BY size',
  ).all()
  return c.html(browsePage(results))
})

app.get('/size/:n', async (c) => {
  const n = Number(c.req.param('n'))
  if (!Number.isInteger(n) || n < 1) {
    return c.html(notFoundPage(`Bad size: ${c.req.param('n')}`), 404)
  }
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash FROM magmas WHERE size = ? ORDER BY id',
  )
    .bind(n)
    .all()
  if (results.length === 0) {
    return c.html(notFoundPage(`No magmas of size ${n}.`), 404)
  }
  return c.html(sizePage(n, results.map((r) => r.canonical_hash)))
})

app.get('/magma/:hash', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_RE.test(hash)) {
    return c.html(notFoundPage('Malformed hash.'), 404)
  }
  const row = await c.env.DB.prepare(
    'SELECT id, canonical_hash, size, satisfies_255, r2_key, submitted_at, submitted_by FROM magmas WHERE canonical_hash = ?',
  )
    .bind(hash)
    .first()
  if (!row) {
    return c.html(notFoundPage('No such magma.'), 404)
  }
  return c.html(magmaPage(row))
})

app.get('/magma/:hash/image.png', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_RE.test(hash)) return c.notFound()
  const row = await c.env.DB.prepare(
    'SELECT r2_key FROM magmas WHERE canonical_hash = ?',
  )
    .bind(hash)
    .first()
  if (!row) return c.notFound()
  const obj = await c.env.BUCKET.get(row.r2_key)
  if (!obj) return c.notFound()
  const text = await obj.text()
  const table = parseCanonicalText(text)
  const png = await magmaToPng(table)
  return new Response(png, {
    headers: {
      'content-type': 'image/png',
      'cache-control': 'public, max-age=31536000, immutable',
    },
  })
})

app.get('/magma/:hash/table.txt', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_RE.test(hash)) return c.notFound()
  const row = await c.env.DB.prepare(
    'SELECT r2_key FROM magmas WHERE canonical_hash = ?',
  )
    .bind(hash)
    .first()
  if (!row) return c.notFound()
  const obj = await c.env.BUCKET.get(row.r2_key)
  if (!obj) return c.notFound()
  return new Response(obj.body, {
    headers: {
      'content-type': 'text/plain; charset=utf-8',
      'cache-control': 'public, max-age=31536000, immutable',
    },
  })
})

export default app
