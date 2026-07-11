import { chromium } from 'playwright';
import { createHmac } from 'crypto';

const BASE = 'http://localhost:3000';
const SECRET = 'test-e2e-secret';

function hmacSha256(payload: string) {
  return createHmac('sha256', SECRET).update(payload).digest('hex');
}

function createCaptchaToken(answer: string) {
  const expiry = Math.floor(Date.now() / 1000) + 300;
  return `${answer}:${expiry}:${hmacSha256(`${answer}:${expiry}`)}`;
}

async function main() {
  const captchaToken = createCaptchaToken('test123');
  const regResp = await fetch(`${BASE}/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded', Cookie: `captcha=${captchaToken}` },
    body: new URLSearchParams({ username: `debug_${Date.now()}`, password: 'test1234', captcha_answer: 'test123' }),
    redirect: 'manual',
  });
  const sessionToken = ((await regResp.headers.get('set-cookie') || '').match(/session=([^;]+)/) || [])[1];
  if (!sessionToken) { const t = await regResp.text(); console.error('Reg failed:', t.slice(0, 300)); process.exit(1); }

  console.log('Session:', sessionToken.slice(0, 40) + '...');

  // Create some memos
  for (const c of ['# First memo', '## Second memo', '### Third memo']) {
    await fetch(`${BASE}/memos`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded', Cookie: `session=${sessionToken}` },
      body: new URLSearchParams({ content: c }),
      redirect: 'manual',
    });
  }

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, baseURL: BASE });
  await context.addCookies([{
    name: 'session', value: sessionToken,
    domain: 'localhost', path: '/', httpOnly: true, sameSite: 'Lax' as const,
  }]);
  const page = await context.newPage();

  // Listen for HTMX loads
  page.on('response', resp => {
    if (resp.url().includes('/memos-feed')) console.log('HTMX /memos-feed loaded:', resp.status());
  });

  await page.goto('/app', { waitUntil: 'networkidle' });
  console.log('Page loaded, waiting for HTMX...');
  await page.waitForTimeout(4000);

  // Check if memos feed loaded
  const feedHtml = await page.evaluate(() => {
    const feed = document.getElementById('timeline') || document.querySelector('[id*="feed"], [id*="timeline"]');
    return feed ? feed.innerHTML.slice(0, 500) : 'No timeline element found';
  });
  console.log('Timeline HTML (first 500):', feedHtml);

  // Check all IDs that contain "memo"
  const memoIds = await page.evaluate(() =>
    Array.from(document.querySelectorAll('[id*="memo"], [id*="note"], [class*="memo"], [class*="note"]'))
      .map(el => `${el.tagName}#${el.id}.${el.className.slice(0, 40)}`)
  );
  console.log('Memo/note elements:', memoIds);

  await browser.close();
}

main().catch(console.error);
