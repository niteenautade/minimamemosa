import { test as setup, expect } from '@playwright/test';

const AUTH_FILE = 'e2e/.auth/user.json';

setup('authenticate as e2euser', async ({ page }) => {
  await page.goto('/login');
  await page.fill('#username', 'e2euser');
  await page.fill('#password', 'test1234');
  await page.click('button[type="submit"]');
  await page.waitForURL(/\/app/);
  await page.context().storageState({ path: AUTH_FILE });
});
