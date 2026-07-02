import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(fileURLToPath(new URL("../..", import.meta.url)));

export const BASE_URL = process.env.GRAVITIUM_BASE_URL ?? "http://127.0.0.1:8080";
export const TIMEOUT_MS = Number(process.env.GRAVITIUM_TIMEOUT_MS ?? 240_000);
export const VIEWPORT = { width: 1280, height: 720 };
export const SCREENSHOT_DIR =
  process.env.GRAVITIUM_SCREENSHOT_DIR ?? join(repoRoot, "artifacts", "screenshots");
