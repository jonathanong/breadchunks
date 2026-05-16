import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const { chunk } = await import('../index.js')

const __dirname = dirname(fileURLToPath(import.meta.url))
const fixturesDir = join(__dirname, '../../fixtures')

function readFixture(name) {
  return readFileSync(join(fixturesDir, name), 'utf8')
}

// ---------------------------------------------------------------------------
// Async + string inputs (basic parity with old API)
// ---------------------------------------------------------------------------

test('empty string returns no chunks', async () => {
  const [chunks] = await chunk([''])
  assert.equal(chunks.length, 0)
})

test('no headers returns single level-0 chunk', async () => {
  const [chunks] = await chunk(['Hello, world!'])
  assert.equal(chunks.length, 1)
  assert.equal(chunks[0].level, 0)
  assert.equal(chunks[0].breadcrumb, '')
})

test('single header chunk has correct level and header', async () => {
  const [chunks] = await chunk(['# My Header\n\nSome content.'])
  assert(chunks.length >= 1)
  const h = chunks.find((c) => c.header === 'My Header')
  assert(h)
  assert.equal(h.level, 1)
  assert.equal(h.breadcrumb, 'My Header')
})

test('breadcrumb nests with > separator', async () => {
  const [chunks] = await chunk(['# H1\n\nA.\n\n## H2\n\nB.'], { phase: 1 })
  const h2 = chunks.find((c) => c.header === 'H2')
  assert(h2)
  assert.equal(h2.breadcrumb, 'H1 > H2')
})

test('code blocks with # inside are not treated as headers', async () => {
  const text = '# Real Header\n\n```\n# not a header\n```\n\nAfter code.'
  const [chunks] = await chunk([text])
  assert(!chunks.some((c) => c.breadcrumb.includes('not a header')))
  const combined = chunks.map((c) => c.text).join(' ')
  assert(combined.includes('# not a header'))
})

test('phase 1 only — no merging', async () => {
  const text = '# H\n\nA.\n\n# H\n\nB.'
  const [[p1], [full]] = await Promise.all([
    chunk([text], { phase: 1 }),
    chunk([text]),
  ])
  assert(p1.length >= full.length)
})

test('phase 2 merges same-breadcrumb small chunks', async () => {
  const text = '# H\n\nA.\n\n# H\n\nB.'
  const [[p1], [p2]] = await Promise.all([
    chunk([text], { phase: 1 }),
    chunk([text], { phase: 2, minLength: 1000, maxLength: 10000 }),
  ])
  assert(p2.length < p1.length)
})

test('phase 3 parent absorbs small child', async () => {
  const [chunks] = await chunk(
    ['# Parent\n\nP.\n\n## Child\n\nC.'],
    { minLength: 10000, maxLength: 100000 },
  )
  assert.equal(chunks.length, 1)
  assert(chunks[0].text.includes('## Child'))
})

test('title option sets headers[0]', async () => {
  const [chunks] = await chunk(['No headers here.'], { title: 'My Doc' })
  assert.equal(chunks[0].headers[0], 'My Doc')
  assert.equal(chunks[0].breadcrumb, 'My Doc')
})

test('whitespace-only input returns no chunks', async () => {
  const [chunks] = await chunk(['   \n\n  '])
  assert.equal(chunks.length, 0)
})

test('chunk fields are correctly typed', async () => {
  const [chunks] = await chunk(['# H1\n\nContent.'])
  const c = chunks[0]
  assert.equal(typeof c.level, 'number')
  assert.equal(typeof c.breadcrumb, 'string')
  assert.equal(typeof c.text, 'string')
  assert.equal(typeof c.length, 'number')
  assert(Array.isArray(c.headers))
  assert(c.headers.length === 6)
})

// ---------------------------------------------------------------------------
// Buffer inputs
// ---------------------------------------------------------------------------

test('Buffer input produces the same result as string input', async () => {
  const text = '# H1\n\nContent.\n\n## H2\n\nMore.'
  const [fromStr] = await chunk([text])
  const [fromBuf] = await chunk([Buffer.from(text, 'utf8')])
  assert.deepEqual(fromBuf, fromStr)
})

test('mixed Buffer and string in one batch', async () => {
  const a = '# A\n\nText A.'
  const b = '# B\n\nText B.'
  const [ra, rb] = await chunk([Buffer.from(a, 'utf8'), b])
  assert(ra.some((c) => c.header === 'A'))
  assert(rb.some((c) => c.header === 'B'))
})

test('invalid UTF-8 Buffer rejects', async () => {
  const bad = Buffer.from([0xff, 0xfe, 0x00])
  await assert.rejects(chunk([bad]), /UTF-8/)
})

// ---------------------------------------------------------------------------
// Batch processing
// ---------------------------------------------------------------------------

test('empty batch resolves to empty array', async () => {
  const result = await chunk([])
  assert.deepEqual(result, [])
})

test('batch returns one entry per input in order', async () => {
  const docs = [
    '# Doc One\n\nFirst.',
    '# Doc Two\n\nSecond.',
    '# Doc Three\n\nThird.',
  ]
  const results = await chunk(docs)
  assert.equal(results.length, 3)
  assert(results[0].some((c) => c.header === 'Doc One'))
  assert(results[1].some((c) => c.header === 'Doc Two'))
  assert(results[2].some((c) => c.header === 'Doc Three'))
})

test('batch with shared options applies to all inputs', async () => {
  // Use ## (h2) so the title slot (h1) is not overwritten by a document heading
  const docs = ['## A\n\nShort.', '## B\n\nAlso short.']
  const results = await chunk(docs, { title: 'Suite' })
  for (const chunks of results) {
    assert(chunks[0].breadcrumb.startsWith('Suite'))
  }
})

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

test('fixture: tech-guide produces nested breadcrumbs', async () => {
  const text = readFixture('tech-guide.md')
  // Use phase 1 — default phase 3 absorbs small sub-sections into the parent,
  // collapsing all nested breadcrumbs on a short fixture like this one.
  const [chunks] = await chunk([text], { phase: 1 })
  assert(chunks.length > 0)
  assert(chunks.some((c) => c.breadcrumb.includes(' > ')))
})

test('fixture: tech-guide code blocks preserved', async () => {
  const text = readFixture('tech-guide.md')
  const [chunks] = await chunk([text])
  const combined = chunks.map((c) => c.text).join('\n')
  assert(combined.includes('brew install toolbox'))
  assert(!chunks.some((c) => c.breadcrumb.includes('not a header')))
})

test('fixture: deeply-nested collapses with large limits', async () => {
  const text = readFixture('deeply-nested.md')
  const [chunks] = await chunk([text], { minLength: 100000, maxLength: 1000000 })
  assert(chunks.length <= 5)
})

test('fixture: gettysburg has expected structure', async () => {
  const text = readFixture('gettysburg.md')
  const [chunks] = await chunk([text])
  assert(chunks.length > 0)
  assert(chunks.some((c) => c.breadcrumb.includes('Background') || c.text.includes('1863')))
})

test('fixture: batch of all fixtures returns correct count', async () => {
  const names = ['tech-guide.md', 'deeply-nested.md', 'gettysburg.md', 'recipe.md', 'code-heavy.md']
  const inputs = names.map((n) => readFixture(n))
  const results = await chunk(inputs)
  assert.equal(results.length, names.length)
  for (const chunks of results) {
    assert(chunks.length > 0)
  }
})
