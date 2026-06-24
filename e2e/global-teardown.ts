import * as fs from 'fs';
import * as path from 'path';
import * as cp from 'child_process';

const PID_FILE = path.resolve(__dirname, '..', 'data', 'server.pid');

export default async function globalTeardown(): Promise<void> {
  // Kill the server via PID file
  if (fs.existsSync(PID_FILE)) {
    const pid = parseInt(fs.readFileSync(PID_FILE, 'utf-8').trim(), 10);
    if (pid) {
      try {
        process.kill(pid, 'SIGTERM');
        console.log(`Killed server process ${pid}`);
      } catch (e) {
        console.log(`Could not kill pid ${pid}: ${e}`);
      }
    }
    fs.unlinkSync(PID_FILE);
  }

  // Also kill any leftover Rust process on port 3000
  try {
    cp.execSync("lsof -ti:3000 | xargs kill -9 2>/dev/null || true", { stdio: 'ignore' });
  } catch {}
}
