/**
 * macOS Chrome smoke test (Metal ANGLE).
 * Prefer `npm run verify:view` — see scripts/README.md.
 */
import { chromium } from "playwright";

const URL = "http://127.0.0.1:8080";
const TIMEOUT_MS = 180_000;

const consoleErrors = [];
const pageErrors = [];
const browser = await chromium.launch({
  channel: "chrome",
  headless: true,
  args: ["--enable-unsafe-webgpu", "--use-angle=metal"],
});
const page = await browser.newPage();
page.on("console", (msg) => {
  if (msg.type() === "error") consoleErrors.push(msg.text());
});
page.on("pageerror", (err) => pageErrors.push(String(err)));

await page.goto(URL, { waitUntil: "domcontentloaded", timeout: TIMEOUT_MS });
await page.waitForFunction(
  () => {
    const loading = document.getElementById("gravitium-loading");
    const noWebgpu = document.getElementById("gravitium-no-webgpu");
    return loading?.hasAttribute("hidden") && noWebgpu?.hasAttribute("hidden");
  },
  { timeout: TIMEOUT_MS },
);
await page.waitForTimeout(3000);

const state = await page.evaluate(() => {
  const canvas = document.getElementById("gravitium-canvas");
  return {
    webgpu: !!navigator.gpu,
    canvasWidth: canvas?.width ?? 0,
    canvasHeight: canvas?.height ?? 0,
    title: document.title,
  };
});

await page.screenshot({ path: "/tmp/gravitium-chrome.png" });
await browser.close();

const fatal = [...consoleErrors, ...pageErrors].filter(
  (e) => /panicked|wgpu|WebGPU|surface|fatal/i.test(e) && !/favicon|404/i.test(e),
);
const ok =
  state.webgpu && state.canvasWidth > 0 && state.canvasHeight > 0 && fatal.length === 0;

console.log(JSON.stringify({ name: "chrome", ok, state, fatal }, null, 2));
process.exit(ok ? 0 : 1);
