// FNV-1a 32-bit hash → 8 hex chars. Used as a cache-busting version token
// for image URLs that depend on display_reorder. NULL/empty → '0'.
function reorderVersion(s) {
  if (s == null || s === '') return '0'
  let h = 0x811c9dc5
  for (let i = 0; i < s.length; i++) {
    h ^= s.charCodeAt(i)
    h = (h + ((h << 1) + (h << 4) + (h << 7) + (h << 8) + (h << 24))) >>> 0
  }
  return h.toString(16).padStart(8, '0')
}

function imageUrl(hash, displayReorder) {
  return `/magma/${hash}/image.png?v=${reorderVersion(displayReorder)}`
}

function escapeHtml(s) {
  if (s == null) return ''
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function pageHead({ topLinks = [], title, subtitle }) {
  const top = topLinks.length
    ? `<p class="page-nav">${topLinks
        .map(([href, text]) => `<a href="${href}">${text}</a>`)
        .join(' &nbsp;&middot;&nbsp; ')}</p>`
    : ''
  const sub = subtitle ? `<p class="page-subtitle">${subtitle}</p>` : ''
  return `
      ${top}
      <h2>${title}</h2>
      ${sub}`
}

function commentSection(row, user) {
  const hash = row.canonical_hash
  const hasComment = row.comment_content && row.comment_content.length > 0
  const meta = row.comment_id
    ? `<p class="comment-meta">last edited ${row.comment_author ? `by ${escapeHtml(row.comment_author)} ` : ''}at ${escapeHtml(row.comment_at)} &middot; <a href="/magma/${hash}/comments">history</a></p>`
    : ''
  const display = hasComment
    ? `<div class="comment-body">${escapeHtml(row.comment_content)}</div>`
    : `<p class="muted">No comment yet.</p>`
  const editor = user
    ? `<details class="comment-edit">
        <summary>edit</summary>
        <form method="post" action="/magma/${hash}/comment">
          <textarea name="content" rows="6" maxlength="4096">${escapeHtml(row.comment_content || '')}</textarea>
          <div><button type="submit">save</button> <span class="muted">submit empty to clear</span></div>
        </form>
      </details>`
    : ''
  return `<section class="comment-section">
        <h3>Comment</h3>
        ${display}
        ${meta}
        ${editor}
      </section>`
}

function authNav(user) {
  if (user) {
    const name = escapeHtml(user.display_name || user.email || 'user')
    return `<a href="/profile" class="auth-user">${name}</a>
          <form class="auth-logout" method="post" action="/auth/logout"><button type="submit">log out</button></form>`
  }
  return `<a href="/auth/github">log in with GitHub</a>`
}

function layout(title, bodyInner, user) {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${escapeHtml(title)}</title>
    <link rel="icon" type="image/png" href="/favicon.png" />
    <link rel="stylesheet" href="/style.css" />
  </head>
  <body>
    <header>
      <div class="inner">
        <h1><a href="/">Equation 677 Database</a></h1>
        <nav>
          <span class="auth-nav">${authNav(user)}</span>
        </nav>
      </div>
    </header>
    <main>${bodyInner}</main>
    <footer><a href="https://github.com/icarm/Equation-677-Database">source</a> &nbsp;&middot;&nbsp; <a href="https://icarm.io">icarm.io</a></footer>
  </body>
</html>
`
}

export function landingPage(samples = [], user = null) {
  const thumbs = samples
    .map((s) => {
      const short = s.canonical_hash.slice(0, 8)
      const title = `magma ${short} of size ${s.size}`
      return `<a class="thumb" href="/magma/${s.canonical_hash}" title="${escapeHtml(title)}"><img src="${imageUrl(s.canonical_hash, s.display_reorder)}" width="128" height="128" alt="${escapeHtml(title)}" loading="lazy" /></a>`
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
      <p class="browse-cta"><a href="/all">Browse the database &rarr;</a></p>
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
  return layout('Equation 677 Database', inner, user)
}

export function bySizePage(sizes, user = null) {
  const total = sizes.reduce((acc, s) => acc + s.count, 0)
  const rows = sizes
    .map(
      (s) => `<li><a href="/size/${s.size}">size ${s.size}</a> <span class="count">(${s.count})</span></li>`,
    )
    .join('\n      ')
  const head = pageHead({
    topLinks: [['/all', '&larr; all']],
    title: 'By size',
    subtitle: `${total} isomorphism class${total === 1 ? '' : 'es'} across ${sizes.length} size${sizes.length === 1 ? '' : 's'}. <a href="/manifest.json" download>Manifest (JSON) &darr;</a>`,
  })
  const inner = `${head}
      <ul class="size-list">
      ${rows}
      </ul>`
  return layout('By size — Equation 677 Database', inner, user)
}

export function allPage(items, user = null) {
  const thumbs = items
    .map((s) => {
      const short = s.canonical_hash.slice(0, 8)
      const title = `magma ${short} of size ${s.size}`
      return `<a class="thumb" href="/magma/${s.canonical_hash}" title="${escapeHtml(title)}"><img src="${imageUrl(s.canonical_hash, s.display_reorder)}" width="96" height="96" alt="${escapeHtml(title)}" loading="lazy" /></a>`
    })
    .join('\n      ')
  const head = pageHead({
    topLinks: [['/by-size', 'by size &rarr;']],
    title: 'All',
    subtitle: `${items.length} isomorphism class${items.length === 1 ? '' : 'es'}. <a href="/manifest.json" download>Manifest (JSON) &darr;</a>`,
  })
  const inner = `${head}
      <div class="thumb-grid">
      ${thumbs}
      </div>`
  return layout('All — Equation 677 Database', inner, user)
}

export function sizePage(n, items, user = null) {
  const thumbs = items
    .map((it) => {
      const h = it.canonical_hash
      const title = `magma ${h.slice(0, 8)} of size ${n}`
      return `<a class="thumb" href="/magma/${h}" title="${escapeHtml(title)}"><img src="${imageUrl(h, it.display_reorder)}" width="96" height="96" alt="${escapeHtml(title)}" /></a>`
    })
    .join('\n      ')
  const head = pageHead({
    topLinks: [['/all', '&larr; all'], ['/by-size', '&larr; by size']],
    title: `Size ${n}`,
    subtitle: `${items.length} isomorphism class${items.length === 1 ? '' : 'es'}.`,
  })
  const inner = `${head}
      <div class="thumb-grid">
      ${thumbs}
      </div>`
  return layout(`Size ${n} — Equation 677 Database`, inner, user)
}

export function magmaPage(row, user = null) {
  const hash = row.canonical_hash
  const short = hash.slice(0, 12)
  const submitted = row.submitted_by
    ? `<dd>${escapeHtml(row.submitted_by)}</dd>`
    : `<dd class="muted">&mdash;</dd>`
  const head = pageHead({
    topLinks: [
      ['/all', '&larr; all'],
      ['/by-size', '&larr; by size'],
      [`/size/${row.size}`, `&larr; size ${row.size}`],
    ],
    title: `Magma <code>${escapeHtml(short)}&hellip;</code>`,
  })
  const inner = `${head}
      <div class="magma-image-wrap">
        <img class="magma-image" src="${imageUrl(hash, row.display_reorder)}" alt="magma ${escapeHtml(short)}" />
      </div>
      <dl class="magma-meta">
        <dt>Size</dt>
        <dd>${row.size}</dd>
        <dt>Isomorphism class hash</dt>
        <dd><code>${escapeHtml(hash)}</code></dd>
        <dt>Satisfies Equation 255</dt>
        <dd>${row.satisfies_255 ? 'yes' : 'no'}</dd>
        <dt>Right-cancellative</dt>
        <dd>${row.right_cancellative === null || row.right_cancellative === undefined ? '<span class="muted">unknown</span>' : row.right_cancellative ? 'yes' : 'no'}</dd>
        <dt>Idempotent</dt>
        <dd>${row.idempotent === null || row.idempotent === undefined ? '<span class="muted">unknown</span>' : row.idempotent ? 'yes' : 'no'}</dd>
        <dt>Submitted by</dt>
        ${submitted}
        <dt>Submitted at</dt>
        <dd>${escapeHtml(row.submitted_at)}</dd>
        <dt>Display reorder</dt>
        <dd class="reorder">
          ${row.display_reorder
            ? `<code>${escapeHtml(row.display_reorder)}</code>`
            : `<span class="muted">identity</span>`}
          <span class="reorder-history-link"><a href="/magma/${hash}/reorder-history">history</a></span>
          ${user
            ? `<details class="reorder-edit">
              <summary>edit</summary>
              <form method="post" action="/magma/${hash}/display-reorder">
                <input type="text" name="display_reorder" value="${escapeHtml(row.display_reorder || '')}" placeholder="0,1,2,..." maxlength="4096" />
                <div><button type="submit">save</button> <span class="muted">submit empty for identity</span></div>
              </form>
            </details>`
            : ''}
        </dd>
        <dt>Raw table</dt>
        <dd><a href="/magma/${hash}/table.txt">text</a></dd>
      </dl>
      ${commentSection(row, user)}`
  return layout(`Magma ${short} — Equation 677 Database`, inner, user)
}

export function submitResultPage(result, user = null) {
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
      : '<p>This isomorphism class was <strong>already in the database</strong>.</p>'
    body = `
        <p><strong>Accepted.</strong> The table satisfies Equation 677.</p>
        <p>Satisfies Equation 255: <strong>${result.is255 ? 'yes' : 'no'}</strong>${result.is255 ? '' : ' &mdash; this would resolve the open problem!'}</p>
        <p>Right-cancellative: <strong>${result.rightCancellative ? 'yes' : 'no'}</strong></p>
        <p>Idempotent: <strong>${result.idempotent ? 'yes' : 'no'}</strong></p>
        ${freshLine}
        <p><a href="/magma/${result.hash}">View the isomorphism class &rarr;</a></p>`
    status = 'accepted'
  }
  const inner = `
      <h2>Submission result</h2>
      <div class="submit-result submit-${status}">
        ${body}
      </div>
      <p><a href="/">&larr; back</a></p>`
  return layout('Submission result — Equation 677 Database', inner, user)
}

export function profilePage(user, tokens, newToken) {
  const head = pageHead({
    title: 'Profile',
    subtitle: `Signed in as ${escapeHtml(user.display_name || user.email || 'user')} (via ${escapeHtml(user.provider)}).`,
  })
  const newTokenBlock = newToken
    ? `<div class="new-token">
        <p><strong>New token created.</strong> Copy it now &mdash; this is the only time it will be shown.</p>
        <pre class="token-secret">${escapeHtml(newToken.token)}</pre>
        <p class="muted">Use it as <code>Authorization: Bearer ${escapeHtml(newToken.token)}</code> when calling the API.</p>
      </div>`
    : ''
  const tokenRows = tokens.length
    ? tokens
        .map((t) => {
          const label = t.name ? escapeHtml(t.name) : '<span class="muted">(unnamed)</span>'
          const status = t.revoked_at
            ? `<span class="muted">revoked ${escapeHtml(t.revoked_at)}</span>`
            : `<form method="post" action="/profile/tokens/${t.id}/revoke" class="inline-form">
                <button type="submit" class="link-button">revoke</button>
              </form>`
          const lastUsed = t.last_used_at ? escapeHtml(t.last_used_at) : '<span class="muted">never</span>'
          return `<tr>
            <td><code>${escapeHtml(t.prefix)}&hellip;</code></td>
            <td>${label}</td>
            <td>${escapeHtml(t.created_at)}</td>
            <td>${lastUsed}</td>
            <td>${status}</td>
          </tr>`
        })
        .join('\n')
    : `<tr><td colspan="5" class="muted">No tokens yet.</td></tr>`
  const inner = `${head}
      ${newTokenBlock}
      <section class="tokens">
        <h3>API tokens</h3>
        <p>Use a token in the <code>Authorization: Bearer &hellip;</code> header to submit magmas without logging in interactively.</p>
        <table class="tokens-table">
          <thead><tr><th>Prefix</th><th>Name</th><th>Created</th><th>Last used</th><th></th></tr></thead>
          <tbody>${tokenRows}</tbody>
        </table>
        <form method="post" action="/profile/tokens" class="new-token-form">
          <label>Name (optional) <input type="text" name="name" maxlength="100" placeholder="e.g. laptop CLI" /></label>
          <button type="submit">Generate new token</button>
        </form>
      </section>`
  return layout('Profile — Equation 677 Database', inner, user)
}

export function reorderHistoryPage(hash, entries, user = null) {
  const short = hash.slice(0, 12)
  const head = pageHead({
    topLinks: [[`/magma/${hash}`, `&larr; magma ${escapeHtml(short)}&hellip;`]],
    title: 'Display reorder history',
    subtitle: `${entries.length} entr${entries.length === 1 ? 'y' : 'ies'}.`,
  })
  const items = entries.length
    ? entries
        .map((e, idx) => {
          const reorderQ = e.display_reorder ? encodeURIComponent(e.display_reorder) : ''
          const thumb = `<img src="/magma/${hash}/image.png?reorder=${reorderQ}" width="96" height="96" alt="reorder thumbnail" loading="lazy" />`
          const isCurrent = idx === 0
          const restoreButton = user && !isCurrent
            ? `<form method="post" action="/magma/${hash}/display-reorder" class="inline-form">
                <input type="hidden" name="display_reorder" value="${escapeHtml(e.display_reorder || '')}" />
                <button type="submit">restore</button>
              </form>`
            : ''
          const currentBadge = isCurrent ? `<span class="muted">(current)</span>` : ''
          return `<li class="reorder-entry">
        <div class="reorder-entry-thumb">${thumb}</div>
        <div class="reorder-entry-info">
          <p class="comment-meta">${e.author ? escapeHtml(e.author) : '<span class="muted">(no user)</span>'} &middot; ${escapeHtml(e.created_at)} ${currentBadge} ${restoreButton}</p>
          ${e.display_reorder
            ? `<div class="reorder-value-wrap"><code class="reorder-value">${escapeHtml(e.display_reorder)}</code></div>`
            : `<p class="muted">identity</p>`}
        </div>
      </li>`
        })
        .join('\n')
    : `<li class="muted">No entries.</li>`
  const inner = `${head}
      <ul class="reorder-history">${items}</ul>`
  return layout(`Reorder history ${short} — Equation 677 Database`, inner, user)
}

export function commentHistoryPage(hash, entries, user = null) {
  const short = hash.slice(0, 12)
  const head = pageHead({
    topLinks: [[`/magma/${hash}`, `&larr; magma ${escapeHtml(short)}&hellip;`]],
    title: 'Comment history',
    subtitle: `${entries.length} edit${entries.length === 1 ? '' : 's'}.`,
  })
  const items = entries.length
    ? entries
        .map(
          (e) => `<li>
        <p class="comment-meta">${e.author ? escapeHtml(e.author) : '<span class="muted">(deleted user)</span>'} &middot; ${escapeHtml(e.created_at)}</p>
        ${e.content && e.content.length > 0
          ? `<div class="comment-body">${escapeHtml(e.content)}</div>`
          : `<p class="muted">(cleared)</p>`}
      </li>`,
        )
        .join('\n')
    : `<li class="muted">No comments yet.</li>`
  const inner = `${head}
      <ul class="comment-history">${items}</ul>`
  return layout(`Comment history ${short} — Equation 677 Database`, inner, user)
}

export function notFoundPage(message, user = null, backLink = null) {
  const link = backLink
    ? `<a href="${backLink.href}">${backLink.label}</a>`
    : `<a href="/">&larr; home</a>`
  const inner = `
      <h2>Not found</h2>
      <p>${escapeHtml(message || 'No such page.')}</p>
      <p>${link}</p>`
  return layout('Not found — Equation 677 Database', inner, user)
}
