import { chromium } from 'playwright';
import { createHmac } from 'crypto';
import { execSync } from 'child_process';
import { readFileSync } from 'fs';

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
  // Register user
  const captchaToken = createCaptchaToken('test123');
  const regResp = await fetch(`${BASE}/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded', Cookie: `captcha=${captchaToken}` },
    body: new URLSearchParams({ username: `ss_${Date.now()}`, password: 'test1234', captcha_answer: 'test123' }),
    redirect: 'manual',
  });
  const sessionToken = ((await regResp.headers.get('set-cookie') || '').match(/session=([^;]+)/) || [])[1];
  if (!sessionToken) { console.error('reg failed'); process.exit(1); }

  // Create memos with tags — first one is public for sharing
  const memos = [
    { content: '# Welcome!\n\nA lightweight #memoa app. **Fast** and _minimal_.', visibility: 'public' },
    { content: '## Shopping\n\n- Milk\n- Eggs\n- Bread\n\n#groceries #essentials' },
    { content: '### Ideas\n\n1. Build a CLI tool\n2. Write a blog post\n3. Contribute to OSS\n#ideas #rust' },
    { content: 'Meeting notes\n\nDiscussed Q2 roadmap\n#work #meetings' },
    { content: '## Code\n\n```rust\nfn main() {\n    println!("Hello!");\n}\n```\n\n#rust #coding' },
  ];
  let firstMemoId: number | null = null;
  for (const { content, visibility } of memos) {
    const params: Record<string, string> = { content };
    if (visibility) params.visibility = visibility;
    const resp = await fetch(`${BASE}/memos`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded', Cookie: `session=${sessionToken}` },
      body: new URLSearchParams(params),
      redirect: 'manual',
    });
    if (!firstMemoId) {
      const html = await resp.clone().text();
      const match = html.match(/id="memo-(\d+)"/);
      if (match) firstMemoId = parseInt(match[1]);
    }
  }

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 900 }, baseURL: BASE });
  const page = await context.newPage();

  // === 1. LOGIN PAGE ===
  await page.goto('/login', { waitUntil: 'networkidle' });
  await page.evaluate(() => localStorage.setItem('theme', 'dark'));
  await page.goto('/login', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1500);
  await page.screenshot({ path: 'screenshots/01-login-dark.png' });
  console.log('✓ 01-login-dark.png');

  // Authenticate
  await context.addCookies([{
    name: 'session', value: sessionToken,
    domain: 'localhost', path: '/', httpOnly: true, sameSite: 'Lax' as const,
  }]);

  // === 2. TIMELINE with editor + memos ===
  await page.goto('/app', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);
  await page.screenshot({ path: 'screenshots/02-timeline-dark.png' });
  console.log('✓ 02-timeline-dark.png');

  // === 3. PLUS MENU EXPANDED ===
  await page.goto('/app', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);
  const plusBtn = page.locator('button[onclick*="togglePlusMenu"]').first();
  if (await plusBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
    await plusBtn.click();
    await page.waitForTimeout(800);
  }
  await page.screenshot({ path: 'screenshots/03-plus-menu-dark.png' });
  console.log('✓ 03-plus-menu-dark.png');

  // === 4. SINGLE NOTE VIEW ===
  const noteItem = page.locator('#notes-list-container [data-note-id]').first();
  if (await noteItem.isVisible({ timeout: 5000 }).catch(() => false)) {
    await noteItem.click();
    await page.waitForTimeout(2000);
  }
  await page.screenshot({ path: 'screenshots/04-note-detail-dark.png' });
  console.log('✓ 04-note-detail-dark.png');

  // === 4. CALENDAR EXPANDED ===
  await page.goto('/app', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);
  // Expand calendar section
  const calendarBtn = page.locator('button[onclick*="calendar-section"]').first();
  if (await calendarBtn.isVisible()) {
    await calendarBtn.click();
    await page.waitForTimeout(1000);
  }
  await page.screenshot({ path: 'screenshots/05-calendar-dark.png' });
  console.log('✓ 05-calendar-dark.png');

  // === 6. TAGS EXPANDED + FILTERED ===
  // First expand the tags section
  const tagsBtn = page.locator('button[onclick*="tags-section"]').first();
  if (await tagsBtn.isVisible()) {
    await tagsBtn.click();
    await page.waitForTimeout(1000);
  }
  // Click on the #rust tag to filter
  const rustTag = page.locator('button:has-text("#rust")').first();
  if (await rustTag.isVisible({ timeout: 3000 }).catch(() => false)) {
    await rustTag.click();
    await page.waitForTimeout(2000);
  }
  await page.screenshot({ path: 'screenshots/06-tag-filter-dark.png' });
  console.log('✓ 06-tag-filter-dark.png');

  // === 7. RESOURCES PANEL ===
  await page.goto('/app/resources', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'screenshots/07-resources-dark.png' });
  console.log('✓ 07-resources-dark.png');

  // === 8. SHARED PUBLIC NOTE (no session cookie) ===
  if (firstMemoId) {
    const shareCtx = await browser.newContext({ viewport: { width: 1440, height: 900 }, baseURL: BASE });
    const sharePg = await shareCtx.newPage();
    await sharePg.goto('/login', { waitUntil: 'networkidle' });
    await sharePg.evaluate(() => localStorage.setItem('theme', 'dark'));
    await sharePg.goto(`/share/${firstMemoId}`, { waitUntil: 'networkidle' });
    await sharePg.waitForTimeout(1500);
    await sharePg.screenshot({ path: 'screenshots/08-share-note-dark.png' });
    console.log('✓ 08-share-note-dark.png');
    await shareCtx.close();
  }

  // === 9. TIMELINE scrolled to memos ===
  await page.goto('/app', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);
  await page.evaluate(() => {
    const editor = document.getElementById('editor-section');
    if (editor) editor.scrollIntoView({ block: 'start' });
    window.scrollBy(0, 350);
  });
  await page.waitForTimeout(500);
  await page.screenshot({ path: 'screenshots/09-memos-scroll-dark.png' });
  console.log('✓ 09-memos-scroll-dark.png');

  // === 10. LIGHT MODE TIMELINE ===
  await page.evaluate(() => {
    localStorage.setItem('theme', 'light');
    document.documentElement.classList.remove('dark');
    const h = [75,70,65,60,55,50,45,40,35,30];
    const L = [0.95,0.90,0.82,0.72,0.62,0.55,0.48,0.42,0.35,0.28];
    const C = [0.025,0.04,0.07,0.09,0.11,0.12,0.13,0.11,0.09,0.07];
    const names = ['--a50','--a100','--a200','--a300','--a400','--a500','--a600','--a700','--a800','--a900'];
    for (let i = 0; i < 10; i++)
      document.documentElement.style.setProperty(names[i], 'oklch('+L[i]+' '+C[i]+' '+h[i]+')');
  });
  await page.goto('/app', { waitUntil: 'networkidle' });
  await page.waitForTimeout(3000);
  await page.screenshot({ path: 'screenshots/10-timeline-light.png' });
  console.log('✓ 10-timeline-light.png');

  // === 11. SETTINGS MODAL with Forest theme ===
  await page.goto('/app', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);
  // Open settings
  const settingsBtn = page.locator('button[onclick*="openSettings"]').first();
  if (await settingsBtn.isVisible()) {
    await settingsBtn.click();
    await page.waitForTimeout(800);
  }
  // Click Forest theme
  const forestTheme = page.locator('#settings-theme-list [data-theme="Forest"]').first();
  if (await forestTheme.isVisible({ timeout: 3000 }).catch(() => false)) {
    await forestTheme.click();
    await page.waitForTimeout(500);
  }
  await page.screenshot({ path: 'screenshots/11-settings-light.png' });
  console.log('✓ 11-settings-light.png');

  // ── Add captions to each screenshot ──
  const captions: Record<string, string> = {
    '01-login-dark.png': '🔐 Dark login screen',
    '02-timeline-dark.png': '📝 Timeline & editor',
    '03-plus-menu-dark.png': '➕ Plus menu expanded',
    '04-note-detail-dark.png': '📄 Single note view',
    '05-calendar-dark.png': '📅 Calendar expanded',
    '06-tag-filter-dark.png': '🏷️ Tag filtering active',
    '07-resources-dark.png': '📎 Resources panel',
    '08-share-note-dark.png': '🔗 Shared public note',
    '09-memos-scroll-dark.png': '📜 Timeline scrolled',
    '10-timeline-light.png': '☀️ Light mode timeline',
    '11-settings-light.png': '⚙️ Settings & Forest theme',
  };
  for (const [file, caption] of Object.entries(captions)) {
    const imgPath = `screenshots/${file}`;
    const base64 = readFileSync(imgPath).toString('base64');
    const html = `<!DOCTYPE html>
<html><head><meta charset="utf-8">
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{display:flex;flex-direction:column;width:fit-content;background:transparent;font-family:system-ui,-apple-system,sans-serif}
img{display:block;max-width:100%;height:auto}
.caption{background:#1a1a2e;color:#e0e0e0;padding:14px 20px;font-size:18px;font-weight:600;letter-spacing:0.3px;text-align:center;border-top:1px solid #2a2a4e}
</style></head><body>
<img src="data:image/png;base64,${base64}">
<div class="caption">${caption}</div>
</body></html>`;
    const cp = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    await cp.setContent(html, { waitUntil: 'networkidle' });
    const { width, height } = await cp.evaluate(() => ({
      width: Math.max((document.querySelector('img') as HTMLImageElement).offsetWidth, (document.querySelector('.caption') as HTMLElement).offsetWidth),
      height: (document.querySelector('img') as HTMLImageElement).offsetHeight + (document.querySelector('.caption') as HTMLElement).offsetHeight,
    }));
    await cp.setViewportSize({ width: Math.ceil(width), height: Math.ceil(height) });
    await cp.waitForTimeout(200);
    await cp.screenshot({ path: imgPath });
    await cp.close();
    console.log(`  caption → ${file}`);
  }

  await browser.close();

  // ── Generate GIF ──
  execSync(
    'ffmpeg -y -framerate 1 -pattern_type glob -i \'screenshots/[0-9][0-9]*.png\' '
    + '-vf "scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=256[p];[s1][p]paletteuse=dither=none" '
    + '-loop 0 screenshots/demo.gif',
    { stdio: 'inherit' }
  );
  console.log('✓ screenshots/demo.gif');
}

main().catch(console.error);
