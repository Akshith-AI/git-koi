const assert = require("node:assert/strict");
const fs = require("node:fs");
const http = require("node:http");
const path = require("node:path");
const { chromium } = require("playwright");

// Build and generate both bundles first; this test never calls LeetCode or GitHub.
const root = path.resolve("target");
const mime = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".png": "image/png",
  ".gif": "image/gif",
};
const server = http.createServer((request, response) => {
  const file = path.resolve(
    root,
    "." + decodeURIComponent(request.url.split("?")[0]),
  );
  if (!file.startsWith(root + path.sep)) {
    response.writeHead(403).end();
    return;
  }
  fs.readFile(file, (error, bytes) => {
    if (error) {
      response.writeHead(404).end();
      return;
    }
    response.setHeader(
      "Content-Type",
      mime[path.extname(file)] || "application/octet-stream",
    );
    response.end(bytes);
  });
});

(async () => {
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const browser = await chromium.launch({
    headless: true,
    ...(process.env.LTKOI_BROWSER
      ? { executablePath: process.env.LTKOI_BROWSER }
      : {}),
  });
  try {
    for (const mode of ["betta", "pond"]) {
      const page = await browser.newPage({
        viewport: { width: 700, height: 850 },
      });
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(
        `http://127.0.0.1:${server.address().port}/animated-${mode}/embed.html`,
      );
      const canvas = page.locator("canvas");
      await page.waitForFunction(() =>
        document.querySelector("#status").textContent.includes("Hover"),
      );
      const first = await canvas.getAttribute("data-frame");
      await page.waitForFunction(
        (frame) => document.querySelector("canvas").dataset.frame !== frame,
        first,
      );
      const hit = await canvas.evaluate((canvas) => {
        const context = canvas.getContext("2d");
        const pixels = context.getImageData(0, 0, 256, 256).data;
        for (let y = 60; y < 125; y++)
          for (let x = 80; x < 220; x++) {
            if (pixels[(y * 256 + x) * 4 + 3] > 0) return { x, y };
          }
        throw new Error("No betta pixels in hit area");
      });
      const bounds = await canvas.boundingBox();
      await page.mouse.move(
        bounds.x + (hit.x / 256) * bounds.width,
        bounds.y + (hit.y / 256) * bounds.height,
      );
      await page.waitForFunction(
        () => document.querySelector("canvas").dataset.flare === "4",
      );
      await page.mouse.move(5, 5);
      await page.waitForFunction(
        () => document.querySelector("canvas").dataset.flare === "0",
      );
      await page.keyboard.press("Tab");
      await page.waitForFunction(() => document.activeElement.id === "fish");
      await page.waitForFunction(
        () => document.querySelector("canvas").dataset.flare === "4",
      );
      await page.keyboard.press("Tab");
      await page.waitForFunction(
        () => document.querySelector("canvas").dataset.flare === "0",
      );
      await page.locator("#pause").click();
      await page.waitForTimeout(100);
      const paused = await canvas.getAttribute("data-frame");
      await page.waitForTimeout(150);
      assert.equal(await canvas.getAttribute("data-frame"), paused);
      await page.locator("#pause").click();
      await page.emulateMedia({ reducedMotion: "reduce" });
      await page.waitForTimeout(100);
      const still = await canvas.getAttribute("data-frame");
      await page.waitForTimeout(150);
      assert.equal(await canvas.getAttribute("data-frame"), still);
      await page.keyboard.press("Shift+Tab");
      await page.waitForFunction(
        () => document.querySelector("canvas").dataset.flare === "4",
      );
      await page.screenshot({
        path: path.join(root, `widget-${mode}-verified.png`),
      });
      assert.deepEqual(errors, []);
      await page.close();
      console.log(
        `${mode}: animation, pixel hover, leave, keyboard, pause, reduced motion passed`,
      );
    }
    const context = await browser.newContext({
      hasTouch: true,
      isMobile: true,
      viewport: { width: 390, height: 780 },
    });
    const page = await context.newPage();
    await page.goto(
      `http://127.0.0.1:${server.address().port}/animated-betta/embed.html`,
    );
    await page.waitForFunction(() =>
      document.querySelector("#status").textContent.includes("Hover"),
    );
    await page.locator("#fish").tap();
    await page.waitForFunction(
      () => document.querySelector("canvas").dataset.flare === "4",
    );
    await page.waitForFunction(
      () => document.querySelector("canvas").dataset.flare === "0",
    );
    await page.close();
    await context.close();
    console.log("touch: brief flare followed by relaxation passed");
  } finally {
    await browser.close();
    server.close();
  }
})().catch((error) => {
  console.error(error);
  server.close();
  process.exitCode = 1;
});
