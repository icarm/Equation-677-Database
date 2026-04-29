import { getCookie, setCookie, deleteCookie } from 'hono/cookie'
import { sha256Hex } from './magma.js'

export const TOKEN_PREFIX = 'eq677_'
const TOKEN_RANDOM_BYTES = 20 // 40 hex chars → 160 bits

const SESSION_COOKIE = 'session'
const STATE_COOKIE = 'oauth_state'
const SESSION_TTL_SEC = 30 * 24 * 60 * 60 // 30 days
const STATE_TTL_SEC = 10 * 60

const PROVIDERS = {
  github: {
    authorize: 'https://github.com/login/oauth/authorize',
    token: 'https://github.com/login/oauth/access_token',
    userInfo: 'https://api.github.com/user',
    scope: 'read:user user:email',
    clientIdEnv: 'GITHUB_CLIENT_ID',
    clientSecretEnv: 'GITHUB_CLIENT_SECRET',
    mapUser: (info) => ({
      provider_user_id: String(info.id),
      email: info.email || null,
      display_name: info.name || info.login || null,
      avatar_url: info.avatar_url || null,
    }),
  },
}

function randomHex(bytes) {
  const a = new Uint8Array(bytes)
  crypto.getRandomValues(a)
  let s = ''
  for (const b of a) s += b.toString(16).padStart(2, '0')
  return s
}

function originOf(req) {
  const u = new URL(req.url)
  return `${u.protocol}//${u.host}`
}

function isHttps(req) {
  return req.url.startsWith('https://')
}

function sessionKey(token) {
  return `session:${token}`
}

export async function loadCurrentUser(c) {
  const token = getCookie(c, SESSION_COOKIE)
  if (!token) return null
  return (await c.env.SESSIONS.get(sessionKey(token), 'json')) || null
}

export async function startOAuth(c) {
  const providerName = c.req.param('provider')
  const provider = PROVIDERS[providerName]
  if (!provider) return c.notFound()
  const clientId = c.env[provider.clientIdEnv]
  if (!clientId) return c.json({ error: `${providerName} OAuth not configured` }, 503)
  const state = randomHex(16)
  setCookie(c, STATE_COOKIE, state, {
    httpOnly: true,
    secure: isHttps(c.req.raw),
    sameSite: 'Lax',
    maxAge: STATE_TTL_SEC,
    path: '/',
  })
  const redirectUri = `${originOf(c.req.raw)}/auth/${providerName}/callback`
  const authUrl = new URL(provider.authorize)
  authUrl.searchParams.set('client_id', clientId)
  authUrl.searchParams.set('redirect_uri', redirectUri)
  authUrl.searchParams.set('scope', provider.scope)
  authUrl.searchParams.set('state', state)
  for (const [k, v] of Object.entries(provider.extraAuthorizeParams || {})) {
    authUrl.searchParams.set(k, v)
  }
  return c.redirect(authUrl.toString(), 302)
}

export async function handleCallback(c) {
  const providerName = c.req.param('provider')
  const provider = PROVIDERS[providerName]
  if (!provider) return c.notFound()
  const error = c.req.query('error')
  const code = c.req.query('code')
  const stateParam = c.req.query('state')
  const stateCookie = getCookie(c, STATE_COOKIE)
  deleteCookie(c, STATE_COOKIE, { path: '/' })
  if (error || !code) {
    return c.redirect('/', 302)
  }
  if (!stateParam || !stateCookie || stateParam !== stateCookie) {
    return c.redirect('/', 302)
  }
  const clientId = c.env[provider.clientIdEnv]
  const clientSecret = c.env[provider.clientSecretEnv]
  if (!clientId || !clientSecret) {
    return c.json({ error: `${providerName} OAuth not configured` }, 503)
  }
  const redirectUri = `${originOf(c.req.raw)}/auth/${providerName}/callback`
  const tokenResp = await fetch(provider.token, {
    method: 'POST',
    headers: {
      'content-type': 'application/x-www-form-urlencoded',
      accept: 'application/json',
    },
    body: new URLSearchParams({
      client_id: clientId,
      client_secret: clientSecret,
      code,
      redirect_uri: redirectUri,
      grant_type: 'authorization_code',
    }),
  })
  if (!tokenResp.ok) {
    return c.json({ error: 'token exchange failed', detail: await tokenResp.text() }, 502)
  }
  const tokenData = await tokenResp.json()
  if (!tokenData.access_token) {
    return c.json({ error: 'no access token in token response' }, 502)
  }
  const userResp = await fetch(provider.userInfo, {
    headers: {
      authorization: `Bearer ${tokenData.access_token}`,
      'user-agent': 'eq677-database',
      accept: 'application/json',
    },
  })
  if (!userResp.ok) {
    return c.json({ error: 'user info fetch failed', detail: await userResp.text() }, 502)
  }
  const info = await userResp.json()
  const mapped = provider.mapUser(info)
  if (!mapped.provider_user_id) {
    return c.json({ error: 'provider returned no user id' }, 502)
  }

  const existing = await c.env.DB.prepare(
    'SELECT id FROM users WHERE provider = ? AND provider_user_id = ?',
  )
    .bind(providerName, mapped.provider_user_id)
    .first()
  let userId
  if (existing) {
    userId = existing.id
    await c.env.DB.prepare(
      `UPDATE users SET email = ?, display_name = ?, avatar_url = ?,
                        last_login_at = CURRENT_TIMESTAMP
         WHERE id = ?`,
    )
      .bind(mapped.email, mapped.display_name, mapped.avatar_url, userId)
      .run()
  } else {
    const ins = await c.env.DB.prepare(
      `INSERT INTO users (provider, provider_user_id, email, display_name, avatar_url, last_login_at)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)`,
    )
      .bind(
        providerName,
        mapped.provider_user_id,
        mapped.email,
        mapped.display_name,
        mapped.avatar_url,
      )
      .run()
    userId = ins.meta.last_row_id
  }

  const sessionToken = randomHex(32)
  const sessionValue = {
    id: userId,
    provider: providerName,
    email: mapped.email,
    display_name: mapped.display_name,
    avatar_url: mapped.avatar_url,
  }
  await c.env.SESSIONS.put(sessionKey(sessionToken), JSON.stringify(sessionValue), {
    expirationTtl: SESSION_TTL_SEC,
  })
  setCookie(c, SESSION_COOKIE, sessionToken, {
    httpOnly: true,
    secure: isHttps(c.req.raw),
    sameSite: 'Lax',
    maxAge: SESSION_TTL_SEC,
    path: '/',
  })
  return c.redirect('/', 302)
}

export async function loadUserFromToken(c) {
  const auth = c.req.header('authorization') || ''
  const m = auth.match(/^Bearer\s+(\S+)$/i)
  if (!m) return null
  const token = m[1]
  if (!token.startsWith(TOKEN_PREFIX)) return null
  const tokenHash = await sha256Hex(token)
  const row = await c.env.DB.prepare(
    `SELECT u.id, u.provider, u.email, u.display_name, u.avatar_url, t.id AS token_id
       FROM api_tokens t JOIN users u ON u.id = t.user_id
       WHERE t.token_hash = ? AND t.revoked_at IS NULL`,
  )
    .bind(tokenHash)
    .first()
  if (!row) return null
  const tokenId = row.token_id
  delete row.token_id
  c.executionCtx?.waitUntil(
    c.env.DB.prepare('UPDATE api_tokens SET last_used_at = CURRENT_TIMESTAMP WHERE id = ?')
      .bind(tokenId)
      .run(),
  )
  return row
}

export async function generateApiToken(env, userId, name) {
  const a = new Uint8Array(TOKEN_RANDOM_BYTES)
  crypto.getRandomValues(a)
  let body = ''
  for (const b of a) body += b.toString(16).padStart(2, '0')
  const token = `${TOKEN_PREFIX}${body}`
  const tokenHash = await sha256Hex(token)
  const prefix = token.slice(0, TOKEN_PREFIX.length + 8) // e.g. 'eq677_abcdef12'
  const ins = await env.DB.prepare(
    `INSERT INTO api_tokens (user_id, name, token_hash, prefix)
       VALUES (?, ?, ?, ?)`,
  )
    .bind(userId, name || null, tokenHash, prefix)
    .run()
  return { id: ins.meta.last_row_id, token, prefix }
}

export async function logout(c) {
  const token = getCookie(c, SESSION_COOKIE)
  if (token) {
    await c.env.SESSIONS.delete(sessionKey(token))
    deleteCookie(c, SESSION_COOKIE, { path: '/' })
  }
  return c.redirect('/', 302)
}
