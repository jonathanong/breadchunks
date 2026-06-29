const fs = require('node:fs')
const { join } = require('node:path')

const file = join(__dirname, 'index.js')
let c = fs.readFileSync(file, 'utf8')

const check = [
  "const nativeLibraryPath = process.env.NAPI_RS_NATIVE_LIBRARY_PATH",
  "if (!require('node:path').isAbsolute(nativeLibraryPath) || require('node:path').extname(nativeLibraryPath) !== '.node') {",
  "  throw new Error('NAPI_RS_NATIVE_LIBRARY_PATH must be an absolute path to a .node file')",
  '}',
].join('\n')
const marker = "if (process.env.NAPI_RS_NATIVE_LIBRARY_PATH) {"

if (c.includes(check)) {
  process.exit(0)
}

if (!c.includes(marker)) {
  console.error('Patch target missing')
  process.exit(1)
}

c = c.replace(marker, marker + '\n    ' + check)
fs.writeFileSync(file, c, 'utf8')
