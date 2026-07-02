/** egui control helpers for Playwright verification. */

export async function clickRestart(page) {
  await page.getByText("↻", { exact: true }).click();
}

export async function setOuterRadiusAu(page, outerRadiusAu) {
  const changed = await page.evaluate((value) => {
    const label = [...document.querySelectorAll("*")].find(
      (el) => el.childNodes.length === 1 && el.textContent?.trim() === "Outer radius (AU)",
    );
    if (!label) return false;

    let row = label.parentElement;
    while (row && !row.querySelector("input")) {
      row = row.parentElement;
    }
    const input = row?.querySelector("input");
    if (!input) return false;

    input.value = String(value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  }, outerRadiusAu);

  if (!changed) {
    throw new Error("Outer radius (AU) slider not found");
  }
}

export async function readOuterRadiusAu(page) {
  return page.evaluate(() => {
    const text = document.body.innerText;
    const match = text.match(/Outer radius \(AU\)\s*([\d.]+)/);
    return match ? Number(match[1]) : null;
  });
}
