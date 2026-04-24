function escapeHtml(s) {
  if (s == null) return ''
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function layout(title, bodyInner) {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${escapeHtml(title)}</title>
    <link rel="stylesheet" href="/style.css" />
  </head>
  <body>
    <header>
      <div class="inner">
        <h1><a href="/">Equation 677 Database</a></h1>
        <nav>
          <a href="/browse">Browse</a>
        </nav>
      </div>
    </header>
    <main>${bodyInner}</main>
    <footer><a href="https://icarm.io">icarm.io</a></footer>
  </body>
</html>
`
}

export function landingPage(samples = []) {
  const thumbs = samples
    .map((s) => {
      const short = s.canonical_hash.slice(0, 8)
      const title = `magma ${short} of size ${s.size}`
      return `<a class="thumb" href="/magma/${s.canonical_hash}" title="${escapeHtml(title)}"><img src="/magma/${s.canonical_hash}/image.png" width="128" height="128" alt="${escapeHtml(title)}" loading="lazy" /></a>`
    })
    .join('\n        ')
  const sampleBlock = samples.length
    ? `
      <div class="landing-samples">
        ${thumbs}
      </div>
      <p class="landing-samples-caption">(random magmas from the database)</p>`
    : ''
  const inner = `
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
      <p>This site collects finite magmas that satisfy <a href="https://teorth.github.io/equational_theories/implications/?677">Equation 677</a>. So far, every known example also satisfies Equation 255 &mdash; finding one that does <em>not</em> (or proving none exists) is the main open question remaining from the <a href="https://teorth.github.io/equational_theories/">Equational Theories Project</a>.</p>
      <p>The canonicalization and database are based on <a href="https://github.com/memoryleak47/eq677">memoryleak47/eq677</a>.</p>
      <p class="browse-cta"><a href="/browse">Browse the database &rarr;</a></p>
      ${sampleBlock}
      <section class="submit">
        <h2>Submit a candidate magma</h2>
        <p class="submit-help">Paste a Cayley table: <em>n</em> rows, each with <em>n</em> non-negative integers &lt; <em>n</em>, whitespace- or comma-separated.</p>
        <form method="post" action="/submit-form">
          <textarea name="table" rows="10" required placeholder="0 4 3 2 1&#10;3 1 4 0 2&#10;1 0 2 4 3&#10;4 2 1 3 0&#10;2 3 0 1 4"></textarea>
          <div class="submit-row">
            <label>Submitter (optional) <input type="text" name="submitter" maxlength="256" placeholder="your name or handle" /></label>
            <button type="submit">Submit</button>
          </div>
        </form>
      </section>`
  return layout('Equation 677 Database', inner)
}

export function browsePage(sizes) {
  const total = sizes.reduce((acc, s) => acc + s.count, 0)
  const rows = sizes
    .map(
      (s) => `<li><a href="/size/${s.size}">size ${s.size}</a> <span class="count">(${s.count})</span></li>`,
    )
    .join('\n      ')
  const inner = `
      <h2>Browse the database</h2>
      <p>${total} canonical magma${total === 1 ? '' : 's'} across ${sizes.length} size${sizes.length === 1 ? '' : 's'}. <a href="/all">See all &rarr;</a></p>
      <ul class="size-list">
      ${rows}
      </ul>`
  return layout('Browse — Equation 677 Database', inner)
}

export function allPage(hashes) {
  const thumbs = hashes
    .map(
      (h) =>
        `<a class="thumb" href="/magma/${h}"><img src="/magma/${h}/image.png" width="96" height="96" alt="" loading="lazy" /></a>`,
    )
    .join('\n      ')
  const inner = `
      <div class="thumb-grid">
      ${thumbs}
      </div>`
  return layout('All — Equation 677 Database', inner)
}

export function sizePage(n, hashes) {
  const thumbs = hashes
    .map(
      (h) =>
        `<a class="thumb" href="/magma/${h}" title="${h}"><img src="/magma/${h}/image.png" width="96" height="96" alt="magma ${h.slice(0, 8)}" /></a>`,
    )
    .join('\n      ')
  const inner = `
      <p><a href="/browse">&larr; browse</a></p>
      <h2>Magmas of size ${n}</h2>
      <p>${hashes.length} canonical form${hashes.length === 1 ? '' : 's'}.</p>
      <div class="thumb-grid">
      ${thumbs}
      </div>`
  return layout(`Size ${n} — Equation 677 Database`, inner)
}

export function magmaPage(row) {
  const hash = row.canonical_hash
  const short = hash.slice(0, 12)
  const submitted = row.submitted_by
    ? `<dd>${escapeHtml(row.submitted_by)}</dd>`
    : `<dd class="muted">&mdash;</dd>`
  const inner = `
      <p><a href="/browse">&larr; browse</a> &nbsp;&middot;&nbsp; <a href="/size/${row.size}">size ${row.size}</a></p>
      <h2>Magma <code>${escapeHtml(short)}&hellip;</code></h2>
      <div class="magma-image-wrap">
        <img class="magma-image" src="/magma/${hash}/image.png" alt="magma ${escapeHtml(short)}" />
      </div>
      <dl class="magma-meta">
        <dt>Size</dt>
        <dd>${row.size}</dd>
        <dt>Canonical hash</dt>
        <dd><code>${escapeHtml(hash)}</code></dd>
        <dt>Satisfies Equation 255</dt>
        <dd>${row.satisfies_255 ? 'yes' : 'no'}</dd>
        <dt>Submitted by</dt>
        ${submitted}
        <dt>Submitted at</dt>
        <dd>${escapeHtml(row.submitted_at)}</dd>
        <dt>Raw table</dt>
        <dd><a href="/magma/${hash}/table.txt">text</a></dd>
      </dl>`
  return layout(`Magma ${short} — Equation 677 Database`, inner)
}

export function submitResultPage(result) {
  let status, body
  if (result.kind === 'parse_error') {
    status = 'rejected'
    body = `
        <p><strong>Could not parse the submission.</strong></p>
        <p class="witness">${escapeHtml(result.message)}</p>`
  } else if (result.kind === 'not_677') {
    body = `
        <p><strong>The table does not satisfy Equation 677.</strong></p>
        <p class="witness">Witness: at x = ${result.x}, y = ${result.y}, the identity fails.</p>`
    status = 'rejected'
  } else if (result.kind === 'canonicalizer_error') {
    body = `
        <p><strong>Canonicalizer error.</strong></p>
        <p class="witness">status ${result.status}: ${escapeHtml(result.detail || '')}</p>`
    status = 'error'
  } else {
    const freshLine = result.fresh
      ? '<p>This is a <strong>new</strong> entry in the database.</p>'
      : '<p>This canonical form was <strong>already in the database</strong>.</p>'
    body = `
        <p><strong>Accepted.</strong> The table satisfies Equation 677.</p>
        <p>Satisfies Equation 255: <strong>${result.is255 ? 'yes' : 'no'}</strong>${result.is255 ? '' : ' &mdash; this would resolve the open problem!'}</p>
        ${freshLine}
        <p><a href="/magma/${result.hash}">View the canonical form &rarr;</a></p>`
    status = 'accepted'
  }
  const inner = `
      <h2>Submission result</h2>
      <div class="submit-result submit-${status}">
        ${body}
      </div>
      <p><a href="/">&larr; back</a></p>`
  return layout('Submission result — Equation 677 Database', inner)
}

export function notFoundPage(message) {
  const inner = `
      <h2>Not found</h2>
      <p>${escapeHtml(message || 'No such page.')}</p>
      <p><a href="/">&larr; home</a></p>`
  return layout('Not found — Equation 677 Database', inner)
}
