// Draws the 1024x1024 Murmur app icon (a microphone on a blue rounded square) and writes it
// as a PNG, with no image dependencies. Usage: node scripts/make-icon.mjs <out.png>
import { writeFileSync } from 'node:fs'
import { deflateSync } from 'node:zlib'

const SIZE = 1024
const out = process.argv[2] ?? 'app-icon.png'

const clamp01 = (v) => Math.min(1, Math.max(0, v))
const mix = (a, b, t) => a + (b - a) * t

function roundedRect(x, y, cx, cy, hw, hh, r) {
  const qx = Math.abs(x - cx) - (hw - r)
  const qy = Math.abs(y - cy) - (hh - r)
  return Math.hypot(Math.max(qx, 0), Math.max(qy, 0)) + Math.min(Math.max(qx, qy), 0) - r
}

function capsule(x, y, x1, y1, x2, y2, r) {
  const px = x - x1, py = y - y1, dx = x2 - x1, dy = y2 - y1
  const t = clamp01((px * dx + py * dy) / (dx * dx + dy * dy || 1))
  return Math.hypot(px - dx * t, py - dy * t) - r
}

function pixel(x, y) {
  const body = roundedRect(x, y, 512, 512, 512, 512, 230)
  const bodyCover = clamp01(0.5 - body)
  if (bodyCover === 0) return [0, 0, 0, 0]

  const t = y / SIZE
  let r = mix(0x5b, 0x2a, t), g = mix(0x9a, 0x55, t), b = mix(0xff, 0xdc, t)

  const mic = capsule(x, y, 512, 360, 512, 500, 108)
  const ring = Math.abs(Math.hypot(x - 512, y - 500) - 190) - 17
  const cradle = y > 500 ? ring : 1e9
  const stem = capsule(x, y, 512, 690, 512, 770, 17)
  const base = capsule(x, y, 420, 790, 604, 790, 17)
  const glyph = clamp01(0.5 - Math.min(mic, cradle, stem, base))

  r = mix(r, 255, glyph); g = mix(g, 255, glyph); b = mix(b, 255, glyph)
  return [Math.round(r), Math.round(g), Math.round(b), Math.round(255 * bodyCover)]
}

const raw = Buffer.alloc((SIZE * 4 + 1) * SIZE)
for (let y = 0; y < SIZE; y++) {
  raw[y * (SIZE * 4 + 1)] = 0
  for (let x = 0; x < SIZE; x++) {
    const [r, g, b, a] = pixel(x + 0.5, y + 0.5)
    const o = y * (SIZE * 4 + 1) + 1 + x * 4
    raw[o] = r; raw[o + 1] = g; raw[o + 2] = b; raw[o + 3] = a
  }
}

const crcTable = Array.from({ length: 256 }, (_, n) => {
  let c = n
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1
  return c >>> 0
})
const crc32 = (buf) => {
  let c = 0xffffffff
  for (const byte of buf) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8)
  return (c ^ 0xffffffff) >>> 0
}
const chunk = (type, data) => {
  const body = Buffer.concat([Buffer.from(type), data])
  const length = Buffer.alloc(4); length.writeUInt32BE(data.length)
  const crc = Buffer.alloc(4); crc.writeUInt32BE(crc32(body))
  return Buffer.concat([length, body, crc])
}
const header = Buffer.alloc(13)
header.writeUInt32BE(SIZE, 0); header.writeUInt32BE(SIZE, 4)
header[8] = 8; header[9] = 6; header[10] = 0; header[11] = 0; header[12] = 0

writeFileSync(out, Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', header),
  chunk('IDAT', deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]))
console.log(`wrote ${out}`)
