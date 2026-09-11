// Opt-in diagnostic only, enabled with VITE_RESIZE_DIAGNOSTICS=1 in a dev run.
// See docs/resize-investigation-2026-09-11.md.
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";

export async function installResizeDiagnostics(onSettled = () => {}) {
  if (!import.meta.env.DEV) throw new Error("Resize diagnostics require a dev run");
  window.collectorResizeDiagnostics?.dispose();
  const root = document.querySelector(".capture-container, .reader-container");
  if (!root || root.classList.contains("resizing")) {
    throw new Error("Open Capture or Reader and wait until resizing has settled");
  }
  const appWindow = getCurrentWindow();
  const entries = [];
  const removers = [];
  const started = performance.now();
  let saveTimer;
  const properties = ["backdrop-filter", "-webkit-backdrop-filter", "box-shadow"];
  const inline = properties.map((name) => [
    root.style.getPropertyValue(name), root.style.getPropertyPriority(name),
  ]);
  const describe = (element) => element instanceof Element
    ? `${element.tagName.toLowerCase()}${element.id ? `#${element.id}` : ""}.${[...element.classList].join(".")}`
    : null;
  const record = (type, data = {}) => {
    if (entries.length >= 10000) entries.shift();
    entries.push({ ...data, timeMs: Date.now(), ms: Math.round(performance.now() - started), type });
    if (["mouseup", "tauri-resized", "tauri-moved", "class", "mark"].includes(type)) {
      clearTimeout(saveTimer);
      saveTimer = setTimeout(onSettled, 900);
    }
  };
  const restore = () => properties.forEach((name, i) => {
    if (inline[i][0]) root.style.setProperty(name, ...inline[i]);
    else root.style.removeProperty(name);
  });
  const snapshot = () => {
    const handle = root.querySelector(".resize-handle");
    const rect = handle?.getBoundingClientRect();
    const style = getComputedStyle(root);
    return {
      label: appWindow.label,
      viewport: [innerWidth, innerHeight],
      resizing: root.classList.contains("resizing"),
      effects: properties.map((name) => [name, style.getPropertyValue(name)]),
      handle: rect?.toJSON(),
      handleHit: rect ? describe(document.elementFromPoint(
        rect.x + rect.width / 2, rect.y + rect.height / 2,
      )) : null,
    };
  };
  const mouse = (event) => record(event.type, {
    point: [event.clientX, event.clientY], button: event.button,
    buttons: event.buttons, target: describe(event.target),
    handle: event.target instanceof Element && Boolean(event.target.closest(".resize-handle")),
  });
  for (const type of ["mousedown", "mouseup", "pointercancel"]) {
    document.addEventListener(type, mouse, true);
    removers.push(() => document.removeEventListener(type, mouse, true));
  }
  const observer = new MutationObserver(() => {
    const style = getComputedStyle(root);
    record("class", {
      resizing: root.classList.contains("resizing"),
      effects: properties.map((name) => [name, style.getPropertyValue(name)]),
    });
  });
  observer.observe(root, { attributes: true, attributeFilter: ["class"] });
  removers.push(() => observer.disconnect());
  try {
    removers.push(await appWindow.onResized(({ payload }) => record("tauri-resized", payload)));
    removers.push(await appWindow.onMoved(({ payload }) => record("tauri-moved", payload)));
  } catch (error) {
    removers.forEach((remove) => remove());
    throw error;
  }
  let disposed = false;
  const api = {
    snapshot,
    mark: (name) => record("mark", { name, ...snapshot() }),
    clear: () => { entries.length = 0; },
    report: () => JSON.stringify({ snapshot: snapshot(), entries }, null, 2),
    // Change only between trials, after .resizing has cleared. Keep baseline
    // effects constant while the original handler/timer continue unchanged.
    mitigation: (enabled) => {
      if (disposed) throw new Error("Diagnostics have been disposed");
      if (root.classList.contains("resizing")) throw new Error("Wait for resize to settle");
      restore();
      if (!enabled) {
        const style = getComputedStyle(root);
        const baseline = properties.map((name) => style.getPropertyValue(name));
        properties.forEach((name, i) => root.style.setProperty(name, baseline[i], "important"));
      }
      record("mitigation", { enabled });
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      clearTimeout(saveTimer);
      removers.forEach((remove) => remove());
      restore();
      if (window.collectorResizeDiagnostics === api) delete window.collectorResizeDiagnostics;
    },
  };
  window.collectorResizeDiagnostics = api;
  record("installed", snapshot());
  return api;
}

// A shadow root keeps this temporary toolbar independent of app CSS. It is
// centred inside the window, away from every native resize edge and the grip.
export async function mountResizeDiagnosticPanel() {
  if (!import.meta.env.DEV) return;
  const host = document.createElement("div");
  host.style.cssText = "position:fixed;top:64px;left:50%;transform:translateX(-50%);z-index:2147483647;width:260px;max-width:80vw";
  const shadow = host.attachShadow({ mode: "open" });
  shadow.innerHTML = `<style>
    :host { color-scheme:dark; }
    section { background:#18202c;color:#fff;border:1px solid #7ba4d4;border-radius:8px;padding:8px;font:12px system-ui; }
    button { padding:5px;margin:3px 2px;color:#fff;background:#30425b;border:1px solid #8194af;border-radius:4px;cursor:pointer;font:inherit; }
    button[aria-pressed="true"] { background:#196a44;border-color:#8bd5ad; }
    output { display:block;line-height:1.4;overflow-wrap:anywhere; }
  </style><section aria-label="Resize-Diagnose">
    <strong>Resize-Vergleich</strong>
    <div><button data-mode="a">A: Original</button><button data-mode="b">B: Effekte konstant</button></div>
    <button data-action="save">Protokoll speichern</button><button data-action="stop">Beenden</button>
    <output>Diagnose wird gestartet …</output>
  </section>`;
  document.body.append(host);
  const output = shadow.querySelector("output");
  let diagnostics;
  let busy = false;
  const run = async (task) => {
    if (busy) return;
    busy = true;
    try { await task(); }
    catch (error) { output.textContent = `Diagnosefehler: ${String(error)}`; }
    finally { busy = false; }
  };
  const save = async () => invoke("resize_diagnostics", {
    action: "save", frontend: diagnostics.report(),
  });
  const mode = async (value) => {
    diagnostics.mitigation(value === "a");
    diagnostics.mark(value === "a" ? "A-original" : "B-effects-constant");
    await invoke("resize_diagnostics", { action: "snapshot" });
    shadow.querySelectorAll("[data-mode]").forEach((button) => {
      button.setAttribute("aria-pressed", String(button.dataset.mode === value));
    });
    output.textContent = value === "a" ? "A aktiv – jetzt wie gewohnt resizen." : "B aktiv – denselben Drag wiederholen, danach BTT.";
  };
  shadow.addEventListener("click", (event) => {
    const button = event.target.closest?.("button");
    if (!button || !diagnostics) return;
    run(async () => {
      if (button.dataset.mode) return mode(button.dataset.mode);
      if (button.dataset.action === "save") {
        const result = await save();
        output.textContent = `Gespeichert: ${result.nativeEvents} native Ereignisse. ${result.path}`;
      } else {
        await save();
        await invoke("resize_diagnostics", { action: "stop" });
        diagnostics.dispose();
        host.remove();
      }
    });
  });
  await run(async () => {
    await invoke("resize_diagnostics", { action: "start" });
    diagnostics = await installResizeDiagnostics(() => {
      save().catch((error) => { output.textContent = `Speicherfehler: ${String(error)}`; });
    });
    await mode("a");
    await save(); // Confirms native + frontend wiring before any live trial.
  });
}
