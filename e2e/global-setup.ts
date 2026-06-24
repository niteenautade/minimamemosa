import * as fs from 'fs';
import * as path from 'path';
import * as cp from 'child_process';
import { createHmac } from 'crypto';

const TEST_DB_PATH = path.resolve(__dirname, '..', 'data', 'e2e-test.db');
const PROJECT_ROOT = path.resolve(__dirname, '..');
const PID_FILE = path.resolve(__dirname, '..', 'data', 'server.pid');
const SECRET = 'test-e2e-secret';

function hmacSha256(payload: string): string {
  const h = createHmac('sha256', SECRET);
  h.update(payload);
  return h.digest('hex');
}

function createCaptchaToken(answer: string): string {
  const expiry = Math.floor(Date.now() / 1000) + 300;
  const payload = `${answer}:${expiry}`;
  return `${payload}:${hmacSha256(payload)}`;
}

async function waitForServer(url: string, timeoutMs = 90000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const resp = await fetch(url);
      if (resp.ok || resp.status === 302) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`Server at ${url} did not start within ${timeoutMs}ms`);
}

export default async function globalSetup(): Promise<void> {
  const dataDir = path.dirname(TEST_DB_PATH);
  if (!fs.existsSync(dataDir)) fs.mkdirSync(dataDir, { recursive: true });

  // Remove old test DB so we start clean
  for (const f of fs.readdirSync(dataDir)) {
    if (f.startsWith('e2e-test.db')) {
      fs.unlinkSync(path.join(dataDir, f));
    }
  }

  // Kill any existing process on port 3000
  try {
    cp.execSync("lsof -ti:3000 | xargs kill -9 2>/dev/null || true", { stdio: 'ignore' });
  } catch {}

  // Start the Rust server with test env vars
  console.log('\nBuilding & starting MinimaMemosa server...');
  const server = cp.spawn('cargo', ['run'], {
    cwd: PROJECT_ROOT,
    env: { ...process.env, DATABASE_PATH: TEST_DB_PATH, SESSION_SECRET: SECRET },
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: false,
  });

  server.stdout?.on('data', (d: Buffer) => {
    const line = d.toString().trim();
    if (line) console.log(`  [server] ${line}`);
  });
  server.stderr?.on('data', (d: Buffer) => {
    const line = d.toString().trim();
    if (line) console.log(`  [server] ${line}`);
  });
  server.on('exit', (code) => console.log(`  [server] exited (${code})`));

  if (server.pid) fs.writeFileSync(PID_FILE, String(server.pid));

  // Wait for server
  console.log('Waiting for server...');
  await waitForServer('http://localhost:3000/login');
  console.log('Server is up. Registering test user...');

  // Register test user via HTTP (bypass captcha via HMAC cookie injection)
  const answer = 'test123';
  const captchaToken = createCaptchaToken(answer);
  const form = new URLSearchParams({
    username: 'e2euser',
    password: 'test1234',
    captcha_answer: answer,
  });

  const regResp = await fetch('http://localhost:3000/register', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Cookie: `captcha=${captchaToken}`,
    },
    body: form.toString(),
    redirect: 'manual',
  });

  if (regResp.status === 302) {
    console.log('Test user registered: e2euser / test1234\n');
  } else {
    const text = await regResp.text();
    console.log(`Registration: HTTP ${regResp.status} — ${text.substring(0, 150)}\n`);
  }
}
