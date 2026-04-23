const CRC_TABLE = (() => {
  const t = new Uint32Array(256)
  for (let n = 0; n < 256; n++) {
    let c = n
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
    }
    t[n] = c >>> 0
  }
  return t
})()

function crc32(bytes) {
  let c = 0xffffffff
  for (let i = 0; i < bytes.length; i++) {
    c = CRC_TABLE[(c ^ bytes[i]) & 0xff] ^ (c >>> 8)
  }
  return (c ^ 0xffffffff) >>> 0
}

async function deflate(bytes) {
  const stream = new Response(bytes).body.pipeThrough(new CompressionStream('deflate'))
  const buf = await new Response(stream).arrayBuffer()
  return new Uint8Array(buf)
}

function chunk(type, data) {
  const out = new Uint8Array(8 + data.length + 4)
  const dv = new DataView(out.buffer)
  dv.setUint32(0, data.length, false)
  for (let i = 0; i < 4; i++) out[4 + i] = type.charCodeAt(i)
  out.set(data, 8)
  dv.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)), false)
  return out
}

function concat(parts) {
  let len = 0
  for (const p of parts) len += p.length
  const out = new Uint8Array(len)
  let at = 0
  for (const p of parts) {
    out.set(p, at)
    at += p.length
  }
  return out
}

// Color for magma entry v out of n. HSL hue wheel gives good distinguishability.
function colorFor(v, n) {
  const h = (v / n) * 360
  const s = 0.72
  const l = 0.52
  const c = (1 - Math.abs(2 * l - 1)) * s
  const hp = h / 60
  const x = c * (1 - Math.abs((hp % 2) - 1))
  let r = 0
  let g = 0
  let b = 0
  if (hp < 1) { r = c; g = x }
  else if (hp < 2) { r = x; g = c }
  else if (hp < 3) { g = c; b = x }
  else if (hp < 4) { g = x; b = c }
  else if (hp < 5) { r = x; b = c }
  else { r = c; b = x }
  const m = l - c / 2
  return [
    Math.round((r + m) * 255),
    Math.round((g + m) * 255),
    Math.round((b + m) * 255),
  ]
}

export async function magmaToPng(table) {
  const n = table.length
  // truecolor RGB, 8 bits per channel. One filter byte per scanline.
  const raw = new Uint8Array(n * (1 + n * 3))
  let p = 0
  const palette = new Uint8Array(n * 3)
  for (let v = 0; v < n; v++) {
    const [r, g, b] = colorFor(v, n)
    palette[v * 3] = r
    palette[v * 3 + 1] = g
    palette[v * 3 + 2] = b
  }
  for (let y = 0; y < n; y++) {
    raw[p++] = 0 // filter: None
    const row = table[y]
    for (let x = 0; x < n; x++) {
      const idx = row[x] * 3
      raw[p++] = palette[idx]
      raw[p++] = palette[idx + 1]
      raw[p++] = palette[idx + 2]
    }
  }
  const compressed = await deflate(raw)

  const sig = new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])
  const ihdr = new Uint8Array(13)
  const dv = new DataView(ihdr.buffer)
  dv.setUint32(0, n, false)
  dv.setUint32(4, n, false)
  ihdr[8] = 8 // bit depth
  ihdr[9] = 2 // color type: truecolor
  ihdr[10] = 0
  ihdr[11] = 0
  ihdr[12] = 0

  return concat([
    sig,
    chunk('IHDR', ihdr),
    chunk('IDAT', compressed),
    chunk('IEND', new Uint8Array(0)),
  ])
}

export function parseCanonicalText(text) {
  const rows = text
    .replace(/,/g, ' ')
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
  return rows.map((line) => line.split(/\s+/).map((tok) => Number(tok)))
}
