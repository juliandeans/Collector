import test from "node:test";
import assert from "node:assert/strict";
import { isDragExemptElement, setupCmdWindowDrag } from "../src/lib/windowDrag.js";

test("isDragExemptElement returns true for interactive elements", () => {
  const createMockElement = (matches) => ({
    closest: (selector) => {
      const parts = selector.split(",").map((s) => s.trim());
      return parts.some((p) => matches.includes(p)) ? {} : null;
    },
  });

  assert.equal(isDragExemptElement(createMockElement([".wikilink"])), true);
  assert.equal(isDragExemptElement(createMockElement(["a[href]"])), true);
  assert.equal(isDragExemptElement(createMockElement(["button"])), true);
  assert.equal(isDragExemptElement(createMockElement(["input"])), true);
  assert.equal(isDragExemptElement(createMockElement(["select"])), true);
  assert.equal(isDragExemptElement(createMockElement([".resize-handle"])), true);
});

test("isDragExemptElement returns false for textarea, contenteditable, and standard containers", () => {
  const createMockElement = () => ({
    closest: () => null,
  });

  assert.equal(isDragExemptElement(null), false);
  assert.equal(isDragExemptElement({}), false);
  assert.equal(isDragExemptElement(createMockElement()), false);
});

test("setupCmdWindowDrag returns early no-op when not in Tauri environment", () => {
  const events = [];
  const mockTarget = {
    addEventListener: (type, fn, opts) => events.push({ type, fn, opts }),
    removeEventListener: () => {},
  };

  const cleanup = setupCmdWindowDrag(mockTarget);
  assert.equal(typeof cleanup, "function");
  assert.equal(events.length, 0, "Should not attach listener outside Tauri");
  cleanup();
});

test("setupCmdWindowDrag handles mousedown events in Tauri environment", async () => {
  let invokedCommand = null;
  let dragPromiseReject = false;

  // Simulate Tauri environment
  globalThis.window = {
    __TAURI_INTERNALS__: {
      metadata: {
        currentWindow: { label: "main" },
      },
      invoke: async (cmd) => {
        invokedCommand = cmd;
        if (dragPromiseReject) {
          throw new Error("IPC drag error");
        }
      },
    },
  };

  let registeredListener = null;
  let removedListener = null;
  const mockTarget = {
    addEventListener: (type, fn, opts) => {
      if (type === "mousedown") registeredListener = fn;
    },
    removeEventListener: (type, fn, opts) => {
      if (type === "mousedown") removedListener = fn;
    },
  };

  const cleanup = setupCmdWindowDrag(mockTarget);
  assert.ok(registeredListener, "Mousedown listener should be attached in Tauri");

  // Case 1: Normal click without Command (e.g. text selection)
  let prevented = false;
  invokedCommand = null;
  registeredListener({
    button: 0,
    metaKey: false,
    target: { closest: () => null },
    preventDefault: () => {
      prevented = true;
    },
  });
  assert.equal(prevented, false, "Normal click without Cmd must NOT preventDefault");
  assert.equal(invokedCommand, null, "Normal click must NOT invoke start_dragging");

  // Case 2: Right click with Command
  prevented = false;
  invokedCommand = null;
  registeredListener({
    button: 2,
    metaKey: true,
    target: { closest: () => null },
    preventDefault: () => {
      prevented = true;
    },
  });
  assert.equal(prevented, false, "Right click must NOT initiate drag");
  assert.equal(invokedCommand, null);

  // Case 3: Cmd-click on exempt element (e.g. wikilink)
  prevented = false;
  invokedCommand = null;
  registeredListener({
    button: 0,
    metaKey: true,
    target: {
      closest: (selector) => (selector.includes(".wikilink") ? {} : null),
    },
    preventDefault: () => {
      prevented = true;
    },
  });
  assert.equal(prevented, false, "Cmd-click on wikilink must NOT preventDefault or drag");
  assert.equal(invokedCommand, null);

  // Case 4: Cmd-drag on editor / textarea / window background
  prevented = false;
  invokedCommand = null;
  registeredListener({
    button: 0,
    metaKey: true,
    target: { closest: () => null },
    preventDefault: () => {
      prevented = true;
    },
  });
  assert.equal(prevented, true, "Cmd-drag on background or textarea MUST preventDefault and initiate drag");
  assert.equal(invokedCommand, "plugin:window|start_dragging");

  // Case 5: Promise rejection in startDragging is handled gracefully without unhandled rejection
  dragPromiseReject = true;
  assert.doesNotThrow(() => {
    registeredListener({
      button: 0,
      metaKey: true,
      target: { closest: () => null },
      preventDefault: () => {},
    });
  });

  // Case 6: Cleanup removes listener
  cleanup();
  assert.equal(removedListener, registeredListener, "Cleanup must remove the exact listener");

  delete globalThis.window;
});
