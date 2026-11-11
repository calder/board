import { expect, test, type Page } from "@playwright/test";

const scenes = {
  empty: "/",
  message: `/?background=rainbow&text=${encodeURIComponent("🟥 Hi Aya 🟦\nWelcome to the\nworld")}`,
  "too-long": `/?text=${encodeURIComponent("One\nTwo\nThree\nFour\nFive\nSix\nSeven")}`,
};

async function open(page: Page, path: string) {
  await page.goto(path);
  await expect(page.locator(".tile")).toHaveCount(132);
  // With the keyboard up someone is typing, so the text box has focus.
  if (test.info().project.name.endsWith("keyboard")) {
    await page.locator("textarea").focus();
  }
}

async function expectOnScreen(page: Page, selector: string) {
  const box = await page.locator(selector).boundingBox();
  const { width, height } = page.viewportSize()!;
  expect(box, `${selector} is rendered`).not.toBeNull();
  expect(box!.x, `${selector} left edge`).toBeGreaterThanOrEqual(0);
  expect(box!.y, `${selector} top edge`).toBeGreaterThanOrEqual(0);
  expect(box!.x + box!.width, `${selector} right edge`).toBeLessThanOrEqual(width + 0.5);
  expect(box!.y + box!.height, `${selector} bottom edge`).toBeLessThanOrEqual(height + 0.5);
}

async function expectNoSideScroll(page: Page) {
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
  expect(overflow, "page scrolls sideways").toBeLessThanOrEqual(0);
}

for (const [name, path] of Object.entries(scenes)) {
  test(name, async ({ page }) => {
    await open(page, path);
    await expectNoSideScroll(page);
    for (const selector of [".frame", ".compose", ".toolbar"]) {
      await expectOnScreen(page, selector);
    }
    // The test board starts blank, so a fresh page shows it until someone types.
    if (name === "empty") {
      await expect(page.locator(".badge")).toHaveText("Live");
    }
    if (name === "too-long") {
      await expectOnScreen(page, ".note.shown");
    }
    await expect(page).toHaveScreenshot(`${name}.png`);
  });
}

test("background menu", async ({ page }) => {
  await open(page, scenes.message);
  await page.locator(".picker .round").click();
  await expectOnScreen(page, ".options");
  await expectNoSideScroll(page);
  await expect(page).toHaveScreenshot("background-menu.png");
});

test("board", async ({ page }) => {
  const grid = Array.from({ length: 6 }, (_, row) =>
    Array.from({ length: 22 }, () => (row === 0 || row === 5 ? 67 : 0)),
  );
  grid[2].splice(8, 5, 8, 5, 12, 12, 15);
  // The stream ends after one grid and waits long to retry, so the page settles offline.
  await page.route("/events", (route) =>
    route.fulfill({
      contentType: "text/event-stream",
      body: `retry: 600000\ndata: ${JSON.stringify(grid)}\n\n`,
    }),
  );
  await open(page, "/");
  await expect(page.locator(".badge")).toHaveText("Offline");
  await expect(page.locator(".board")).toContainText("HELLO");
  await expectOnScreen(page, ".badge");
  await expect(page.locator(".send")).toBeDisabled();
  await expect(page).toHaveScreenshot("board.png");

  // Tapping the board switches to the draft so it can be sent.
  await page.locator(".stage").click();
  await expect(page.locator(".badge")).toHaveCount(0);
  await expect(page.locator(".board")).not.toContainText("HELLO");
  await expect(page.locator(".send")).toBeEnabled();
});
