import { test, expect, Page } from '@playwright/test';

test.describe('Notes & UI Flows', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/app');
  });

  async function waitForTiptap(page: Page): Promise<void> {
    await page.waitForFunction(() => {
      const ed = (window as any).tiptapEditor;
      return ed && typeof ed.commands === 'object';
    });
  }

  async function typeInEditor(page: Page, text: string): Promise<void> {
    await page.locator('.ProseMirror').first().focus();
    await page.keyboard.type(text, { delay: 20 });
  }

  async function createNote(page: Page, text: string): Promise<void> {
    await waitForTiptap(page);
    await typeInEditor(page, text);
    await expect(page.locator('#save-memo-btn')).toBeEnabled();
    await page.click('#save-memo-btn');
    await page.waitForTimeout(1000);
  }

  test('should display the timeline view with editor and save button', async ({ page }) => {
    await waitForTiptap(page);
    await expect(page.locator('#memo-form')).toBeVisible();
    await expect(page.locator('#save-memo-btn')).toBeVisible();
    await expect(page.locator('[href="/logout"]')).toBeVisible();
  });

  test('should create a new note', async ({ page }) => {
    await createNote(page, 'Hello from E2E test');
    await expect(page.locator('#timeline').getByText('Hello from E2E test')).toBeVisible();
  });

  test('should create a note with tags', async ({ page }) => {
    await createNote(page, 'This note has #rust and #testing tags');
    await expect(page.locator('#timeline').getByText('note has')).toBeVisible();
  });

  test('should edit an existing note', async ({ page }) => {
    await createNote(page, 'Original content for edit test');

    // Hover to reveal action buttons
    const memo = page.locator('#timeline [id^="memo-"]').first();
    await memo.hover();

    // Click edit
    await memo.locator('button[onclick*="editMemo"]').click();

    // Wait for edit form to load with its own editor
    await page.waitForFunction(() => {
      const ef = document.querySelector('[id^="memo-edit-form-"]');
      return ef && getComputedStyle(ef).display !== 'none';
    });

    // Give Tiptap time to init inside the edit form
    await page.waitForTimeout(1500);

    // Edit the content by clearing and retyping
    await waitForTiptap(page);
    const proseMirrors = page.locator('.ProseMirror');
    const count = await proseMirrors.count();
    // If there are multiple editors, pick the one in the edit form
    if (count > 1) {
      await proseMirrors.nth(count - 1).focus();
    } else {
      await proseMirrors.first().focus();
    }
    await page.keyboard.press('Meta+a');
    await page.keyboard.press('Control+a');
    await page.keyboard.type('Edited content for verification', { delay: 15 });

    await page.waitForTimeout(500);

    // Click Save on edit form
    const saveBtn = page.locator('[id^="save-memo-edit-btn-"]').first();
    await expect(saveBtn).toBeEnabled({ timeout: 5000 });
    await saveBtn.click();
    await page.waitForTimeout(1000);

    await expect(page.locator('#timeline').getByText('Edited content for verification')).toBeVisible();
  });

  test('should open emoji picker and insert an emoji', async ({ page }) => {
    await waitForTiptap(page);

    await page.locator('button[onclick*="toggleEmojiPicker"]').first().click();
    await expect(page.locator('#emoji-picker')).toBeVisible();

    const grid = page.locator('#emoji-grid');
    await expect(grid).toBeVisible();
    const count = await grid.locator('button').count();
    expect(count).toBeGreaterThan(0);

    await grid.locator('button').first().click();
    await expect(page.locator('#emoji-picker')).not.toBeVisible();

    await page.click('#save-memo-btn');
    await page.waitForTimeout(1000);
  });

  test('should open plus menu with all options', async ({ page }) => {
    await waitForTiptap(page);

    await page.locator('button[onclick*="togglePlusMenu"]').first().click();
    await expect(page.locator('#plus-menu')).toBeVisible();
    await expect(page.locator('#plus-menu')).toContainText('Upload Image');
    await expect(page.locator('#plus-menu')).toContainText('Upload File');
    await expect(page.locator('#plus-menu')).toContainText('Record Audio');
    await expect(page.locator('#plus-menu')).toContainText('Link Note');

    await page.locator('.ProseMirror').first().click({ force: true });
    await expect(page.locator('#plus-menu')).not.toBeVisible();
  });

  test('should change note visibility to public', async ({ page }) => {
    await waitForTiptap(page);

    await page.locator('#memo-form button[onclick*="toggleVisDropdown"]').first().click();
    await expect(page.locator('#memo-form .vis-dropdown-menu')).toBeVisible();
    await page.locator('#memo-form button[data-vis-value="public"]').click();
    await expect(page.locator('#memo-form .vis-label')).not.toContainText('Private');
  });

  test('should change note visibility to protected with password', async ({ page }) => {
    await waitForTiptap(page);

    await page.locator('#memo-form button[onclick*="toggleVisDropdown"]').first().click();
    await page.locator('#memo-form button[data-vis-value="protected"]').click();
    await expect(page.locator('#vis-password-modal')).toBeVisible();
    await expect(page.locator('#vis-pwd-title')).toContainText('Password');

    await page.fill('#vis-pwd-input', 'mypassword');
    await page.fill('#vis-pwd-confirm', 'mypassword');
    await page.locator('button[onclick*="confirmVisPwd"]').click();
    await expect(page.locator('#vis-password-modal')).not.toBeVisible();
  });

  test('should toggle dark mode', async ({ page }) => {
    await waitForTiptap(page);

    const before = await page.evaluate(() =>
      document.documentElement.classList.contains('dark'),
    );

    await page.locator('header button[onclick*="toggleTheme"]').click();
    const after = await page.evaluate(() =>
      document.documentElement.classList.contains('dark'),
    );
    expect(after).toBe(!before);

    // restore
    await page.locator('header button[onclick*="toggleTheme"]').click();
  });

  test('should open settings modal and select accent theme', async ({ page }) => {
    await waitForTiptap(page);

    await page.locator('button[onclick*="openSettings"]').first().click();
    await expect(page.locator('#settings-modal')).toBeVisible();
    await expect(page.locator('#settings-theme-list')).toBeVisible();
    await expect(page.locator('#settings-theme-list')).toContainText('Default');
    await expect(page.locator('#settings-theme-list')).toContainText('Rose');

    await page.locator('#settings-theme-list [data-theme="Forest"]').click();
    await page.locator('button[onclick*="saveTheme"]').click();
    await expect(page.locator('#settings-modal')).not.toBeVisible();
  });

  test('should delete a note', async ({ page }) => {
    await createNote(page, 'Note to be deleted');

    // Hover to reveal buttons
    const memo = page.locator('#timeline [id^="memo-"]').first();
    await memo.hover();

    let dialogHandled = false;
    page.on('dialog', async (dialog) => {
      expect(dialog.message()).toContain('Delete');
      await dialog.accept();
      dialogHandled = true;
    });

    await memo.locator('button[onclick*="deleteMemo"]').click();
    await page.waitForTimeout(1500);
    expect(dialogHandled).toBe(true);
  });

  test('should switch to notes panel', async ({ page }) => {
    await createNote(page, 'Note in notes panel');

    await page.click('#icon-notes');
    await page.waitForURL(/\/app\/notes/);

    await expect(page.locator('#notes-panel')).toBeVisible();
    await expect(page.locator('#notes-panel')).toContainText('Notes');
  });

  test('should switch to resources panel', async ({ page }) => {
    await waitForTiptap(page);

    await page.click('#icon-resources');
    await page.waitForURL(/\/app\/resources/);

    await expect(page.locator('#resources-panel')).toBeVisible();
    await expect(page.locator('#resources-panel')).toContainText('Resources');
  });

  test('should show avatar with user initial', async ({ page }) => {
    await waitForTiptap(page);
    await expect(page.locator('.avatar-initials').first()).toContainText('E');
  });

  test('should see timeline date group labels', async ({ page }) => {
    await createNote(page, 'Note for date label test');
    await expect(page.locator('#timeline .uppercase').first()).toBeVisible();
  });

  test('should see the sidebar with search', async ({ page }) => {
    await createNote(page, 'Sidebar test note');

    await page.click('#icon-notes');
    await page.waitForURL(/\/app\/notes/);

    await expect(page.locator('#notes-panel input[placeholder*="Search"]')).toBeVisible();
  });
});
