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

export function landingPage() {
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
      <p><a href="/browse">Browse the database &rarr;</a></p>
      <ul>
        <li><a href="https://github.com/memoryleak47/eq677">github.com/memoryleak47/eq677</a></li>
        <li><a href="https://teorth.github.io/equational_theories/">Equational Theories Project</a></li>
      </ul>`
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
      <p>${total} canonical magma${total === 1 ? '' : 's'} across ${sizes.length} size${sizes.length === 1 ? '' : 's'}.</p>
      <ul class="size-list">
      ${rows}
      </ul>`
  return layout('Browse — Equation 677 Database', inner)
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

export function notFoundPage(message) {
  const inner = `
      <h2>Not found</h2>
      <p>${escapeHtml(message || 'No such page.')}</p>
      <p><a href="/">&larr; home</a></p>`
  return layout('Not found — Equation 677 Database', inner)
}
