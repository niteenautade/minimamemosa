import { defineConfig } from '@playwright/test';

export default defineConfig({
  timeout: 60000,
  retries: 2,
  workers: 1,
  use: {
    baseURL: 'http://localhost:3000',
    headless: true,
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',
  },
  globalSetup: require.resolve('./e2e/global-setup'),
  globalTeardown: require.resolve('./e2e/global-teardown'),
  projects: [
    {
      name: 'auth-setup',
      testMatch: 'auth.setup.ts',
    },
    {
      name: 'auth',
      testMatch: 'auth.spec.ts',
    },
    {
      name: 'notes',
      testMatch: 'notes.spec.ts',
      dependencies: ['auth-setup'],
      use: { storageState: 'e2e/.auth/user.json' },
    },
    {
      name: 'features',
      testMatch: 'features.spec.ts',
      dependencies: ['auth-setup'],
      use: { storageState: 'e2e/.auth/user.json' },
    },
  ],
});
