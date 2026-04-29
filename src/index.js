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
import {
  landingPage,
  bySizePage,
  allPage,
  sizePage,
  magmaPage,
  submitResultPage,
  notFoundPage,
  profilePage,
  commentHistoryPage,
  reorderHistoryPage,
} from './pages.js'
import {
  loadCurrentUser,
  loadUserFromToken,
  generateApiToken,
  updateSessionUser,
  startOAuth,
  handleCallback,
  logout,
} from './auth.js'

export { Canonicalizer } from './canonicalizer.js'

const HASH_RE = /^[0-9a-f]{64}$/
const HASH_PREFIX_RE = /^[0-9a-f]{1,64}$/

// Resolve a (possibly partial) lowercase-hex hash to a unique full canonical_hash.
// Returns { hash } on success, or { error: 'malformed' | 'not_found' | 'ambiguous' }.
async function resolveHash(env, raw) {
  if (typeof raw !== 'string' || !HASH_PREFIX_RE.test(raw)) {
    return { error: 'malformed' }
  }
  if (raw.length === 64) return { hash: raw }
  const { results } = await env.DB.prepare(
    'SELECT canonical_hash FROM magmas WHERE canonical_hash LIKE ? LIMIT 2',
  )
    .bind(`${raw}%`)
    .all()
  if (results.length === 0) return { error: 'not_found' }
  if (results.length > 1) return { error: 'ambiguous' }
  return { hash: results[0].canonical_hash }
}

const app = new Hono()

app.use(async (c, next) => {
  let user = await loadCurrentUser(c)
  if (!user) user = await loadUserFromToken(c)
  c.set('user', user)
  await next()
})

app.get('/auth/:provider', startOAuth)
app.get('/auth/:provider/callback', handleCallback)
app.post('/auth/logout', logout)

async function listTokens(env, userId) {
  const { results } = await env.DB.prepare(
    `SELECT id, name, prefix, created_at, last_used_at, revoked_at
       FROM api_tokens WHERE user_id = ? ORDER BY created_at DESC`,
  )
    .bind(userId)
    .all()
  return results
}

app.get('/profile', async (c) => {
  const user = c.get('user')
  if (!user) return c.redirect('/auth/github', 302)
  const tokens = await listTokens(c.env, user.id)
  return c.html(profilePage(user, tokens, null))
})

app.post('/profile/name', async (c) => {
  const user = c.get('user')
  if (!user) return c.redirect('/auth/github', 302)
  const body = await c.req.parseBody()
  const name = (typeof body.name === 'string' ? body.name : '').trim().slice(0, 100)
  if (name.length === 0) return c.redirect('/profile', 302)
  await c.env.DB.prepare('UPDATE users SET display_name = ? WHERE id = ?')
    .bind(name, user.id)
    .run()
  await updateSessionUser(c, { display_name: name })
  return c.redirect('/profile', 302)
})

app.post('/profile/tokens', async (c) => {
  const user = c.get('user')
  if (!user) return c.redirect('/auth/github', 302)
  const body = await c.req.parseBody()
  const name = (typeof body.name === 'string' ? body.name : '').trim().slice(0, 100) || null
  const created = await generateApiToken(c.env, user.id, name)
  const tokens = await listTokens(c.env, user.id)
  return c.html(profilePage(user, tokens, created))
})

app.post('/profile/tokens/:id/revoke', async (c) => {
  const user = c.get('user')
  if (!user) return c.redirect('/auth/github', 302)
  const id = Number(c.req.param('id'))
  if (Number.isInteger(id)) {
    await c.env.DB.prepare(
      `UPDATE api_tokens SET revoked_at = CURRENT_TIMESTAMP
         WHERE id = ? AND user_id = ? AND revoked_at IS NULL`,
    )
      .bind(id, user.id)
      .run()
  }
  return c.redirect('/profile', 302)
})

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
  const user = c.get('user')
  if (!user) return c.json({ error: 'authentication required' }, 401)
  const contentType = (c.req.header('content-type') || '').split(';')[0].trim().toLowerCase()
  if (contentType && contentType !== 'text/plain') {
    return c.json({ error: `content-type must be text/plain, got ${contentType}` }, 415)
  }
  const raw = await c.req.text()
  const submitter = user.display_name || user.email || `user-${user.id}`
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
  const user = c.get('user')
  if (!user) return c.redirect('/auth/github', 302)
  const body = await c.req.parseBody()
  const raw = typeof body.table === 'string' ? body.table : ''
  const submitter = user.display_name || user.email || `user-${user.id}`
  const result = await submitMagma(raw, submitter, c.env)
  return c.html(submitResultPage(result, user))
})

app.get('/', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, size, display_reorder FROM magmas ORDER BY RANDOM() LIMIT 4',
  ).all()
  return c.html(landingPage(results, c.get('user')))
})

app.get('/by-size', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT size, COUNT(*) AS count FROM magmas GROUP BY size ORDER BY size',
  ).all()
  return c.html(bySizePage(results, c.get('user')))
})

app.get('/all', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, size, display_reorder FROM magmas ORDER BY size, id',
  ).all()
  return c.html(allPage(results, c.get('user')))
})

app.get('/size/:n', async (c) => {
  const n = Number(c.req.param('n'))
  if (!Number.isInteger(n) || n < 1) {
    return c.html(notFoundPage(`Bad size: ${c.req.param('n')}`, c.get('user')), 404)
  }
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, display_reorder FROM magmas WHERE size = ? ORDER BY id',
  )
    .bind(n)
    .all()
  if (results.length === 0) {
    return c.html(notFoundPage(`No magmas of size ${n}.`, c.get('user')), 404)
  }
  return c.html(sizePage(n, results, c.get('user')))
})

app.get('/magma/:hash', async (c) => {
  const raw = c.req.param('hash')
  const resolved = await resolveHash(c.env, raw)
  if (resolved.error === 'malformed') return c.html(notFoundPage('Malformed hash.', c.get('user')), 404)
  if (resolved.error === 'not_found') return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  if (resolved.error === 'ambiguous') {
    return c.html(notFoundPage(`Ambiguous hash prefix "${raw}" — matches multiple magmas.`, c.get('user')), 400)
  }
  if (resolved.hash !== raw) {
    return c.redirect(`/magma/${resolved.hash}`, 302)
  }
  const row = await c.env.DB.prepare(
    `SELECT m.id, m.canonical_hash, m.size, m.satisfies_255, m.right_cancellative,
            m.idempotent, m.display_reorder, m.r2_key, m.submitted_at, m.submitted_by,
            cl.id AS comment_id, cl.content AS comment_content, cl.created_at AS comment_at,
            u.display_name AS comment_author
       FROM magmas m
       LEFT JOIN comments_log cl ON cl.id = m.current_comment_id
       LEFT JOIN users u ON u.id = cl.user_id
       WHERE m.canonical_hash = ?`,
  )
    .bind(resolved.hash)
    .first()
  if (!row) {
    return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  }
  return c.html(magmaPage(row, c.get('user')))
})

app.get('/magma/:hash/image.png', async (c) => {
  const resolved = await resolveHash(c.env, c.req.param('hash'))
  if (resolved.error) return c.notFound()
  const row = await c.env.DB.prepare(
    'SELECT r2_key, display_reorder FROM magmas WHERE canonical_hash = ?',
  )
    .bind(resolved.hash)
    .first()
  if (!row) return c.notFound()
  const obj = await c.env.BUCKET.get(row.r2_key)
  if (!obj) return c.notFound()
  const text = await obj.text()
  let table = parseCanonicalText(text)
  // ?reorder=<value> overrides the row's stored display_reorder.
  // ?reorder= (empty) → identity. Param absent → use the row's current.
  const reorderQuery = c.req.query('reorder')
  const reorderToApply =
    reorderQuery === undefined ? row.display_reorder : reorderQuery || null
  if (reorderToApply) {
    const parsed = parseReorder(reorderToApply, table.length)
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

const BUCKET_PUBLIC_BASE = 'https://eq677-magmas.icarm.cloud'

app.get('/manifest.json', async (c) => {
  const { results } = await c.env.DB.prepare(
    `SELECT canonical_hash, size, satisfies_255, right_cancellative, idempotent,
            display_reorder, r2_key, submitted_at, submitted_by
       FROM magmas ORDER BY size, id`,
  ).all()
  const magmas = results.map((r) => ({
    canonical_hash: r.canonical_hash,
    size: r.size,
    satisfies_255: r.satisfies_255 === null ? null : Boolean(r.satisfies_255),
    right_cancellative: r.right_cancellative === null ? null : Boolean(r.right_cancellative),
    idempotent: r.idempotent === null ? null : Boolean(r.idempotent),
    display_reorder: r.display_reorder,
    submitted_at: r.submitted_at,
    submitted_by: r.submitted_by,
    url: `${BUCKET_PUBLIC_BASE}/${r.r2_key}`,
  }))
  return new Response(
    JSON.stringify({ count: magmas.length, magmas }),
    {
      headers: {
        'content-type': 'application/json; charset=utf-8',
        'content-disposition': 'attachment; filename="eq677-manifest.json"',
        'cache-control': 'public, max-age=300',
      },
  })
})

const REORDER_BODY_MAX = 16 * 1024 // generous: even n=1000 needs <5 KB

app.post('/magma/:hash/display-reorder', async (c) => {
  const user = c.get('user')
  const ct = (c.req.header('content-type') || '').toLowerCase()
  const isJson = ct.startsWith('application/json')
  if (!user) {
    return isJson
      ? c.json({ error: 'authentication required' }, 401)
      : c.redirect('/auth/github', 302)
  }
  const resolved = await resolveHash(c.env, c.req.param('hash'))
  if (resolved.error === 'malformed') {
    return isJson ? c.json({ error: 'malformed hash' }, 404) : c.notFound()
  }
  if (resolved.error === 'not_found') {
    return isJson ? c.json({ error: 'no such magma' }, 404) : c.notFound()
  }
  if (resolved.error === 'ambiguous') {
    return isJson ? c.json({ error: 'ambiguous hash prefix' }, 400) : c.notFound()
  }
  const hash = resolved.hash
  const declaredLen = Number(c.req.header('content-length'))
  if (Number.isFinite(declaredLen) && declaredLen > REORDER_BODY_MAX) {
    return isJson
      ? c.json({ error: `body exceeds ${REORDER_BODY_MAX} bytes` }, 413)
      : c.notFound()
  }
  let incoming // string | null
  if (isJson) {
    const raw = await c.req.text()
    if (raw.length > REORDER_BODY_MAX) {
      return c.json({ error: `body exceeds ${REORDER_BODY_MAX} bytes` }, 413)
    }
    let body
    try {
      body = JSON.parse(raw)
    } catch {
      return c.json({ error: 'body must be JSON' }, 400)
    }
    if (typeof body !== 'object' || body === null || !('display_reorder' in body)) {
      return c.json({ error: 'body must be { "display_reorder": string | null }' }, 400)
    }
    incoming = body.display_reorder
    if (incoming !== null && typeof incoming !== 'string') {
      return c.json({ error: 'display_reorder must be a string or null' }, 400)
    }
  } else {
    const body = await c.req.parseBody()
    const v = typeof body.display_reorder === 'string' ? body.display_reorder : ''
    incoming = v.length > 0 ? v : null
  }
  const row = await c.env.DB.prepare(
    'SELECT id, size FROM magmas WHERE canonical_hash = ?',
  )
    .bind(hash)
    .first()
  if (!row) return isJson ? c.json({ error: 'no such magma' }, 404) : c.notFound()
  let stored = null
  if (incoming !== null) {
    const parsed = parseReorder(incoming, row.size)
    if (parsed.error) {
      return isJson
        ? c.json({ error: parsed.error }, 400)
        : c.html(
            notFoundPage(parsed.error, user, {
              href: `/magma/${hash}`,
              label: '&larr; back to magma',
            }),
            400,
          )
    }
    stored = parsed.sigma.join(',')
  }
  await c.env.DB.prepare(
    'INSERT INTO display_reorder_log (magma_id, user_id, display_reorder) VALUES (?, ?, ?)',
  )
    .bind(row.id, user.id, stored)
    .run()
  await c.env.DB.prepare('UPDATE magmas SET display_reorder = ? WHERE id = ?')
    .bind(stored, row.id)
    .run()
  if (isJson) {
    return c.json({ canonical_hash: hash, display_reorder: stored })
  }
  return c.redirect(`/magma/${hash}/reorder-history`, 302)
})

app.get('/magma/:hash/reorder-history', async (c) => {
  const resolved = await resolveHash(c.env, c.req.param('hash'))
  if (resolved.error === 'malformed') return c.html(notFoundPage('Malformed hash.', c.get('user')), 404)
  if (resolved.error === 'not_found') return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  if (resolved.error === 'ambiguous') {
    return c.html(notFoundPage(`Ambiguous hash prefix "${c.req.param('hash')}" — matches multiple magmas.`, c.get('user')), 400)
  }
  if (resolved.hash !== c.req.param('hash')) {
    return c.redirect(`/magma/${resolved.hash}/reorder-history`, 302)
  }
  const { results } = await c.env.DB.prepare(
    `SELECT dr.id, dr.display_reorder, dr.created_at, u.display_name AS author
       FROM display_reorder_log dr
       LEFT JOIN users u ON u.id = dr.user_id
       JOIN magmas m ON m.id = dr.magma_id
       WHERE m.canonical_hash = ?
       ORDER BY dr.id DESC`,
  )
    .bind(resolved.hash)
    .all()
  return c.html(reorderHistoryPage(resolved.hash, results, c.get('user')))
})

const COMMENT_MAX = 4096

app.post('/magma/:hash/comment', async (c) => {
  const user = c.get('user')
  if (!user) return c.json({ error: 'authentication required' }, 401)
  const resolved = await resolveHash(c.env, c.req.param('hash'))
  if (resolved.error === 'malformed') return c.json({ error: 'malformed hash' }, 404)
  if (resolved.error === 'not_found') return c.json({ error: 'no such magma' }, 404)
  if (resolved.error === 'ambiguous') return c.json({ error: 'ambiguous hash prefix' }, 400)
  const ct = (c.req.header('content-type') || '').toLowerCase()
  const isJson = ct.startsWith('application/json')
  let content
  if (isJson) {
    let body
    try {
      body = await c.req.json()
    } catch {
      return c.json({ error: 'body must be JSON' }, 400)
    }
    if (typeof body !== 'object' || body === null || typeof body.content !== 'string') {
      return c.json({ error: 'body must be { "content": string }' }, 400)
    }
    content = body.content
  } else {
    const body = await c.req.parseBody()
    content = typeof body.content === 'string' ? body.content : ''
  }
  if (content.length > COMMENT_MAX) {
    if (isJson) return c.json({ error: `comment exceeds ${COMMENT_MAX} chars` }, 413)
    return c.html(notFoundPage(`Comment exceeds ${COMMENT_MAX} chars.`, user), 413)
  }
  const magma = await c.env.DB.prepare('SELECT id FROM magmas WHERE canonical_hash = ?')
    .bind(resolved.hash)
    .first()
  if (!magma) {
    return isJson ? c.json({ error: 'no such magma' }, 404) : c.notFound()
  }
  const ins = await c.env.DB.prepare(
    'INSERT INTO comments_log (magma_id, user_id, content) VALUES (?, ?, ?)',
  )
    .bind(magma.id, user.id, content)
    .run()
  const newCommentId = ins.meta.last_row_id
  await c.env.DB.prepare('UPDATE magmas SET current_comment_id = ? WHERE id = ?')
    .bind(newCommentId, magma.id)
    .run()
  if (isJson) {
    return c.json({
      canonical_hash: resolved.hash,
      comment_id: newCommentId,
      content,
    })
  }
  return c.redirect(`/magma/${resolved.hash}`, 302)
})

app.get('/magma/:hash/comments', async (c) => {
  const resolved = await resolveHash(c.env, c.req.param('hash'))
  if (resolved.error === 'malformed') return c.html(notFoundPage('Malformed hash.', c.get('user')), 404)
  if (resolved.error === 'not_found') return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  if (resolved.error === 'ambiguous') {
    return c.html(notFoundPage(`Ambiguous hash prefix "${c.req.param('hash')}" — matches multiple magmas.`, c.get('user')), 400)
  }
  if (resolved.hash !== c.req.param('hash')) {
    return c.redirect(`/magma/${resolved.hash}/comments`, 302)
  }
  const { results } = await c.env.DB.prepare(
    `SELECT cl.id, cl.content, cl.created_at, u.display_name AS author
       FROM comments_log cl
       LEFT JOIN users u ON u.id = cl.user_id
       JOIN magmas m ON m.id = cl.magma_id
       WHERE m.canonical_hash = ?
       ORDER BY cl.id DESC`,
  )
    .bind(resolved.hash)
    .all()
  return c.html(commentHistoryPage(resolved.hash, results, c.get('user')))
})

app.get('/magma/:hash/table.txt', async (c) => {
  const resolved = await resolveHash(c.env, c.req.param('hash'))
  if (resolved.error) return c.notFound()
  const row = await c.env.DB.prepare(
    'SELECT r2_key FROM magmas WHERE canonical_hash = ?',
  )
    .bind(resolved.hash)
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
