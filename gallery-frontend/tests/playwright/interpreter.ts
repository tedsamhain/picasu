import type { Page, Locator } from '@playwright/test'
import { expect } from '@playwright/test'
import { UiWhenItem, UiThenItem } from './types'
import { GivenContext } from './executeGiven'

function resolveLocator(page: Page, roleLabel: string): Locator {
  const slashIdx = roleLabel.indexOf('/')
  const role = roleLabel.slice(0, slashIdx) as any
  const name = slashIdx === -1 ? undefined : roleLabel.slice(slashIdx + 1) || undefined
  return name ? page.getByRole(role, { name }) : page.getByRole(role)
}

export async function executeWhen(
  page: Page,
  when: UiWhenItem[],
  ctx: GivenContext
): Promise<void> {
  for (const step of when) {
    if ('navigate' in step) {
      await page.goto(interpolate(step.navigate, ctx.vars))
    } else if ('click' in step) {
      await resolveLocator(page, step.click).click()
    } else if ('fill' in step) {
      await resolveLocator(page, step.fill).fill(step.value)
    } else if ('submit' in step) {
      await page.keyboard.press('Enter')
    }
  }
}

export async function executeThen(
  page: Page,
  then: UiThenItem[],
  ctx: GivenContext
): Promise<void> {
  for (const assertion of then) {
    if ('ui.visible' in assertion) {
      await expect(resolveLocator(page, assertion['ui.visible'])).toBeVisible()
    } else if ('ui.hidden' in assertion) {
      await expect(resolveLocator(page, assertion['ui.hidden'])).not.toBeVisible()
    } else if ('ui.text' in assertion && 'contains' in assertion) {
      await expect(resolveLocator(page, assertion['ui.text'])).toContainText(
        interpolate(assertion.contains, ctx.vars)
      )
    } else if ('ui.route' in assertion) {
      await expect(page).toHaveURL(new RegExp(interpolate(assertion['ui.route'], ctx.vars)))
    } else if ('ui.modal' in assertion) {
      const dialog = page.getByRole('dialog')
      if (assertion['ui.modal'] === 'open') {
        await expect(dialog).toBeVisible()
      } else {
        await expect(dialog).not.toBeVisible()
      }
    }
  }
}

function interpolate(value: string, vars: Record<string, string>): string {
  return value.replace(/\$\{(\w+)\}/g, (_, key) => vars[key] ?? '')
}
