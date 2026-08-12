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
  canonicalPrefix,
  COMMENT_MAX,
  MAX_SIZE,
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
  sizeCommentHistoryPage,
  reorderHistoryPage,
  apiDocsPage,
  acknowledgePage,
  fmePastePage,
  recentPage,
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

const HASH_PREFIX_RE = /^[0-9a-f]{1,64}$/

// D1's LIKE pattern length is capped at 50; a 64-char hash plus '%' exceeds
// it, so use exact equality when the input is already a full hash.
function hashWhereClause(raw) {
  return raw.length === 64
    ? { op: 'm.canonical_hash = ?', bind: raw }
    : { op: 'm.canonical_hash LIKE ?', bind: `${raw}%` }
}

// Resolve a (possibly partial) hash and fetch the row in a single DB
// roundtrip. `columnsSql` is the SELECT list; `joinsSql` is appended after
// `FROM magmas m`. Returns { row } on success or { error }.
async function resolveAndFetchRow(env, raw, columnsSql, joinsSql = '') {
  if (typeof raw !== 'string' || !HASH_PREFIX_RE.test(raw)) {
    return { error: 'malformed' }
  }
  const { op, bind } = hashWhereClause(raw)
  const sql = `SELECT ${columnsSql} FROM magmas m ${joinsSql} WHERE ${op} LIMIT 2`
  const { results } = await env.DB.prepare(sql).bind(bind).all()
  if (results.length === 0) return { error: 'not_found' }
  if (results.length > 1) return { error: 'ambiguous' }
  return { row: results[0] }
}

// History-page variant: resolve and fetch a multi-row history in parallel, so
// effective latency is max(t_resolve, t_history) rather than the sum.
// `historyStmt(op, bind)` should return a prepared+bound D1 statement.
async function resolveAndFetchHistory(env, raw, historyStmt) {
  if (typeof raw !== 'string' || !HASH_PREFIX_RE.test(raw)) {
    return { error: 'malformed' }
  }
  const { op, bind } = hashWhereClause(raw)
  const resolveStmt = env.DB
    .prepare(`SELECT canonical_hash FROM magmas m WHERE ${op} LIMIT 2`)
    .bind(bind)
  const [resolveRes, historyRes] = await Promise.all([
    resolveStmt.all(),
    historyStmt(op, bind).all(),
  ])
  if (resolveRes.results.length === 0) return { error: 'not_found' }
  if (resolveRes.results.length > 1) return { error: 'ambiguous' }
  return { hash: resolveRes.results[0].canonical_hash, rows: historyRes.results }
}

const app = new Hono()

app.use(async (c, next) => {
  let user = await loadCurrentUser(c)
  if (!user) user = await loadUserFromToken(c)
  c.set('user', user)
  await next()
})

app.use(async (c, next) => {
  if (c.req.method !== 'POST') return next()
  const user = c.get('user')
  const key = user
    ? `user:${user.id}`
    : `ip:${c.req.header('cf-connecting-ip') || 'unknown'}`
  try {
    const { success } = await c.env.POST_RATE_LIMITER.limit({ key })
    if (!success) {
      const ct = (c.req.header('content-type') || '').toLowerCase()
      if (ct.startsWith('application/json')) {
        return c.json({ error: 'rate limit exceeded' }, 429)
      }
      return c.html(notFoundPage('Rate limit exceeded — please slow down.', user), 429)
    }
  } catch {
    // rate-limiter unavailable: fail open
  }
  return next()
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

app.get('/api', (c) => c.html(apiDocsPage(c.get('user'))))

app.get('/acknowledge', (c) => c.html(acknowledgePage(c.get('user'))))

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

async function submitMagma(raw, submitter, submitterUserId, env) {
  const parsed = parseText(raw)
  if (parsed.error) return { kind: 'parse_error', message: parsed.error }
  const table = parsed.table
  const n = table.length
  const check = satisfies677(table)
  if (!check.ok) return { kind: 'not_677', x: check.x, y: check.y }
  const rightCancellative = isRightCancellative(table)
  const idempotent = isIdempotent(table)

  const stub = getContainer(env.CANONICALIZER)
  let canonResp
  try {
    canonResp = await stub.fetch('http://container/canonicalize', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ table }),
    })
  } catch (err) {
    // Container crashed or was unreachable mid-request (e.g. OOM-killed on a
    // pathological input). Surface the friendly error path instead of a 500.
    return {
      kind: 'canonicalizer_error',
      status: 502,
      detail: `canonicalizer unreachable: ${err instanceof Error ? err.message : String(err)}`,
    }
  }
  if (!canonResp.ok) {
    const detail = await canonResp.text()
    return { kind: 'canonicalizer_error', status: canonResp.status, detail }
  }
  const { canonical, is255, perm } = await canonResp.json()
  const canonicalHash = await sha256Hex(canonical)

  // `perm` is π: canonical element k came from the submitter's element π[k].
  // The display_reorder σ that reproduces the submitter's ordering when
  // applied to the canonical table is π⁻¹. Validate defensively; if anything
  // is off, fall back to identity (null) rather than refusing the submission.
  let seedReorder = null
  if (Array.isArray(perm) && perm.length === n) {
    const inv = new Array(n)
    const seen = new Uint8Array(n)
    let ok = true
    let isIdentity = true
    for (let k = 0; k < n; k++) {
      const v = perm[k]
      if (!Number.isInteger(v) || v < 0 || v >= n || seen[v]) { ok = false; break }
      seen[v] = 1
      inv[v] = k
      if (v !== k) isIdentity = false
    }
    if (ok && !isIdentity) seedReorder = inv.join(',')
  }

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
    'INSERT INTO magmas (canonical_hash, size, satisfies_255, right_cancellative, idempotent, display_reorder, r2_key, submitted_by, submitted_by_user_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)',
  )
    .bind(canonicalHash, n, is255 ? 1 : 0, rightCancellative ? 1 : 0, idempotent ? 1 : 0, seedReorder, r2Key, submitter, submitterUserId)
    .run()

  // Seed the reorder log so its history always starts with the canonical
  // (identity) ordering, followed by the submitter's ordering when it
  // differs. Mirrors migration 0008's prepopulation for pre-existing magmas.
  await env.DB.prepare(
    'INSERT INTO display_reorder_log (magma_id, user_id, display_reorder) VALUES (?, NULL, NULL)',
  )
    .bind(result.meta.last_row_id)
    .run()
  if (seedReorder !== null) {
    await env.DB.prepare(
      'INSERT INTO display_reorder_log (magma_id, user_id, display_reorder) VALUES (?, NULL, ?)',
    )
      .bind(result.meta.last_row_id, seedReorder)
      .run()
  }

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
  if (contentType && contentType !== 'text/plain' && contentType !== 'application/json') {
    return c.json(
      { error: `content-type must be text/plain or application/json, got ${contentType}` },
      415,
    )
  }
  const raw = await c.req.text()
  const submitter = user.display_name || user.email || `user-${user.id}`
  const result = await submitMagma(raw, submitter, user.id, c.env)
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
  const result = await submitMagma(raw, submitter, user.id, c.env)
  if (result.kind === 'ok') {
    return c.redirect(`/magma/${canonicalPrefix(result.hash)}`, 302)
  }
  return c.html(submitResultPage(result, user))
})

app.get('/', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, size, display_reorder FROM magmas ORDER BY RANDOM() LIMIT 4',
  ).all()
  return c.html(landingPage(results, c.get('user')))
})

// Combined activity feed: new magmas, reorder edits, comments, and size-level
// commentary, newest first. Reorder entries with no user (migration-prepopulated
// rows or the identity/seed rows inserted by submitMagma) are filtered out —
// they don't represent user-initiated activity. Comments are always
// user-initiated.
app.get('/recent', async (c) => {
  const PAGE_SIZE = 20
  const rawPage = Number(c.req.query('page'))
  const page = Number.isInteger(rawPage) && rawPage >= 1 ? rawPage : 1
  // Fetch one extra row so we can tell whether a next page exists without
  // running a separate COUNT.
  const { results } = await c.env.DB.prepare(
    `SELECT * FROM (
       SELECT 'magma' AS kind, m.submitted_at AS at, m.id AS row_id,
              m.canonical_hash AS hash, m.display_reorder AS hash_reorder,
              m.size AS size,
              COALESCE(u.display_name, m.submitted_by) AS author,
              NULL AS detail
         FROM magmas m
         LEFT JOIN users u ON u.id = m.submitted_by_user_id
       UNION ALL
       SELECT 'reorder' AS kind, dr.created_at AS at, dr.id AS row_id,
              m.canonical_hash AS hash, m.display_reorder AS hash_reorder,
              m.size AS size, u.display_name AS author,
              dr.display_reorder AS detail
         FROM display_reorder_log dr
         JOIN magmas m ON m.id = dr.magma_id
         LEFT JOIN users u ON u.id = dr.user_id
         WHERE dr.user_id IS NOT NULL
       UNION ALL
       SELECT 'comment' AS kind, cl.created_at AS at, cl.id AS row_id,
              m.canonical_hash AS hash, m.display_reorder AS hash_reorder,
              m.size AS size, u.display_name AS author,
              cl.content AS detail
         FROM comments_log cl
         JOIN magmas m ON m.id = cl.magma_id
         LEFT JOIN users u ON u.id = cl.user_id
       UNION ALL
       SELECT 'size_comment' AS kind, scl.created_at AS at, scl.id AS row_id,
              NULL AS hash, NULL AS hash_reorder,
              scl.size AS size, u.display_name AS author,
              scl.content AS detail
         FROM size_comments_log scl
         LEFT JOIN users u ON u.id = scl.user_id
     )
     ORDER BY at DESC, row_id DESC
     LIMIT ? OFFSET ?`,
  )
    .bind(PAGE_SIZE + 1, (page - 1) * PAGE_SIZE)
    .all()
  const hasNext = results.length > PAGE_SIZE
  const items = hasNext ? results.slice(0, PAGE_SIZE) : results
  return c.html(recentPage(items, page, hasNext, c.get('user')))
})

app.get('/by-size', async (c) => {
  const [countsRes, commentaryRes] = await Promise.all([
    c.env.DB.prepare(
      'SELECT size, COUNT(*) AS count FROM magmas GROUP BY size ORDER BY size',
    ).all(),
    c.env.DB.prepare(
      `SELECT scl.size
         FROM size_comments_log scl
         JOIN (SELECT size, MAX(id) AS max_id
                 FROM size_comments_log
                 GROUP BY size) latest
           ON latest.size = scl.size AND latest.max_id = scl.id
         WHERE scl.content != ''`,
    ).all(),
  ])
  const counts = new Map(countsRes.results.map((r) => [r.size, r.count]))
  const hasCommentary = new Set(commentaryRes.results.map((r) => r.size))
  const rows = []
  for (let i = 1; i <= MAX_SIZE; i++) {
    rows.push({ size: i, count: counts.get(i) || 0, hasCommentary: hasCommentary.has(i) })
  }
  return c.html(bySizePage(rows, c.get('user')))
})

app.get('/all', async (c) => {
  const { results } = await c.env.DB.prepare(
    'SELECT canonical_hash, size, display_reorder FROM magmas ORDER BY size, id',
  ).all()
  return c.html(allPage(results, c.get('user')))
})

app.get('/size/:n', async (c) => {
  const n = Number(c.req.param('n'))
  if (!Number.isInteger(n) || n < 1 || n > MAX_SIZE) {
    return c.html(notFoundPage(`Bad size: ${c.req.param('n')}`, c.get('user')), 404)
  }
  const [magmasResult, comment] = await Promise.all([
    c.env.DB.prepare(
      'SELECT canonical_hash, display_reorder FROM magmas WHERE size = ? ORDER BY id',
    )
      .bind(n)
      .all(),
    c.env.DB.prepare(
      `SELECT cl.id, cl.content, cl.created_at, u.display_name AS author
         FROM size_comments_log cl
         LEFT JOIN users u ON u.id = cl.user_id
         WHERE cl.size = ?
         ORDER BY cl.id DESC
         LIMIT 1`,
    )
      .bind(n)
      .first(),
  ])
  return c.html(sizePage(n, magmasResult.results, comment, c.get('user')))
})

app.post('/size/:n/comment', async (c) => {
  const user = c.get('user')
  const ct = (c.req.header('content-type') || '').toLowerCase()
  const isJson = ct.startsWith('application/json')
  if (!user) {
    return isJson
      ? c.json({ error: 'authentication required' }, 401)
      : c.redirect('/auth/github', 302)
  }
  const n = Number(c.req.param('n'))
  if (!Number.isInteger(n) || n < 1 || n > MAX_SIZE) {
    return isJson
      ? c.json({ error: `size must be an integer in [1, ${MAX_SIZE}]` }, 404)
      : c.html(notFoundPage(`Bad size: ${c.req.param('n')}`, user), 404)
  }
  const declaredLen = Number(c.req.header('content-length'))
  if (Number.isFinite(declaredLen) && declaredLen > COMMENT_BODY_MAX) {
    return isJson
      ? c.json({ error: `body exceeds ${COMMENT_BODY_MAX} bytes` }, 413)
      : c.html(notFoundPage(`Body exceeds ${COMMENT_BODY_MAX} bytes.`, user), 413)
  }
  let content
  if (isJson) {
    const raw = await c.req.text()
    if (raw.length > COMMENT_BODY_MAX) {
      return c.json({ error: `body exceeds ${COMMENT_BODY_MAX} bytes` }, 413)
    }
    let body
    try {
      body = JSON.parse(raw)
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
  const ins = await c.env.DB.prepare(
    'INSERT INTO size_comments_log (size, user_id, content) VALUES (?, ?, ?)',
  )
    .bind(n, user.id, content)
    .run()
  if (isJson) {
    return c.json({ size: n, comment_id: ins.meta.last_row_id, content })
  }
  return c.redirect(`/size/${n}`, 302)
})

app.get('/size/:n/comment-history', async (c) => {
  const n = Number(c.req.param('n'))
  if (!Number.isInteger(n) || n < 1 || n > MAX_SIZE) {
    return c.html(notFoundPage(`Bad size: ${c.req.param('n')}`, c.get('user')), 404)
  }
  const { results } = await c.env.DB.prepare(
    `SELECT cl.id, cl.content, cl.created_at, u.display_name AS author
       FROM size_comments_log cl
       LEFT JOIN users u ON u.id = cl.user_id
       WHERE cl.size = ?
       ORDER BY cl.id DESC`,
  )
    .bind(n)
    .all()
  return c.html(sizeCommentHistoryPage(n, results, c.get('user')))
})

// Old path — kept as a 301 so any external links / bookmarks survive the rename.
app.get('/size/:n/comments', (c) => c.redirect(`/size/${c.req.param('n')}/comment-history`, 301))

app.get('/magma/:hash', async (c) => {
  const raw = c.req.param('hash')
  const result = await resolveAndFetchRow(
    c.env,
    raw,
    `m.id, m.canonical_hash, m.size, m.satisfies_255, m.right_cancellative,
     m.idempotent, m.display_reorder, m.submitted_at,
     COALESCE(su.display_name, m.submitted_by) AS submitted_by,
     cl.id AS comment_id, cl.content AS comment_content, cl.created_at AS comment_at,
     cu.display_name AS comment_author`,
    `LEFT JOIN users su ON su.id = m.submitted_by_user_id
     LEFT JOIN comments_log cl ON cl.id = m.current_comment_id
     LEFT JOIN users cu ON cu.id = cl.user_id`,
  )
  if (result.error === 'malformed') return c.html(notFoundPage('Malformed hash.', c.get('user')), 404)
  if (result.error === 'not_found') return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  if (result.error === 'ambiguous') {
    return c.html(notFoundPage(`Ambiguous hash prefix "${raw}" — matches multiple magmas.`, c.get('user')), 400)
  }
  const row = result.row
  const slug = canonicalPrefix(row.canonical_hash)
  if (raw !== slug) {
    return c.redirect(`/magma/${slug}`, 302)
  }
  return c.html(magmaPage(row, c.get('user')))
})

app.get('/magma/:hash/image.png', async (c) => {
  const result = await resolveAndFetchRow(c.env, c.req.param('hash'), 'm.r2_key, m.display_reorder')
  if (result.error) return c.notFound()
  const row = result.row
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

// Redirect to the Finite Magma Explorer with the table prepopulated.
// FME accepts ?magma=<JSON or whitespace-separated rows>; we send compact JSON.
// Reorder semantics match /image.png and /table.txt: absent → row's stored,
// ?reorder= empty → identity, ?reorder=<value> → override.
app.get('/magma/:hash/fme', async (c) => {
  const result = await resolveAndFetchRow(
    c.env,
    c.req.param('hash'),
    'm.canonical_hash, m.r2_key, m.display_reorder',
  )
  if (result.error) return c.notFound()
  const row = result.row
  const obj = await c.env.BUCKET.get(row.r2_key)
  if (!obj) return c.notFound()
  const text = await obj.text()
  let table = parseCanonicalText(text)
  const reorderQuery = c.req.query('reorder')
  const reorderToApply =
    reorderQuery === undefined ? row.display_reorder : reorderQuery || null
  if (reorderToApply) {
    const parsed = parseReorder(reorderToApply, table.length)
    if (parsed.sigma) table = applyReorder(table, parsed.sigma)
  }
  const tableJson = JSON.stringify(table)
  const fmeBase = 'https://teorth.github.io/equational_theories/fme/'
  const url = `${fmeBase}?magma=${encodeURIComponent(tableJson)}`
  // GitHub Pages (Fastly) caps request URLs at ~8 KB. For larger magmas we
  // can't prepopulate via the query string; render a fallback page that
  // shows the table for manual paste and a bare-FME link.
  const FME_URL_LIMIT = 7500
  if (url.length > FME_URL_LIMIT) {
    return c.html(fmePastePage(row.canonical_hash, tableJson, fmeBase, c.get('user')))
  }
  return c.redirect(url, 302)
})

const BUCKET_PUBLIC_BASE = 'https://eq677-magmas.icarm.cloud'

app.get('/manifest.json', async (c) => {
  const [magmasRes, sizeCommentsRes] = await Promise.all([
    c.env.DB.prepare(
      `SELECT m.canonical_hash, m.size, m.satisfies_255, m.right_cancellative, m.idempotent,
              m.display_reorder, m.r2_key, m.submitted_at,
              COALESCE(u.display_name, m.submitted_by) AS submitted_by,
              cl.content AS comment
         FROM magmas m
         LEFT JOIN users u ON u.id = m.submitted_by_user_id
         LEFT JOIN comments_log cl ON cl.id = m.current_comment_id
         ORDER BY m.size, m.id`,
    ).all(),
    c.env.DB.prepare(
      `SELECT scl.size, scl.content AS comment
         FROM size_comments_log scl
         JOIN (SELECT size, MAX(id) AS max_id
                 FROM size_comments_log
                 GROUP BY size) latest
           ON latest.size = scl.size AND latest.max_id = scl.id
         WHERE scl.content != ''
         ORDER BY scl.size`,
    ).all(),
  ])
  const magmas = magmasRes.results.map((r) => ({
    canonical_hash: r.canonical_hash,
    size: r.size,
    satisfies_255: r.satisfies_255 === null ? null : Boolean(r.satisfies_255),
    right_cancellative: r.right_cancellative === null ? null : Boolean(r.right_cancellative),
    idempotent: r.idempotent === null ? null : Boolean(r.idempotent),
    display_reorder: r.display_reorder,
    comment: r.comment ? r.comment : null,
    submitted_at: r.submitted_at,
    submitted_by: r.submitted_by,
    url: `${BUCKET_PUBLIC_BASE}/${r.r2_key}`,
  }))
  const sizeCommentary = sizeCommentsRes.results.map((r) => ({
    size: r.size,
    comment: r.comment,
  }))
  return new Response(
    JSON.stringify({ count: magmas.length, magmas, size_commentary: sizeCommentary }),
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
  const result = await resolveAndFetchRow(c.env, c.req.param('hash'), 'm.id, m.size, m.canonical_hash')
  if (result.error === 'malformed') {
    return isJson ? c.json({ error: 'malformed hash' }, 404) : c.notFound()
  }
  if (result.error === 'not_found') {
    return isJson ? c.json({ error: 'no such magma' }, 404) : c.notFound()
  }
  if (result.error === 'ambiguous') {
    return isJson ? c.json({ error: 'ambiguous hash prefix' }, 400) : c.notFound()
  }
  const row = result.row
  const hash = row.canonical_hash
  const slug = canonicalPrefix(hash)
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
  let stored = null
  if (incoming !== null) {
    const parsed = parseReorder(incoming, row.size)
    if (parsed.error) {
      return isJson
        ? c.json({ error: parsed.error }, 400)
        : c.html(
            notFoundPage(parsed.error, user, {
              href: `/magma/${slug}`,
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
  return c.redirect(`/magma/${slug}/reorder-history`, 302)
})

app.get('/magma/:hash/reorder-history', async (c) => {
  const raw = c.req.param('hash')
  const result = await resolveAndFetchHistory(c.env, raw, (op, bind) =>
    c.env.DB
      .prepare(
        `SELECT dr.id, dr.display_reorder, dr.created_at, u.display_name AS author
           FROM display_reorder_log dr
           LEFT JOIN users u ON u.id = dr.user_id
           JOIN magmas m ON m.id = dr.magma_id
           WHERE ${op}
           ORDER BY dr.id DESC`,
      )
      .bind(bind),
  )
  if (result.error === 'malformed') return c.html(notFoundPage('Malformed hash.', c.get('user')), 404)
  if (result.error === 'not_found') return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  if (result.error === 'ambiguous') {
    return c.html(notFoundPage(`Ambiguous hash prefix "${raw}" — matches multiple magmas.`, c.get('user')), 400)
  }
  const slug = canonicalPrefix(result.hash)
  if (slug !== raw) {
    return c.redirect(`/magma/${slug}/reorder-history`, 302)
  }
  return c.html(reorderHistoryPage(result.hash, result.rows, c.get('user')))
})

// Cap on the request body itself (large enough for COMMENT_MAX UTF-8 + JSON wrap).
const COMMENT_BODY_MAX = COMMENT_MAX * 4 + 256

app.post('/magma/:hash/comment', async (c) => {
  const user = c.get('user')
  const ct = (c.req.header('content-type') || '').toLowerCase()
  const isJson = ct.startsWith('application/json')
  if (!user) {
    return isJson
      ? c.json({ error: 'authentication required' }, 401)
      : c.redirect('/auth/github', 302)
  }
  const result = await resolveAndFetchRow(c.env, c.req.param('hash'), 'm.id, m.canonical_hash')
  if (result.error === 'malformed') {
    return isJson ? c.json({ error: 'malformed hash' }, 404) : c.notFound()
  }
  if (result.error === 'not_found') {
    return isJson ? c.json({ error: 'no such magma' }, 404) : c.notFound()
  }
  if (result.error === 'ambiguous') {
    return isJson ? c.json({ error: 'ambiguous hash prefix' }, 400) : c.notFound()
  }
  const magma = result.row
  const declaredLen = Number(c.req.header('content-length'))
  if (Number.isFinite(declaredLen) && declaredLen > COMMENT_BODY_MAX) {
    return isJson
      ? c.json({ error: `body exceeds ${COMMENT_BODY_MAX} bytes` }, 413)
      : c.html(notFoundPage(`Body exceeds ${COMMENT_BODY_MAX} bytes.`, user), 413)
  }
  let content
  if (isJson) {
    const raw = await c.req.text()
    if (raw.length > COMMENT_BODY_MAX) {
      return c.json({ error: `body exceeds ${COMMENT_BODY_MAX} bytes` }, 413)
    }
    let body
    try {
      body = JSON.parse(raw)
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
      canonical_hash: magma.canonical_hash,
      comment_id: newCommentId,
      content,
    })
  }
  return c.redirect(`/magma/${canonicalPrefix(magma.canonical_hash)}`, 302)
})

app.get('/magma/:hash/comment-history', async (c) => {
  const raw = c.req.param('hash')
  const result = await resolveAndFetchHistory(c.env, raw, (op, bind) =>
    c.env.DB
      .prepare(
        `SELECT cl.id, cl.content, cl.created_at, u.display_name AS author
           FROM comments_log cl
           LEFT JOIN users u ON u.id = cl.user_id
           JOIN magmas m ON m.id = cl.magma_id
           WHERE ${op}
           ORDER BY cl.id DESC`,
      )
      .bind(bind),
  )
  if (result.error === 'malformed') return c.html(notFoundPage('Malformed hash.', c.get('user')), 404)
  if (result.error === 'not_found') return c.html(notFoundPage('No such magma.', c.get('user')), 404)
  if (result.error === 'ambiguous') {
    return c.html(notFoundPage(`Ambiguous hash prefix "${raw}" — matches multiple magmas.`, c.get('user')), 400)
  }
  const slug = canonicalPrefix(result.hash)
  if (slug !== raw) {
    return c.redirect(`/magma/${slug}/comment-history`, 302)
  }
  return c.html(commentHistoryPage(result.hash, result.rows, c.get('user')))
})

// Old path — kept as a 301 so any external links / bookmarks survive the rename.
app.get('/magma/:hash/comments', (c) => c.redirect(`/magma/${c.req.param('hash')}/comment-history`, 301))

app.get('/magma/:hash/table.txt', async (c) => {
  const result = await resolveAndFetchRow(c.env, c.req.param('hash'), 'm.r2_key')
  if (result.error) return c.notFound()
  const row = result.row
  const obj = await c.env.BUCKET.get(row.r2_key)
  if (!obj) return c.notFound()
  const reorderQuery = c.req.query('reorder')
  if (reorderQuery === undefined) {
    // Canonical — stream straight through.
    return new Response(obj.body, {
      headers: {
        'content-type': 'text/plain; charset=utf-8',
        'cache-control': 'public, max-age=31536000, immutable',
      },
    })
  }
  const text = await obj.text()
  let table = parseCanonicalText(text)
  if (reorderQuery !== '') {
    const parsed = parseReorder(reorderQuery, table.length)
    if (parsed.sigma) table = applyReorder(table, parsed.sigma)
  }
  const out = table.map((r) => r.join(' ')).join('\n') + '\n'
  return new Response(out, {
    headers: {
      'content-type': 'text/plain; charset=utf-8',
      'cache-control': 'public, max-age=31536000, immutable',
    },
  })
})

export default app
