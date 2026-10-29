// Wayfinder end-to-end: Lemonade setup check, chat, embeddings + semantic search on a small real folder,
// the save prompter firing on a real file save, and the commit nudge. Prints one PASS/FAIL line per step.
//
// Modes (WF_MODE):
//   cdp    Windows (or any OS with WebView2/CDP): launch the Tauri exe with
//          WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 and drive it with
//          chromium.connectOverCDP. Needs WF_EXE.
//   bridge Linux: run the real Rust command handlers behind wf-bridge (cargo build --features bridge --bin
//          wf-bridge) and drive the built front end in Chrome. The webview shell is the only part not under
//          test. Needs WF_BRIDGE_BIN and WF_DIST.
//
// Env: LEMONADE_URL (base incl. /api), CHAT_MODEL, EMBED_MODEL, WF_KEEP=1 to keep the temp folders.
import { chromium } from "playwright-core";
import { spawn, execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const MODE = process.env.WF_MODE || "bridge";
const LEM = (process.env.LEMONADE_URL || "http://127.0.0.1:13305/api").replace(/\/+$/, "");
const CHAT_MODEL = process.env.CHAT_MODEL || "";
const EMBED_MODEL = process.env.EMBED_MODEL || "nomic-embed-text-v1-GGUF";
const BRIDGE_PORT = Number(process.env.WF_BRIDGE_PORT || 14321);
const CDP_PORT = Number(process.env.WF_CDP_PORT || 9222);

const results = [];
async function step(name, fn) {
  const t0 = Date.now();
  try {
    await fn();
    results.push({ name, ok: true });
    console.log(`PASS ${name} (${Date.now() - t0} ms)`);
  } catch (e) {
    results.push({ name, ok: false, error: String(e.message || e).split("\n")[0] });
    console.log(`FAIL ${name}: ${String(e.message || e).split("\n")[0]}`);
  }
}
const assert = (c, m) => { if (!c) throw new Error(m); };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---- fixtures: a small folder of real notes, and a git repo with an old commit --------------------
const root = fs.mkdtempSync(path.join(os.tmpdir(), "wf-e2e-"));
const notes = path.join(root, "notes");
const repo = path.join(root, "repo");
fs.mkdirSync(notes); fs.mkdirSync(repo);
const NOTES = {
  "photosynthesis.md": "# Photosynthesis\nPlants use chlorophyll in their leaves to capture sunlight. Light energy turns carbon dioxide and water into glucose and releases oxygen. The light reactions happen in the thylakoids and the Calvin cycle in the stroma.\n",
  "mitochondria.md": "# Cell respiration\nMitochondria are the powerhouse of the cell. They break down glucose with oxygen to make ATP, the energy currency of the cell. The Krebs cycle runs in the matrix.\n",
  "ww1-causes.md": "# Causes of the First World War\nMilitarism, alliances, imperialism and nationalism pushed Europe toward war. The assassination of Archduke Franz Ferdinand in Sarajevo in 1914 was the spark.\n",
  "quadratics.md": "# Quadratic equations\nA quadratic equation ax^2 + bx + c = 0 is solved with the quadratic formula x = (-b +- sqrt(b^2 - 4ac)) / 2a. The discriminant tells you how many real roots there are.\n",
  "french-verbs.md": "# Verbes francais\nLes verbes du premier groupe se terminent en -er. Au present: je parle, tu parles, il parle, nous parlons, vous parlez, ils parlent.\n",
};
for (const [f, t] of Object.entries(NOTES)) fs.writeFileSync(path.join(notes, f), t);

// ---- launch ---------------------------------------------------------------------------------------
let proc = null;
let browser, page, baseUrl = null;
const extraEnv = { WAYFINDER_LLM_ENDPOINT: LEM, ...(CHAT_MODEL ? { WAYFINDER_LLM_MODEL: CHAT_MODEL } : {}), WAYFINDER_EMBED_MODEL: EMBED_MODEL };

async function waitHttp(url, ms) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    try { const r = await fetch(url); if (r.status < 500) return; } catch {}
    await sleep(300);
  }
  throw new Error(`timeout waiting for ${url}`);
}

async function launch() {
  if (MODE === "cdp") {
    const exe = process.env.WF_EXE;
    assert(exe, "WF_EXE (path to the Tauri exe) is required in cdp mode");
    proc = spawn(exe, [], {
      env: { ...process.env, ...extraEnv, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${CDP_PORT}` },
      stdio: "ignore", windowsHide: false,
    });
    await waitHttp(`http://127.0.0.1:${CDP_PORT}/json/version`, 60000);
    browser = await chromium.connectOverCDP(`http://127.0.0.1:${CDP_PORT}`);
    const ctx = browser.contexts()[0];
    for (let i = 0; i < 60 && ctx.pages().length === 0; i++) await sleep(500);
    page = ctx.pages()[0];
    assert(page, "no WebView2 page appeared");
  } else {
    const bin = process.env.WF_BRIDGE_BIN, dist = process.env.WF_DIST;
    assert(bin && dist, "WF_BRIDGE_BIN and WF_DIST are required in bridge mode");
    proc = spawn(bin, [String(BRIDGE_PORT)], { env: { ...process.env, ...extraEnv, WF_DIST: dist }, stdio: "inherit" });
    baseUrl = `http://127.0.0.1:${BRIDGE_PORT}`;
    await waitHttp(baseUrl + "/", 30000);
    browser = await chromium.launch({ executablePath: process.env.CHROME_PATH || "/usr/bin/google-chrome", headless: true, args: ["--no-sandbox"] });
    page = await browser.newPage();
    // Tauri's invoke() goes through window.__TAURI_INTERNALS__; point it at the bridge.
    await page.addInitScript(() => {
      let cb = 1;
      window.__TAURI_INTERNALS__ = {
        metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
        transformCallback: () => cb++,
        unregisterCallback: () => {},
        convertFileSrc: (p) => p,
        invoke: async (cmd, args) => {
          if (cmd.startsWith("plugin:")) throw `${cmd} is not available in the bridge`;
          const r = await fetch(`/invoke/${cmd}`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(args || {}) });
          const body = await r.json();
          if (!r.ok) throw body;
          return body;
        },
      };
    });
    await page.goto(baseUrl + "/");
  }
  await page.waitForSelector("[data-testid=nav-lemonade]", { timeout: 30000 });
}

const nav = (id) => page.click(`[data-testid=nav-${id}]`);

try {
  await launch();
} catch (e) {
  console.log(`FAIL launch: ${e.message}`);
  results.push({ name: "launch", ok: false, error: e.message });
}

if (page) {
  // 1. Lemonade setup check against the real server
  await step("lemonade setup check passes against a real server", async () => {
    await nav("lemonade");
    await page.fill("[data-testid=lemonade-base]", LEM);
    if (CHAT_MODEL) await page.fill("[data-testid=lemonade-chat-model-input]", CHAT_MODEL);
    await page.fill("[data-testid=lemonade-embed-model-input]", EMBED_MODEL);
    await page.click("[data-testid=lemonade-check]");
    await page.waitForSelector("[data-testid=lemonade-report], [data-testid=lemonade-check-error]", { timeout: 120000 });
    const rep = page.locator("[data-testid=lemonade-report]");
    if (!(await rep.count())) throw new Error("check returned an error: " + (await page.locator("[data-testid=lemonade-check-error]").innerText()));
    const probs = (await page.locator("[data-testid=lemonade-problems]").allInnerTexts()).join(" | ");
    assert((await rep.getAttribute("data-all-ok")) === "true", "setup check not ok: " + probs);
    for (const id of ["models", "chat-model", "embed-model", "embed-dim", "chat-reply"]) {
      assert((await page.locator(`[data-testid=lemonade-${id}]`).getAttribute("data-ok")) === "true", `row ${id} not ok`);
    }
  });

  // 2. The check must say so when the server is wrong (no false green)
  await step("lemonade setup check fails honestly for a dead address", async () => {
    await page.fill("[data-testid=lemonade-base]", "http://127.0.0.1:9/api");
    await page.click("[data-testid=lemonade-check]");
    await page.waitForFunction(() => document.querySelector("[data-testid=lemonade-report]")?.getAttribute("data-all-ok") === "false", null, { timeout: 60000 });
    const probs = await page.locator("[data-testid=lemonade-problems]").innerText();
    assert(/not reachable/i.test(probs), "problem text missing: " + probs);
    await page.fill("[data-testid=lemonade-base]", LEM);
  });

  // 3. Chat through the same server
  await step("chat gets a reply from Lemonade", async () => {
    await page.fill("[data-testid=chat-input]", "Reply with one short sentence about mitochondria.");
    await page.click("[data-testid=chat-send]");
    await page.waitForSelector("[data-testid=chat-reply], [data-testid=chat-error]", { timeout: 180000 });
    if (await page.locator("[data-testid=chat-error]").count()) throw new Error(await page.locator("[data-testid=chat-error]").innerText());
    const txt = (await page.locator("[data-testid=chat-reply]").innerText()).trim();
    assert(txt.length > 3, "empty chat reply");
    console.log("   chat reply:", JSON.stringify(txt.slice(0, 160)));
  });

  // 4. Scan a small real folder
  await step("scan indexes the notes folder", async () => {
    await nav("scan");
    await page.fill("[data-testid=scan-path]", notes);
    await page.click("[data-testid=scan-start]");
    await page.waitForFunction(() => /scan complete|files? found|✅/i.test(document.body.innerText) || document.querySelector("[data-testid=error-banner]"), null, { timeout: 60000 });
    if (await page.locator("[data-testid=error-banner]").count()) throw new Error(await page.locator("[data-testid=error-banner]").innerText());
    const idx = path.join(notes, ".wayfinder_index", "index.json");
    for (let i = 0; i < 40 && !fs.existsSync(idx); i++) await sleep(250);
    const files = JSON.parse(fs.readFileSync(idx, "utf8")).files;
    assert(files.length === Object.keys(NOTES).length, `index has ${files.length} files, want ${Object.keys(NOTES).length}`);
  });

  // 5. Embeddings through Lemonade (UI defaults must already point at Lemonade)
  await step("embeddings: UI defaults point at Lemonade", async () => {
    await nav("embed");
    const ep = await page.locator('.form-group:has(label:has-text("Server endpoint")) input').inputValue();
    const md = await page.locator('.form-group:has(label:has-text("Local Model")) input').inputValue();
    assert(/\/api\/?$/.test(ep) && ep.includes("13305"), `endpoint default is ${ep}, expected the local Lemonade on :13305/api`);
    assert(/nomic-embed-text/.test(md), `embedding model default is ${md}`);
  });
  await step("embeddings generated for every file", async () => {
    await page.fill('.form-group:has(label:has-text("Server endpoint")) input', LEM);
    await page.fill('.form-group:has(label:has-text("Local Model")) input', EMBED_MODEL);
    await page.click("[data-testid=embed-start]");
    await page.waitForSelector("[data-testid=embed-result], [data-testid=error-banner]", { timeout: 240000 });
    if (await page.locator("[data-testid=error-banner]").count()) throw new Error(await page.locator("[data-testid=error-banner]").innerText());
    const emb = JSON.parse(fs.readFileSync(path.join(notes, ".wayfinder_index", "embeddings.json"), "utf8"));
    const dims = emb.embeddings.map((e) => (e.embedding || []).length);
    assert(dims.length === Object.keys(NOTES).length, `${dims.length} embeddings on disk`);
    assert(dims.every((d) => d > 0 && d === dims[0]), "bad dims " + dims);
    assert(emb.model === EMBED_MODEL, `embeddings.json model is ${emb.model}`);
    console.log("   embedding dim:", dims[0], "model:", emb.model);
  });

  // 6. Semantic search: the right note comes first even when the query shares few words with it
  for (const [q, want] of [
    ["how do plants turn sunlight into sugar", "photosynthesis.md"],
    ["what started the 1914 European war", "ww1-causes.md"],
    ["finding the roots of x squared equations", "quadratics.md"],
  ]) {
    await step(`semantic search "${q}" -> ${want}`, async () => {
      await nav("search");
      await page.fill("[data-testid=search-query]", q);
      await page.click("[data-testid=search-go]");
      // The mode line carries the query that produced the results on screen.
      await page.waitForFunction((qq) => document.querySelector("[data-testid=search-mode]")?.getAttribute("data-query") === qq, q, { timeout: 60000 });
      assert((await page.locator("[data-testid=search-mode]").getAttribute("data-mode")) === "semantic", "search fell back to keyword: " + (await page.locator("[data-testid=search-mode]").innerText()));
      const names = (await page.locator("[data-testid=search-result] .result-name").allInnerTexts());
      console.log("   ranking:", names.join(" > "));
      assert(names[0] === want, `top result was ${names[0]}`);
    });
  }

  // 7. Save prompter: a real file save in the watched folder
  await step("save prompter fires for a placeholder-named save", async () => {
    // The scan set the index, which starts the watcher on the notes folder (Status shows Running).
    await nav("status");
    await page.waitForSelector(".save-prompts", { timeout: 30000 });
    await sleep(1500);
    fs.writeFileSync(path.join(notes, "Untitled (3).md"), "# lab notes\nenzymes speed up reactions\n");
    await page.waitForSelector(".save-prompts__card", { timeout: 40000 });
    const card = await page.locator(".save-prompts__card").first().innerText();
    assert(/Untitled \(3\)\.md/.test(card), "card is for another file: " + card);
    assert(/untitled\.md/i.test(card), "no suggested name in card: " + card);
    console.log("   clippy:", JSON.stringify(card.split("\n")[0]));
  });
  await step("save prompter stays quiet for a well-named save", async () => {
    const before = await page.locator(".save-prompts__card").count();
    fs.writeFileSync(path.join(notes, "enzyme-lab-report.md"), "# Enzyme lab\nCatalase breaks down hydrogen peroxide.\n");
    await sleep(9000); // two poll ticks (4 s each)
    const cards = await page.locator(".save-prompts__card").allInnerTexts();
    assert(cards.length === before && !cards.some((c) => c.includes("enzyme-lab-report.md")), "prompted for a clean name");
  });

  // 8. Commit nudge through Git Clippy
  await step("commit nudge is loud for an old commit and a pile of files", async () => {
    const git = (...a) => execFileSync("git", a, { cwd: repo, env: { ...process.env, GIT_COMMITTER_DATE: "2020-01-01T00:00:00Z", GIT_AUTHOR_DATE: "2020-01-01T00:00:00Z" } });
    git("init", "-q");
    fs.writeFileSync(path.join(repo, "a.txt"), "a");
    git("add", ".");
    git("-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "old work");
    for (let i = 0; i < 25; i++) fs.writeFileSync(path.join(repo, `new${i}.txt`), "x");
    await nav("git");
    await page.fill("[data-testid=git-repo-path]", repo);
    await page.waitForSelector("[data-testid=commit-nudge]", { timeout: 60000 });
    const cls = await page.locator("[data-testid=commit-nudge]").getAttribute("class");
    assert(/nudge-loud/.test(cls), "nudge class " + cls);
  });
}

// ---- teardown + summary ---------------------------------------------------------------------------
try { await browser?.close(); } catch {}
if (proc) { try { proc.kill(); } catch {} }
if (!process.env.WF_KEEP) { try { fs.rmSync(root, { recursive: true, force: true }); } catch {} }
const failed = results.filter((r) => !r.ok);
console.log(`\n${results.length - failed.length}/${results.length} passed on ${MODE} (${os.platform()})`);
console.log("RESULTS_JSON " + JSON.stringify(results));
process.exit(failed.length ? 1 : 0);
