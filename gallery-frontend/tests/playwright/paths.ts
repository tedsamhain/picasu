import * as fs from 'fs'
import { fileURLToPath } from 'url'
import * as path from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

// gallery-frontend/tests/playwright/ → gallery-frontend/tests/ → gallery-frontend/ → repo root
const REPO_ROOT = path.resolve(__dirname, '..', '..', '..')

// Generate fresh e2e identifiers on first load in the main process.
// Workers inherit these via process.env (child_process.fork copies the
// parent's environment), so every process in the same run agrees.
const runId: string = (() => {
  const existing = process.env.UROCISSA_E2E_RUN_ID
  if (existing) return existing
  const id = Math.random().toString(36).slice(2, 8)
  process.env.UROCISSA_E2E_RUN_ID = id
  return id
})()

const port: number = (() => {
  const existing = process.env.UROCISSA_E2E_PORT
  if (existing) return Number(existing)
  const p = 30000 + Math.floor(Math.random() * 30000)
  process.env.UROCISSA_E2E_PORT = String(p)
  return p
})()

export const E2E_DIR = path.resolve(REPO_ROOT, '.testruns', `playwright-${runId}`)
export const CONFIG_DIR = path.join(E2E_DIR, 'config')
export const DATA_DIR = path.join(E2E_DIR, 'data')
export const IMAGE_HOME = path.join(E2E_DIR, 'images')
export const BACKEND_PORT = port
export const BACKEND_URL = `http://localhost:${port}`
export const FRONTEND_URL = 'http://localhost:5173'
export const ADMIN_PASSWORD = 'e2e_test_pwd'
