import { mkdir, copyFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { spawn } from 'node:child_process'

const release = process.argv.includes('--release')
const root = process.cwd()
const manifestPath = join(root, 'src-tauri', 'Cargo.toml')
const resourceDir = join(root, 'src-tauri', 'target', 'agentbro-bridge-resource')
const exeSuffix = process.platform === 'win32' ? '.exe' : ''
const binaryName = `agentbro-bridge${exeSuffix}`
const profileDir = release ? 'release' : 'debug'
const source = join(root, 'src-tauri', 'target', profileDir, binaryName)
const dest = join(resourceDir, binaryName)
const legacyDest = join(resourceDir, 'agentbro-bridge')
const windowsDest = join(resourceDir, 'agentbro-bridge.exe')

await mkdir(resourceDir, { recursive: true })
await writeFile(legacyDest, '')
await writeFile(windowsDest, '')

await run('cargo', [
  'build',
  '--manifest-path',
  manifestPath,
  '--bin',
  'agentbro-bridge',
  ...(release ? ['--release'] : []),
])

await copyFile(source, dest)

function run(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: 'inherit', shell: process.platform === 'win32' })
    child.on('error', reject)
    child.on('exit', (code) => {
      if (code === 0) {
        resolve()
      } else {
        reject(new Error(`${command} exited with code ${code}`))
      }
    })
  })
}
