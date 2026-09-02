#!/usr/bin/env node
// Backfill fiber_matrix_{symmetric,normal,rank} for magmas already in the
// database (migration 0011 adds the columns as NULL).
//
// Reads the public manifest, downloads each table that still lacks a value,
// computes the properties with the same code the Worker uses, and writes a SQL
// file of UPDATE statements. Apply it with:
//
//   node scripts/backfill-fiber-matrix.mjs [--all] [--out backfill.sql]
//   npx wrangler d1 execute eq677 --remote --file backfill.sql
//
// --all recomputes every magma, not only those with fiber_matrix_rank = NULL
// (useful if the definition ever changes). Right-cancellative magmas are
// handled by the O(1) shortcut in fiberMatrixProperties; the rest take up to
// a few seconds each at size ~1000, so the whole run is on the order of half
// an hour.

import fs from 'node:fs'
import { parseText, fiberMatrixProperties } from '../src/magma.js'

const MANIFEST_URL = 'https://eq677.icarm.cloud/manifest.json'
const args = process.argv.slice(2)
const all = args.includes('--all')
const outIdx = args.indexOf('--out')
const outPath = outIdx >= 0 ? args[outIdx + 1] : 'backfill-fiber-matrix.sql'
const CONCURRENCY = 4

const manifest = await (await fetch(MANIFEST_URL)).json()
const todo = manifest.magmas.filter((m) => all || m.fiber_matrix_rank === null || m.fiber_matrix_rank === undefined)
console.error(`${todo.length} of ${manifest.magmas.length} magmas to process`)

const lines = []
let done = 0
let next = 0
async function worker() {
  while (next < todo.length) {
    const m = todo[next++]
    const text = await (await fetch(m.url)).text()
    const parsed = parseText(text)
    if (parsed.error) throw new Error(`${m.canonical_hash}: ${parsed.error}`)
    if (parsed.table.length !== m.size) throw new Error(`${m.canonical_hash}: size mismatch`)
    const p = fiberMatrixProperties(parsed.table, m.right_cancellative === true)
    lines.push(
      `UPDATE magmas SET fiber_matrix_symmetric = ${p.symmetric ? 1 : 0}, fiber_matrix_normal = ${p.normal ? 1 : 0}, fiber_matrix_rank = ${p.rank} WHERE canonical_hash = '${m.canonical_hash}';`,
    )
    done++
    if (done % 100 === 0) console.error(`${done}/${todo.length}`)
    if (!p.normal) console.error(`NOTE: non-normal fiber matrix: ${m.canonical_hash} (size ${m.size})`)
  }
}
await Promise.all(Array.from({ length: CONCURRENCY }, worker))
fs.writeFileSync(outPath, lines.join('\n') + '\n')
console.error(`wrote ${lines.length} statements to ${outPath}`)
