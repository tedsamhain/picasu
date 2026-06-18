import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'
import * as yaml from 'js-yaml'
import { UiScenario } from './types'

const DIR = path.dirname(fileURLToPath(import.meta.url))
const SCENARIOS_DIR = path.resolve(DIR, '../../../xtask/data/scenarios/ui')

export function loadAllScenarios(): UiScenario[] {
  if (!fs.existsSync(SCENARIOS_DIR)) {
    console.warn(`UI scenarios directory not found: ${SCENARIOS_DIR}`)
    return []
  }

  const files = fs.readdirSync(SCENARIOS_DIR).filter(f => f.endsWith('.yaml'))
  const scenarios: UiScenario[] = []

  for (const file of files) {
    const filePath = path.join(SCENARIOS_DIR, file)
    const raw = fs.readFileSync(filePath, 'utf-8')
    const doc = yaml.load(raw) as Record<string, unknown>
    const parsed = UiScenario.parse(doc)
    scenarios.push(parsed)
  }

  return scenarios
}
