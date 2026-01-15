import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { Bench } from 'tinybench'
import linguist from 'linguist-js'
import { analyzeDirectory } from '../index.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
const projectRoot = join(__dirname, '..')

const bench = new Bench({ time: 1000 })

bench
  .add('linguist-js', async () => {
    await linguist(projectRoot, { offline: true })
  })
  .add('linguisto (native)', async () => {
    await analyzeDirectory(projectRoot)
  })

console.log('Running benchmark...')
await bench.run()

console.table(bench.table())
