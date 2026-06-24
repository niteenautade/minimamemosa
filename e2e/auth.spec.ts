import { test, expect } from '@playwright/test';
import { injectCaptchaCookie, randomUsername } from './helpers';

test.describe('Authentication Flows', () => {
  test('should display login page', async ({ page }) => {
    await page.goto('/login');
    await expect(page.locator('h1')).toContainText('MinimaMemosa');
    await expect(page.locator('button[type="submit"]')).toContainText('Sign In');
    await expect(page.locator('a[href="/register"]')).toContainText('Register');
  });

  test('should display registration page', async ({ page }) => {
    await page.goto('/register');
    await expect(page.locator('h1')).toContainText('Create Account');
    await expect(page.locator('button[type="submit"]')).toContainText('Register');
    await expect(page.locator('img[alt="Captcha"]')).toBeVisible();
  });

  test('should register a new user', async ({ page, context }) => {
    const captchaAnswer = 'test123';
    await page.goto('/register');
    // Inject captcha cookie AFTER page loads, so it overwrites the server's cookie
    await injectCaptchaCookie(context, captchaAnswer);

    await page.fill('#username', randomUsername());
    await page.fill('#password', 'test1234');
    await page.fill('#captcha_answer', captchaAnswer);
    await page.click('button[type="submit"]');

    await page.waitForURL(/\/app/);
    await expect(page.locator('.avatar-initials').first()).toBeVisible();
  });

  test('should reject registration with wrong captcha', async ({ page, context }) => {
    await page.goto('/register');
    await injectCaptchaCookie(context, 'correctAnswer');

    await page.fill('#username', randomUsername());
    await page.fill('#password', 'test1234');
    await page.fill('#captcha_answer', 'wrongAnswer');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Incorrect or expired captcha')).toBeVisible();
  });

  test('should reject registration with short password', async ({ page, context }) => {
    await page.goto('/register');
    await injectCaptchaCookie(context);

    await page.fill('#username', randomUsername());
    await page.fill('#password', '12');
    await page.fill('#captcha_answer', 'test123');
    await page.click('button[type="submit"]');

    await expect(page.getByText('must be at least 4 characters')).toBeVisible();
  });

  test('should login with pre-seeded user', async ({ page }) => {
    await page.goto('/login');
    await page.fill('#username', 'e2euser');
    await page.fill('#password', 'test1234');
    await page.click('button[type="submit"]');

    await page.waitForURL(/\/app/);
    await expect(page.locator('.avatar-initials').first()).toContainText('E');
  });

  test('should reject login with wrong password', async ({ page }) => {
    await page.goto('/login');
    await page.fill('#username', 'e2euser');
    await page.fill('#password', 'wrongpassword');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Invalid credentials')).toBeVisible();
  });

  test('should logout', async ({ page }) => {
    await page.goto('/login');
    await page.fill('#username', 'e2euser');
    await page.fill('#password', 'test1234');
    await page.click('button[type="submit"]');
    await page.waitForURL(/\/app/);

    await page.click('a[href="/logout"]');
    await page.waitForURL(/\/login/);
    await expect(page.locator('h1')).toContainText('MinimaMemosa');
  });
});
