import { TIMEOUT_MS } from "./config.mjs";

export async function waitForSimulation(page, settleMs = 5000) {
  page.setDefaultTimeout(TIMEOUT_MS);
  await page.waitForFunction(() => {
    const loading = document.getElementById("gravitium-loading");
    const noWebgpu = document.getElementById("gravitium-no-webgpu");
    return loading?.hasAttribute("hidden") && noWebgpu?.hasAttribute("hidden");
  });
  if (settleMs > 0) {
    await page.waitForTimeout(settleMs);
  }
}

export async function readSimulationState(page) {
  return page.evaluate(() => {
    const canvas = document.getElementById("gravitium-canvas");
    return {
      webgpu: !!navigator.gpu,
      canvasWidth: canvas?.width ?? 0,
      canvasHeight: canvas?.height ?? 0,
      title: document.title,
      hash: location.hash,
    };
  });
}
