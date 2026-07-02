import { chromium } from "playwright";
import { platform } from "node:os";

/** Launch Chrome with WebGPU enabled (Linux CI uses lavapipe + xvfb-run). */
export async function launchWebGpuBrowser() {
  const args = ["--enable-unsafe-webgpu", "--disable-gpu-sandbox"];

  if (platform() === "linux") {
    args.push("--use-vulkan=swiftshader", "--enable-features=Vulkan");
  } else if (platform() === "darwin") {
    args.push("--use-angle=metal");
  }

  return chromium.launch({
    channel: "chrome",
    headless: false,
    args,
  });
}

export function attachErrorCollectors(page) {
  const consoleErrors = [];
  const pageErrors = [];

  page.on("console", (msg) => {
    if (msg.type() === "error") consoleErrors.push(msg.text());
  });
  page.on("pageerror", (err) => pageErrors.push(String(err)));

  return { consoleErrors, pageErrors };
}

export function fatalErrors(consoleErrors, pageErrors) {
  return [...consoleErrors, ...pageErrors].filter(
    (e) => /panicked|wgpu|WebGPU|surface|fatal/i.test(e) && !/favicon|404/i.test(e),
  );
}
