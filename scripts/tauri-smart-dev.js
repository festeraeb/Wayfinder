#!/usr/bin/env node
// Smart dev launcher (ESM): reuse existing Vite with Wayfinder fingerprint or start a new one, then run tauri dev with TAURI_DEV_URL set.

import { spawn } from 'child_process';
import http from 'http';

const FINGERPRINT = 'WAYFINDER_DEV_FINGERPRINT';
const PREFERRED_PORTS = [5173, 5174, 5175, 5180, 5181];
// Use IPv4 loopback to avoid IPv6-only resolution on some Windows setups.
const HOST = process.env.WAYFINDER_DEV_HOST || '127.0.0.1';
const VITE_CMD = ['npm', 'run', 'dev', '--', '--strictPort', '--host', HOST, '--port'];
const TAURI_CMD = ['npx', 'tauri'];

const wait = (ms) => new Promise((res) => setTimeout(res, ms));

const fetchPort = (port, timeoutMs = 500) => new Promise((resolve) => {
  const req = http.get({ host: HOST, port, path: '/', timeout: timeoutMs }, (res) => {
    let data = '';
    res.on('data', (chunk) => { data += chunk.toString(); })
       .on('end', () => resolve({ ok: true, status: res.statusCode, body: data }));
  });
  req.on('error', () => resolve({ ok: false }));
  req.on('timeout', () => { req.destroy(); resolve({ ok: false }); });
});

async function choosePort() {
  for (const port of PREFERRED_PORTS) {
    const res = await fetchPort(port);
    if (!res.ok) return { port, reuse: false };
    if (res.status === 200 && res.body && res.body.includes(FINGERPRINT)) {
      return { port, reuse: true };
    }
  }
  return { port: PREFERRED_PORTS[PREFERRED_PORTS.length - 1] + 1, reuse: false };
}

async function ensureVite(port) {
  const res = await fetchPort(port);
  if (res.ok && res.status === 200 && res.body && res.body.includes(FINGERPRINT)) return;

  spawn(VITE_CMD[0], [...VITE_CMD.slice(1), String(port)], { stdio: 'inherit', shell: process.platform === 'win32' });

  const maxWaitMs = 20000;
  const start = Date.now();
  while (Date.now() - start < maxWaitMs) {
    const check = await fetchPort(port);
    if (check.ok && check.status === 200) {
      if (!check.body || !check.body.includes(FINGERPRINT)) {
        console.warn(`[warn] Port ${port} responded but fingerprint missing; continuing.`);
      }
      return;
    }
    await wait(400);
  }
  console.error(`[error] Vite did not start on port ${port} within ${maxWaitMs}ms.`);
  process.exit(1);
}

async function run() {
  const args = process.argv.slice(2);
  if (args.length && args[0] !== 'dev') {
    const tauri = spawn(TAURI_CMD[0], [...TAURI_CMD.slice(1), ...args], { stdio: 'inherit', shell: process.platform === 'win32' });
    tauri.on('exit', (code) => process.exit(code ?? 0));
    return;
  }

  const { port, reuse } = await choosePort();
  if (reuse) {
    console.log(`[info] Reusing Vite at http://${HOST}:${port}`);
  } else {
    console.log(`[info] Starting Vite at http://${HOST}:${port}`);
    await ensureVite(port);
  }

  const env = { ...process.env, TAURI_DEV_URL: `http://${HOST}:${port}` };
  const tauri = spawn(TAURI_CMD[0], [...TAURI_CMD.slice(1), 'dev'], { stdio: 'inherit', env, shell: process.platform === 'win32' });
  tauri.on('exit', (code) => process.exit(code ?? 0));
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
