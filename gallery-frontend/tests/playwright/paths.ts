import * as fs from 'fs'
import { fileURLToPath } from 'url'
import * as path from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

// gallery-frontend/tests/playwright/ → gallery-frontend/tests/ → gallery-frontend/ → repo root
const REPO_ROOT = path.resolve(__dirname, '..', '..', '..')

// The main process generates all run-scoped paths and stores them in
// process.env. Workers inherit these via child_process.fork, so every
// process in the same run agrees on directories and ports without
// sharing files or knowing the naming convention.
const E2E_DIR: string = (() => {
  const existing = process.env.TESTRUN_DIR
  if (existing) return existing
  const runId = Math.random().toString(36).slice(2, 8)
  const dir = path.resolve(REPO_ROOT, '.testruns', `playwright-${runId}`)
  process.env.TESTRUN_DIR = dir
  return dir
})()

const port: number = (() => {
  const existing = process.env.TESTRUN_PORT
  if (existing) return Number(existing)
  const p = 30000 + Math.floor(Math.random() * 30000)
  process.env.TESTRUN_PORT = String(p)
  return p
})()

export { E2E_DIR }
export const CONFIG_DIR = path.join(E2E_DIR, 'config')
export const DATA_DIR = path.join(E2E_DIR, 'data')
export const IMAGE_HOME = path.join(E2E_DIR, 'images')
export const BACKEND_PORT = port
export const BACKEND_URL = `http://localhost:${port}`
export const FRONTEND_URL = 'http://localhost:5173'
export const ADMIN_PASSWORD = 'e2e_test_pwd'
