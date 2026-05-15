import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

// Load the native binding built by `npm run build`
const { chunk } = await import('../index.js')

const __dirname = dirname(fileURLToPath(import.meta.url))
const fixturesDir = join(__dirname, '../../fixtures')

function readFixture(name) {
  return readFileSync(join(fixturesDir, name), 'utf8')
}

test('empty string returns one empty chunk', () => {
  const chunks = chunk('')
  assert.equal(chunks.length, 1)
  assert.equal(chunks[0].text, '')
})

test('no headers returns single level-0 chunk', () => {
  const chunks = chunk('Hello, world!')
  assert.equal(chunks.length, 1)
  assert.equal(chunks[0].level, 0)
  assert.equal(chunks[0].breadcrumb, '')
})

test('single header chunk has correct level and header', () => {
  const chunks = chunk('# My Header\n\nSome content.')
  assert(chunks.length >= 1)
  const h = chunks.find((c) => c.header === 'My Header')
  assert(h)
  assert.equal(h.level, 1)
  assert.equal(h.breadcrumb, 'My Header')
})

test('breadcrumb nests with > separator', () => {
  const chunks = chunk('# H1\n\nA.\n\n## H2\n\nB.', { phase: 1 })
  const h2 = chunks.find((c) => c.header === 'H2')
  assert(h2)
  assert.equal(h2.breadcrumb, 'H1 > H2')
})

test('code blocks with # inside are not treated as headers', () => {
  const text = '# Real Header\n\n```\n# not a header\n```\n\nAfter code.'
  const chunks = chunk(text)
  assert(!chunks.some((c) => c.breadcrumb.includes('not a header')))
  const combined = chunks.map((c) => c.text).join(' ')
  assert(combined.includes('# not a header'))
})

test('phase 1 only — no merging', () => {
  const text = '# H\n\nA.\n\n# H\n\nB.'
  const p1 = chunk(text, { phase: 1 })
  const full = chunk(text)
  assert(p1.length >= full.length)
})

test('phase 2 merges same-breadcrumb small chunks', () => {
  const text = '# H\n\nA.\n\n# H\n\nB.'
  const p1 = chunk(text, { phase: 1 })
  const p2 = chunk(text, { phase: 2, minLength: 1000, maxLength: 10000 })
  assert(p2.length < p1.length)
})

test('phase 3 parent absorbs small child', () => {
  const text = '# Parent\n\nP.\n\n## Child\n\nC.'
  const chunks = chunk(text, { minLength: 10000, maxLength: 100000 })
  assert.equal(chunks.length, 1)
  assert(chunks[0].text.includes('## Child'))
})

test('title option sets headers[0]', () => {
  const chunks = chunk('No headers here.', { title: 'My Doc' })
  assert.equal(chunks[0].headers[0], 'My Doc')
  assert.equal(chunks[0].breadcrumb, 'My Doc')
})

test('whitespace-only input returns no chunks', () => {
  const chunks = chunk('   \n\n  ')
  assert.equal(chunks.length, 0)
})

test('chunk fields are correctly typed', () => {
  const chunks = chunk('# H1\n\nContent.')
  const c = chunks[0]
  assert.equal(typeof c.level, 'number')
  assert.equal(typeof c.breadcrumb, 'string')
  assert.equal(typeof c.text, 'string')
  assert.equal(typeof c.length, 'number')
  assert(Array.isArray(c.headers))
  assert(c.headers.length === 6)
})

test('fixture: tech-guide produces nested breadcrumbs', () => {
  const text = readFixture('tech-guide.md')
  const chunks = chunk(text)
  assert(chunks.length > 0)
  assert(chunks.some((c) => c.breadcrumb.includes(' > ')))
})

test('fixture: tech-guide code blocks preserved', () => {
  const text = readFixture('tech-guide.md')
  const chunks = chunk(text)
  const combined = chunks.map((c) => c.text).join('\n')
  assert(combined.includes('brew install toolbox'))
  assert(!chunks.some((c) => c.breadcrumb.includes('not a header')))
})

test('fixture: deeply-nested collapses with large limits', () => {
  const text = readFixture('deeply-nested.md')
  const chunks = chunk(text, { minLength: 100000, maxLength: 1000000 })
  assert(chunks.length <= 5)
})

test('fixture: gettysburg has expected structure', () => {
  const text = readFixture('gettysburg.md')
  const chunks = chunk(text)
  assert(chunks.length > 0)
  assert(chunks.some((c) => c.breadcrumb.includes('Background') || c.text.includes('1863')))
})
