#!/usr/bin/env node
/**
 * Visual check for outer-radius camera framing via in-app Restart (UI flow).
 */
import { BASE_URL, TIMEOUT_MS, VIEWPORT } from "./lib/config.mjs";
import { launchWebGpuBrowser } from "./lib/browser.mjs";
import { waitForSimulation } from "./lib/simulation.mjs";
import { measureViewportMetrics, screenshotPath } from "./lib/screenshot.mjs";
import { clickRestart, readOuterRadiusAu, setOuterRadiusAu } from "./lib/ui.mjs";

async function captureCase(page, outerRadiusAu, label) {
  await setOuterRadiusAu(page, outerRadiusAu);
  await clickRestart(page);
  await waitForSimulation(page);

  const panelValue = await readOuterRadiusAu(page);
  const path = screenshotPath(`${label}-outer-${outerRadiusAu}au.png`);
  await page.screenshot({ path, fullPage: false });

  return {
    label,
    outerRadiusAu,
    panelValue,
    path,
    metrics: measureViewportMetrics(path),
  };
}

const browser = await launchWebGpuBrowser();
const page = await browser.newPage({ viewport: VIEWPORT });
page.setDefaultTimeout(TIMEOUT_MS);

await page.goto(BASE_URL, { waitUntil: "domcontentloaded", timeout: TIMEOUT_MS });
await waitForSimulation(page);

const results = [];
for (const [label, radius] of [
  ["small", 20],
  ["default", 60],
  ["large", 150],
]) {
  results.push(await captureCase(page, radius, label));
}

await browser.close();

const core20 = results.find((r) => r.outerRadiusAu === 20).metrics.coreWidthPx;
const core150 = results.find((r) => r.outerRadiusAu === 150).metrics.coreWidthPx;
const coreRatio = core20 / core150;
const expectedRadiusRatio = 150 / 20;

const checks = {
  coreZoomRatio: coreRatio,
  expectedRadiusRatio,
  panelValues: Object.fromEntries(results.map((r) => [r.outerRadiusAu, r.panelValue])),
  fieldWidthRatios: Object.fromEntries(
    results.map((r) => [r.outerRadiusAu, Number(r.metrics.fieldWidthRatio.toFixed(3))]),
  ),
  coreWidths: Object.fromEntries(
    results.map((r) => [r.outerRadiusAu, Number(r.metrics.coreWidthPx.toFixed(1))]),
  ),
};

const pass =
  results.every((r) => r.panelValue === r.outerRadiusAu) &&
  coreRatio > expectedRadiusRatio * 0.45 &&
  coreRatio < expectedRadiusRatio * 2.5 &&
  results.every((r) => r.metrics.fieldWidthRatio > 0.55) &&
  core20 > core150 * 2;

console.log(JSON.stringify({ results, checks, pass }, null, 2));
process.exit(pass ? 0 : 1);
