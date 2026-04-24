// Minimal USTAR writer for streaming downloads.

const ENC = new TextEncoder()

function writeString(buf, offset, str, maxLen) {
  const bytes = ENC.encode(str)
  const n = Math.min(bytes.length, maxLen)
  for (let i = 0; i < n; i++) buf[offset + i] = bytes[i]
}

function writeOctal(buf, offset, value, fieldLen) {
  // octal + trailing null, right-justified, zero-padded
  const s = value.toString(8).padStart(fieldLen - 1, '0')
  writeString(buf, offset, s, fieldLen - 1)
  buf[offset + fieldLen - 1] = 0
}

export function tarHeader(name, size) {
  const h = new Uint8Array(512)
  if (name.length > 100) {
    throw new Error(`tar filename too long: ${name}`)
  }
  writeString(h, 0, name, 100)
  writeOctal(h, 100, 0o644, 8)
  writeOctal(h, 108, 0, 8) // uid
  writeOctal(h, 116, 0, 8) // gid
  writeOctal(h, 124, size, 12)
  writeOctal(h, 136, Math.floor(Date.now() / 1000), 12)
  // checksum placeholder: spaces
  for (let i = 0; i < 8; i++) h[148 + i] = 0x20
  h[156] = 0x30 // typeflag '0' = regular file
  writeString(h, 257, 'ustar', 6)
  h[263] = 0x30
  h[264] = 0x30 // version "00"
  let sum = 0
  for (let i = 0; i < 512; i++) sum += h[i]
  writeOctal(h, 148, sum, 7)
  h[155] = 0x20
  return h
}

export function padding(size) {
  const rem = size % 512
  return rem === 0 ? new Uint8Array(0) : new Uint8Array(512 - rem)
}

export const endOfArchive = () => new Uint8Array(1024)
