import { join } from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = join(fileURLToPath(new URL("../..", import.meta.url)));

export const BASE_URL = process.env.GRAVITIUM_BASE_URL ?? "http://127.0.0.1:8080";
export const TIMEOUT_MS = Number(process.env.GRAVITIUM_TIMEOUT_MS ?? 240_000);
export const VIEWPORT = { width: 1280, height: 720 };
export const DESKTOP_PANEL_WIDTH = 300;
export const SCREENSHOT_DIR =
  process.env.GRAVITIUM_SCREENSHOT_DIR ?? join(repoRoot, "artifacts", "screenshots");

export const DEFAULT_URL_BODY_PREFIX =
  "v=1&soft=0.01&merge=20.0&ts=1.0&svs=50.0&mss=0.02&seed=12345678&nstars=1&stmass=100.0&ror=3.0&dmmin=0.002&dmmax=0.02&drmin=0.01";
export const DEFAULT_URL_BODY_SUFFIX =
  "&dh=1.0&vpert=0.5&active=10000&term=s:1,e:-3,c:39.4784";

export function urlWithHash(hashBody) {
  return `${BASE_URL}/#${hashBody}`;
}

export function urlForOuterRadius(outerRadiusAu) {
  const body = `${DEFAULT_URL_BODY_PREFIX}&drmax=${outerRadiusAu}${DEFAULT_URL_BODY_SUFFIX}`;
  return urlWithHash(body);
}
