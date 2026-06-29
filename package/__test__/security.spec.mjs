import { test } from 'node:test'
import assert from 'node:assert/strict'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { join, dirname, resolve } from 'node:path'
import { tmpdir } from 'node:os'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))

test('NAPI_RS_NATIVE_LIBRARY_PATH must be an absolute path to a .node file', () => {
  const indexJsPath = join(__dirname, '../index.js')
  const fixtureDir = mkdtempSync(join(tmpdir(), 'breadchunks-native-library-path-'))
  const fixturePath = resolve(fixtureDir, 'malicious.js')
  writeFileSync(fixturePath, 'module.exports = {}\n')

  let threw = false
  let output = ''
  try {
    execFileSync(process.execPath, ['-e', `require(${JSON.stringify(indexJsPath)})`], {
      env: {
        ...process.env,
        NAPI_RS_NATIVE_LIBRARY_PATH: fixturePath,
      },
      encoding: 'utf8',
      stdio: 'pipe',
    })
  } catch (err) {
    threw = true
    output = err.stderr || err.stdout || err.message
  } finally {
    rmSync(fixtureDir, { recursive: true, force: true })
  }

  assert.equal(threw, true, 'Should have thrown an error')
  assert.match(output, /NAPI_RS_NATIVE_LIBRARY_PATH must be an absolute path to a \.node file/)
})
