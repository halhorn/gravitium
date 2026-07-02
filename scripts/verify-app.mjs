#!/usr/bin/env node
/**
 * Smoke test: app loads over WebGPU and produces a screenshot.
 *
 * Prerequisite: `trunk serve` on GRAVITIUM_BASE_URL (default http://127.0.0.1:8080)
 * Linux: run via `scripts/run-visual-verification.sh` (xvfb + lavapipe).
 */
import { BASE_URL, TIMEOUT_MS, VIEWPORT } from "./lib/config.mjs";
import { attachErrorCollectors, fatalErrors, launchWebGpuBrowser } from "./lib/browser.mjs";
import { readSimulationState, waitForSimulation } from "./lib/simulation.mjs";
import { screenshotPath } from "./lib/screenshot.mjs";

const outPath = screenshotPath("verify-app.png");

const browser = await launchWebGpuBrowser();
const page = await browser.newPage({ viewport: VIEWPORT });
const { consoleErrors, pageErrors } = attachErrorCollectors(page);

await page.goto(BASE_URL, { waitUntil: "domcontentloaded", timeout: TIMEOUT_MS });
await waitForSimulation(page, 3000);

const state = await readSimulationState(page);
await page.screenshot({ path: outPath, fullPage: false });
await browser.close();

const fatal = fatalErrors(consoleErrors, pageErrors);
const ok =
  state.webgpu && state.canvasWidth > 0 && state.canvasHeight > 0 && fatal.length === 0;

console.log(JSON.stringify({ ok, state, screenshot: outPath, fatal }, null, 2));
process.exit(ok ? 0 : 1);
