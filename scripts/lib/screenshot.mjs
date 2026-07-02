import { mkdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { PNG } from "pngjs";

import { DESKTOP_PANEL_WIDTH, SCREENSHOT_DIR } from "./config.mjs";

export function ensureScreenshotDir(dir = SCREENSHOT_DIR) {
  mkdirSync(dir, { recursive: true });
  return dir;
}

export function screenshotPath(name, dir = SCREENSHOT_DIR) {
  return join(ensureScreenshotDir(dir), name);
}

/** Rough viewport metrics from a full-window PNG (simulation panel starts at x=300). */
export function measureViewportMetrics(path, panelWidth = DESKTOP_PANEL_WIDTH) {
  const png = PNG.sync.read(readFileSync(path));
  const w = png.width;
  const h = png.height;
  const x0 = panelWidth;
  const vw = w - panelWidth;
  const vh = h;

  let minX = vw;
  let maxX = 0;
  let coreMinX = vw;
  let coreMaxX = 0;
  let coreMinY = vh;
  let coreMaxY = 0;

  const cx = x0 + vw * 0.5;
  const cy = vh * 0.52;

  for (let y = 0; y < vh; y++) {
    for (let x = x0; x < w; x++) {
      const i = (y * w + x) * 4;
      const r = png.data[i];
      const g = png.data[i + 1];
      const b = png.data[i + 2];
      if (r + g + b < 30) continue;

      const localX = x - x0;
      minX = Math.min(minX, localX);
      maxX = Math.max(maxX, localX);

      const dx = x - cx;
      const dy = y - cy;
      if (dx * dx + dy * dy > 55 * 55) continue;
      if (r > 120 && g > 40 && b < 120) {
        coreMinX = Math.min(coreMinX, localX);
        coreMaxX = Math.max(coreMaxX, localX);
        coreMinY = Math.min(coreMinY, y);
        coreMaxY = Math.max(coreMaxY, y);
      }
    }
  }

  const fieldWidth = maxX >= minX ? maxX - minX : 0;
  const coreWidth =
    coreMaxX >= coreMinX ? Math.max(coreMaxX - coreMinX, coreMaxY - coreMinY) : 0;

  return {
    viewportWidth: vw,
    fieldWidthPx: fieldWidth,
    fieldWidthRatio: vw > 0 ? fieldWidth / vw : 0,
    coreWidthPx: coreWidth,
  };
}
