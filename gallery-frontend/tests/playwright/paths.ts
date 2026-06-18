import { fileURLToPath } from 'url'
import * as path from 'path'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const GALLERY_FRONTEND = path.resolve(__dirname, '..')
const REPO_ROOT = path.resolve(GALLERY_FRONTEND, '..')

export const E2E_DIR = path.resolve(REPO_ROOT, 'sandbox', 'e2e')
export const CONFIG_DIR = path.join(E2E_DIR, 'config')
export const DATA_DIR = path.join(E2E_DIR, 'data')
export const IMAGE_HOME = path.join(E2E_DIR, 'images')
export const BACKEND_URL = 'http://localhost:5673'
export const FRONTEND_URL = 'http://localhost:5173'
export const ADMIN_PASSWORD = 'admin'
