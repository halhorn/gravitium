import { mkdirSync } from "node:fs";
import { join } from "node:path";

import { SCREENSHOT_DIR } from "./config.mjs";

export function ensureScreenshotDir(dir = SCREENSHOT_DIR) {
  mkdirSync(dir, { recursive: true });
  return dir;
}

export function screenshotPath(name, dir = SCREENSHOT_DIR) {
  return join(ensureScreenshotDir(dir), name);
}
