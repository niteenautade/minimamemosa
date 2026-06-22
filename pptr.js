const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch();
  const page = await browser.newPage();
  page.on('console', msg => console.log('PAGE LOG:', msg.text()));
  page.on('pageerror', err => console.log('PAGE ERROR:', err.toString()));
  await page.goto(`file://${process.cwd()}/test-esm-no-bundle.html`, {waitUntil: 'networkidle0'});
  
  await page.focus('#memo-editor .ProseMirror').catch(e => console.log("Focus failed", e.message));
  if (await page.$('#memo-editor .ProseMirror')) {
      await page.keyboard.type('# Heading\n');
      const html = await page.evaluate(() => window.tiptapEditor.getHTML());
      console.log('HTML OUT:', html);
  }
  await browser.close();
})();
