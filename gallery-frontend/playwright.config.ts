import { defineConfig } from '@playwright/test'
import * as path from 'path'
import * as os from 'os'
import { fileURLToPath } from 'url'
import { E2E_DIR, BACKEND_PORT } from './tests/playwright/paths'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const isCI = !!process.env.CI

export default defineConfig({
  testDir: './tests/playwright',
  fullyParallel: true,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: isCI ? 1 : undefined,
  outputDir: path.join(E2E_DIR, 'artifacts'),
  reporter: [
    ['list'],
    ['json', { outputFile: path.join(E2E_DIR, 'report.json') }],
    ['html', { outputFolder: path.join(E2E_DIR, 'html-report'), open: 'never' }]
  ],

  use: {
    baseURL: `http://localhost:${BACKEND_PORT}`,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    launchOptions: {
      executablePath: isCI
        ? undefined
        : path.join(
            os.homedir(),
            '.cache',
            'ms-playwright',
            'chromium-1228',
            'chrome-linux64',
            'chrome'
          ),
      env: {
        ...process.env,
        LD_LIBRARY_PATH: '/tmp/nss-libs/usr/lib/x86_64-linux-gnu'
      }
    }
  },

  webServer: [
    {
      command: 'cargo run --bin urocissa',
      cwd: path.resolve(__dirname, '..', 'gallery-backend'),
      port: BACKEND_PORT,
      reuseExistingServer: false,
      env: {
        UROCISSA_CONFIG_HOME: path.join(E2E_DIR, 'config'),
        UROCISSA_DATA_HOME: path.join(E2E_DIR, 'data'),
        UROCISSA_IMAGE_HOME: path.join(E2E_DIR, 'images'),
        UROCISSA_PORT: String(BACKEND_PORT)
      },
      timeout: 180000
    }
  ]
})
