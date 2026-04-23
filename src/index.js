import { Hono } from 'hono'
import { getContainer } from '@cloudflare/containers'
import { parseText, satisfies677, sha256Hex } from './magma.js'

export { Canonicalizer } from './canonicalizer.js'

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
      {
        error: 'table does not satisfy Equation 677',
        witness: { x: check.x, y: check.y },
      },
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
    return c.json(
      { error: 'canonicalizer failed', status: canonResp.status, detail },
      502,
    )
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

app.get('/', (c) =>
  c.html(`<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Equation 677 Database</title>
    <link rel="stylesheet" href="/style.css" />
  </head>
  <body>
    <header>
      <div class="inner">
        <h1>Equation 677 Database</h1>
        <nav>
          <a href="/">Home</a>
        </nav>
      </div>
    </header>
    <main>
      <section class="question">
        <p class="lede">Can we find a finite magma <em>M</em> that satisfies</p>
        <div class="eq-line">
          <span class="eq">&forall; x y : M, &nbsp; x = y &#9671; (x &#9671; ((y &#9671; x) &#9671; y))</span>
          <span class="eq-label">(<a href="https://teorth.github.io/equational_theories/implications/?677">Equation 677</a>)</span>
        </div>
        <p class="lede">but does <strong>not</strong> satisfy</p>
        <div class="eq-line">
          <span class="eq">&forall; x : M, &nbsp; x = ((x &#9671; x) &#9671; x) &#9671; x &thinsp;?</span>
          <span class="eq-label">(<a href="https://teorth.github.io/equational_theories/implications/?255">Equation 255</a>)</span>
        </div>
      </section>
      <ul>
        <li><a href="https://github.com/memoryleak47/eq677">github.com/memoryleak47/eq677</a></li>
        <li><a href="https://teorth.github.io/equational_theories/">Equational Theories Project</a></li>
      </ul>
    </main>
    <footer><a href="https://icarm.io">icarm.io</a></footer>
  </body>
</html>
`),
)

export default app
