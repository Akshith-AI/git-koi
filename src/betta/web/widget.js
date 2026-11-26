(() => {
  "use strict";
  const canvas = document.querySelector("canvas");
  const control = document.querySelector("#fish");
  const pause = document.querySelector("#pause");
  const status = document.querySelector("#status");
  const context = canvas.getContext("2d", { willReadFrequently: true });
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
  const image = new Image();
  let ready = false;
  let hovered = false;
  let focused = false;
  let tapUntil = 0;
  let paused = false;
  let level = 0;
  let frame = 0;
  let elapsed = 0;
  let previous = 0;
  let request = null;
  const pond = document.body.dataset.mode === "pond";
  const draw = () => {
    if (!ready) return;
    const index = Math.round(level) * 60 + frame;
    context.clearRect(0, 0, 256, 256);
    context.imageSmoothingEnabled = false;
    context.drawImage(
      image,
      (index % 10) * 256,
      Math.floor(index / 10) * 256,
      256,
      256,
      0,
      0,
      256,
      256,
    );
    canvas.dataset.flare = String(Math.round(level));
    canvas.dataset.frame = String(frame);
  };
  const target = (now) => (hovered || focused || now < tapUntil ? 4 : 0);
  const tick = (now) => {
    request = null;
    const dt = previous ? Math.min(now - previous, 80) : 0;
    previous = now;
    const opening = target(now);
    if (reduced.matches) level = opening;
    else
      level +=
        Math.sign(opening - level) *
        Math.min(Math.abs(opening - level), dt / 65);
    if (!paused && !reduced.matches && !document.hidden) {
      elapsed += dt;
      frame = Math.floor(elapsed / (1000 / 15)) % 60;
    }
    draw();
    if (
      !document.hidden &&
      (tapUntil > now ||
        (!reduced.matches && (!paused || Math.abs(level - opening) > 0.01)))
    )
      wake();
  };
  function wake() {
    if (ready && request === null && !document.hidden)
      request = requestAnimationFrame(tick);
  }
  function refresh() {
    previous = 0;
    wake();
  }
  image.onload = () => {
    ready = true;
    status.textContent = "Hover the betta, focus it, or tap to flare.";
    draw();
    refresh();
  };
  image.onerror = () => {
    status.textContent =
      "Interactive images could not load. The still portrait remains below.";
  };
  image.src = "sprites.png";
  control.addEventListener("pointermove", (event) => {
    if (event.pointerType === "touch" || !ready) return;
    const bounds = canvas.getBoundingClientRect();
    const x = Math.floor(((event.clientX - bounds.left) * 256) / bounds.width);
    const y = Math.floor(((event.clientY - bounds.top) * 256) / bounds.height);
    const inside = x >= 0 && x < 256 && y >= 0 && y < 256 && (!pond || y < 127);
    let hit = inside;
    if (inside) {
      try {
        hit = context.getImageData(x, y, 1, 1).data[3] > 0;
      } catch {
        /* Local file fallback; HTTP embeds use pixel hit testing. */
      }
    }
    if (hovered !== hit) {
      hovered = hit;
      refresh();
    }
  });
  control.addEventListener("pointerleave", () => {
    hovered = false;
    refresh();
  });
  control.addEventListener("focus", () => {
    focused = control.matches(":focus-visible");
    refresh();
  });
  control.addEventListener("blur", () => {
    focused = false;
    refresh();
  });
  control.addEventListener("click", (event) => {
    if (event.detail === 0) {
      tapUntil = performance.now() + 1500;
      refresh();
    }
  });
  control.addEventListener("pointerup", (event) => {
    if (event.pointerType === "touch" || event.pointerType === "pen") {
      // Pointer focus must not turn a brief touch flare into a permanent one.
      control.blur();
      focused = false;
      tapUntil = performance.now() + 1500;
      refresh();
    }
  });
  pause.addEventListener("click", () => {
    paused = !paused;
    pause.setAttribute("aria-pressed", String(paused));
    pause.textContent = paused ? "Resume swimming" : "Pause swimming";
    refresh();
  });
  reduced.addEventListener("change", () => {
    frame = 0;
    elapsed = 0;
    refresh();
  });
  document.addEventListener("visibilitychange", refresh);
})();
