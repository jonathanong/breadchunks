'use strict'

const { existsSync, readFileSync } = require('fs')
const { join } = require('path')

const { platform, arch } = process

let nativeBinding = null
let loadError = null

function isMusl() {
  if (!existsSync('/usr/bin/ldd')) {
    return true
  }
  return readFileSync('/usr/bin/ldd', 'utf8').includes('musl')
}

const platformMap = {
  'darwin x64': 'breadchunks.darwin-x64.node',
  'darwin arm64': 'breadchunks.darwin-arm64.node',
  'linux x64': isMusl()
    ? 'breadchunks.linux-x64-musl.node'
    : 'breadchunks.linux-x64-gnu.node',
  'linux arm64': isMusl()
    ? 'breadchunks.linux-arm64-musl.node'
    : 'breadchunks.linux-arm64-gnu.node',
  'win32 x64': 'breadchunks.win32-x64-msvc.node',
}

const localFile = platformMap[`${platform} ${arch}`]

if (localFile && existsSync(join(__dirname, localFile))) {
  try {
    nativeBinding = require(join(__dirname, localFile))
  } catch (err) {
    loadError = err
  }
}

if (!nativeBinding) {
  try {
    nativeBinding = require('./breadchunks.node')
  } catch (err) {
    loadError = loadError || err
  }
}

if (!nativeBinding) {
  throw new Error(
    `Failed to load breadchunks native module. Platform: ${platform} ${arch}\n${loadError}`
  )
}

module.exports = nativeBinding
