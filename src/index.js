import { Hono } from 'hono'
import { getContainer } from '@cloudflare/containers'
import {
  parseText,
  satisfies677,
  isRightCancellative,
  isIdempotent,
  parseReorder,
  applyReorder,
  sha256Hex,
} from './magma.js'
import { magmaToPng, parseCanonicalText } from './png.js'
import { tarHeader, padding, endOfArchive } from './tar.js'
import {
  landingPage,
  bySizePage,
  allPage,
  sizePage,
  magmaPage,
  submitResultPage,
  notFoundPage,
} from './pages.js'

export { Canonicalizer } from './canonicalizer.js'

const HASH_RE = /^[0-9a-f]{64}$/

const app = new Hono()

async function submitMagma(raw, submitter, env) {
  const parsed = parseText(raw)
  if (parsed.error) return { kind: 'parse_error', message: parsed.error }
  const table = parsed.table
  const n = table.length
  const check = satisfies677(table)
  if (!check.ok) return { kind: 'not_677', x: check.x, y: check.y }
  const rightCancellative = isRightCancellative(table)
  const idempotent = isIdempotent(table)

  const stub = getContainer(env.CANONICALIZER)
  const canonResp = await stub.fetch('http://container/canonicalize', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ table }),
  })
  if (!canonResp.ok) {
    const detail = await canonResp.text()
    return { kind: 'canonicalizer_error', status: canonResp.status, detail }
  }
  const { canonical, is255 } = await canonResp.json()
  const canonicalHash = await sha256Hex(canonical)

  const existing = await env.DB.prepare(
    'SELECT id, satisfies_255, right_cancellative, idempotent FROM magmas WHERE canonical_hash = ?',
  )
    .bind(canonicalHash)
    .first()
  if (existing) {
    if (existing.right_cancellative === null) {
      await env.DB.prepare(
        'UPDATE magmas SET right_cancellative = ? WHERE id = ?',
      )
        .bind(rightCancellative ? 1 : 0, existing.id)
        .run()
    }
    if (existing.idempotent === null) {
      await env.DB.prepare(
        'UPDATE magmas SET idempotent = ? WHERE id = ?',
      )
        .bind(idempotent ? 1 : 0, existing.id)
        .run()
    }
    return {
      kind: 'ok',
      fresh: false,
      id: existing.id,
      hash: canonicalHash,
      size: n,
      is255: Boolean(existing.satisfies_255),
      rightCancellative,
      idempotent,
    }
  }

  const r2Key = `magmas/${n}/${canonicalHash}.txt`
  await env.BUCKET.put(r2Key, canonical, {
    httpMetadata: { contentType: 'text/plain; charset=utf-8' },
  })
  const result = await env.DB.prepare(
    'INSERT INTO magmas (canonical_hash, size, satisfies_255, right_cancellative, idempotent, r2_key, submitted_by) VALUES (?, ?, ?, ?, ?, ?, ?)',
  )
    .bind(canonicalHash, n, is255 ? 1 : 0, rightCancellative ? 1 : 0, idempotent ? 1 : 0, r2Key, submitter)
    .run()

  return {
    kind: 'ok',
    fresh: true,
    id: result.meta.last_row_id,
    hash: canonicalHash,
    size: n,
    is255: Boolean(is255),
    rightCancellative,
    idempotent,
  }
}

app.post('/submit', async (c) => {
  const contentType = (c.req.header('content-type') || '').split(';')[0].trim().toLowerCase()
  if (contentType && contentType !== 'text/plain') {
    return c.json({ error: `content-type must be text/plain, got ${contentType}` }, 415)
  }
  const raw = await c.req.text()
  const submitter = (c.req.header('x-magma-submitter') || '').trim().slice(0, 256) || null
  const result = await submitMagma(raw, submitter, c.env)
  if (result.kind === 'parse_error') return c.json({ error: result.message }, 400)
  if (result.kind === 'not_677') {
    return c.json(
      { error: 'table does not satisfy Equation 677', witness: { x: result.x, y: result.y } },
      422,
    )
  }
  if (result.kind === 'canonicalizer_error') {
    return c.json(
      { error: 'canonicalizer failed', status: result.status, detail: result.detail },
      502,
    )
  }
  return c.json({
    id: result.id,
    canonical_hash: result.hash,
    size: result.size,
    satisfies_255: result.is255,
    right_cancellative: result.rightCancellative,
    idempotent: result.idempotent,
    fresh: result.fresh,
  })
})

app.post('/submit-form', async (c) => {
  const body = await c.req.parseBody()
  const raw = typeof body.table === 'string' ? body.table : ''
  const submitter =
    (typeof body.submitter === 'string' ? body.submitter : '').trim().slice(0, 256) || null
  const result = await submitMagma(raw, submitter, c.env)
  return c.html(submitResultPage(result))
})

app.get('/', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, size, display_reorder FROM magmas ORDER BY RANDOM() LIMIT 4',
  ).all()
  return c.html(landingPage(results))
})

app.get('/by-size', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT size, COUNT(*) AS count FROM magmas GROUP BY size ORDER BY size',
  ).all()
  return c.html(bySizePage(results))
})

app.get('/all', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, size, display_reorder FROM magmas ORDER BY size, id',
  ).all()
  return c.html(allPage(results))
})

app.get('/size/:n', async (c) => {
  const n = Number(c.req.param('n'))
  if (!Number.isInteger(n) || n < 1) {
    return c.html(notFoundPage(`Bad size: ${c.req.param('n')}`), 404)
  }
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, display_reorder FROM magmas WHERE size = ? ORDER BY id',
  )
    .bind(n)
    .all()
  if (results.length === 0) {
    return c.html(notFoundPage(`No magmas of size ${n}.`), 404)
  }
  return c.html(sizePage(n, results))
})

app.get('/magma/:hash', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_RE.test(hash)) {
    return c.html(notFoundPage('Malformed hash.'), 404)
  }
  const row = await c.env.DB.prepare(
    'SELECT id, canonical_hash, size, satisfies_255, right_cancellative, idempotent, display_reorder, r2_key, submitted_at, submitted_by FROM magmas WHERE canonical_hash = ?',
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
    'SELECT r2_key, display_reorder FROM magmas WHERE canonical_hash = ?',
  )
    .bind(hash)
    .first()
  if (!row) return c.notFound()
  const obj = await c.env.BUCKET.get(row.r2_key)
  if (!obj) return c.notFound()
  const text = await obj.text()
  let table = parseCanonicalText(text)
  if (row.display_reorder) {
    const parsed = parseReorder(row.display_reorder, table.length)
    if (parsed.sigma) table = applyReorder(table, parsed.sigma)
  }
  const png = await magmaToPng(table)
  return new Response(png, {
    headers: {
      'content-type': 'image/png',
      'cache-control': 'public, max-age=31536000, immutable',
    },
  })
})

app.get('/download.tar.gz', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT size, canonical_hash, r2_key FROM magmas ORDER BY size, id',
  ).all()
  const bucket = c.env.BUCKET
  let i = 0
  let done = false
  const tar = new ReadableStream({
    async pull(controller) {
      try {
        while (i < results.length) {
          const row = results[i++]
          const obj = await bucket.get(row.r2_key)
          if (!obj) continue
          const body = new Uint8Array(await obj.arrayBuffer())
          const name = `magmas/${row.size}/${row.canonical_hash}.txt`
          controller.enqueue(tarHeader(name, body.length))
          controller.enqueue(body)
          const pad = padding(body.length)
          if (pad.length > 0) controller.enqueue(pad)
          return
        }
        if (!done) {
          done = true
          controller.enqueue(endOfArchive())
          controller.close()
        }
      } catch (e) {
        controller.error(e)
      }
    },
  })
  const gz = tar.pipeThrough(new CompressionStream('gzip'))
  return new Response(gz, {
    headers: {
      'content-type': 'application/gzip',
      'content-disposition': 'attachment; filename="eq677-magmas.tar.gz"',
      'cache-control': 'public, max-age=300',
    },
  })
})

app.post('/magma/:hash/display-reorder', async (c) => {
  const hash = c.req.param('hash')
  if (!HASH_RE.test(hash)) {
    return c.json({ error: 'malformed hash' }, 404)
  }
  let body
  try {
    body = await c.req.json()
  } catch {
    return c.json({ error: 'body must be JSON' }, 400)
  }
  if (typeof body !== 'object' || body === null || !('display_reorder' in body)) {
    return c.json({ error: 'body must be { "display_reorder": string | null }' }, 400)
  }
  const incoming = body.display_reorder
  if (incoming !== null && typeof incoming !== 'string') {
    return c.json({ error: 'display_reorder must be a string or null' }, 400)
  }
  const row = await c.env.DB.prepare(
    'SELECT id, size FROM magmas WHERE canonical_hash = ?',
  )
    .bind(hash)
    .first()
  if (!row) return c.json({ error: 'no such magma' }, 404)
  let stored = null
  if (incoming !== null) {
    const parsed = parseReorder(incoming, row.size)
    if (parsed.error) return c.json({ error: parsed.error }, 400)
    stored = parsed.sigma.join(',')
  }
  await c.env.DB.prepare(
    'UPDATE magmas SET display_reorder = ? WHERE id = ?',
  )
    .bind(stored, row.id)
    .run()
  return c.json({ canonical_hash: hash, display_reorder: stored })
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
