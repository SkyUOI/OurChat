import { test, expect } from '@playwright/test'

test.describe('Authentication', () => {
  test('protected route redirects to login without token', async ({ page }) => {
    await page.goto('/dashboard')
    await expect(page).toHaveURL(/.*login/)

    await page.goto('/monitor')
    await expect(page).toHaveURL(/.*login/)

    await page.goto('/users')
    await expect(page).toHaveURL(/.*login/)
  })

  test('can access protected routes with valid token', async ({ page }) => {
    await page.addScriptTag({
      content: `localStorage.setItem('token', 'fake-test-token');`,
    })

    await page.goto('/')
    await expect(page).not.toHaveURL(/.*login/)
  })

  test('login page renders correctly', async ({ page }) => {
    await page.goto('/login')

    await expect(page.locator('h2')).toContainText('Admin Login')
    await expect(page.locator('input')).toHaveCount(2)
    await expect(page.locator('button[type="submit"]')).toBeVisible()
  })
})
