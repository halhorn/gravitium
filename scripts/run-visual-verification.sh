#!/usr/bin/env bash
# Run the generic view verification script (WebGPU + screenshot).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASE_URL="${GRAVITIUM_BASE_URL:-http://127.0.0.1:8080}"

if ! curl -sf "${BASE_URL}/" >/dev/null; then
  echo "error: ${BASE_URL} is not reachable. Start trunk first:" >&2
  echo "  RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve --address 127.0.0.1 --port 8080" >&2
  exit 1
fi

cd "${ROOT}/scripts"
if [[ ! -d node_modules ]]; then
  npm install
  npx playwright install chromium
fi

unset NO_COLOR FORCE_COLOR

if [[ "$(uname -s)" == "Linux" ]]; then
  export VK_ICD_FILENAMES="${VK_ICD_FILENAMES:-/usr/share/vulkan/icd.d/lvp_icd.x86_64.json}"
  xvfb-run -a bash -lc "cd '${ROOT}/scripts' && npm run verify:view"
else
  npm run verify:view
fi
