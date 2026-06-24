import { createHmac } from 'crypto';
import { Page, BrowserContext } from '@playwright/test';

const SECRET = process.env.SESSION_SECRET || 'test-e2e-secret';

export function hmacSha256(payload: string): string {
  const h = createHmac('sha256', SECRET);
  h.update(payload);
  return h.digest('hex');
}

export function createCaptchaToken(answer: string): string {
  const expiry = Math.floor(Date.now() / 1000) + 300;
  const payload = `${answer}:${expiry}`;
  return `${payload}:${hmacSha256(payload)}`;
}

export async function injectCaptchaCookie(context: BrowserContext, answer = 'test123'): Promise<void> {
  const token = createCaptchaToken(answer);
  await context.addCookies([
    {
      name: 'captcha',
      value: token,
      domain: 'localhost',
      path: '/',
      httpOnly: false,
      sameSite: 'Lax',
    },
  ]);
}

export function randomUsername(): string {
  return `testuser_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`;
}
