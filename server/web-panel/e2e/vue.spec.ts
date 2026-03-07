import { test, expect } from '@playwright/test'

test('visits the app root url redirects to login when not authenticated', async ({
  page,
}) => {
  await page.goto('/')

  await expect(page).toHaveURL(/.*login/)
  await expect(page.locator('h2')).toContainText('Admin Login')
})
