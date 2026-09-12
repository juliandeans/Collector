import test from "node:test";
import assert from "node:assert/strict";
import { handleAppShortcut, createCaptureActions } from "../src/lib/app/captureActions.js";
import {
    buildInitialAppSettings,
    applySettingsUpdate,
} from "../src/lib/app/settingsBridge.js";

function createKeydownEvent({ key, code, metaKey = false, ctrlKey = false, shiftKey = false, altKey = false }) {
    let prevented = false;
    return {
        key,
        code,
        metaKey,
        ctrlKey,
        shiftKey,
        altKey,
        preventDefault() {
            prevented = true;
        },
        get prevented() {
            return prevented;
        },
    };
}

function createActionsHarness({ isLoading = false, isOcrLoading = false, clearDraftError = null } = {}) {
    const calls = {
        freeBlobUrls: 0,
        resetCaptureState: 0,
        clearDraftInvokes: 0,
        statusMessages: [],
        order: [],
    };

    const textarea = {
        focus() {
            calls.focusedTextarea = true;
        },
    };

    const actions = createCaptureActions({
        invoke: async (cmd) => {
            if (cmd === "clear_capture_draft") {
                if (clearDraftError) throw clearDraftError;
                calls.clearDraftInvokes += 1;
                calls.order.push("invoke");
            }
        },
        getContent: () => "",
        setContent: () => {},
        getIsLoading: () => isLoading,
        setIsLoading: () => {},
        getIsOcrLoading: () => isOcrLoading,
        setIsOcrLoading: () => {},
        getCaptureModeActive: () => false,
        setCaptureModeActive: () => {},
        showStatus: (message, type, duration) => {
            calls.statusMessages.push({ message, type, duration });
        },
        freeBlobUrls: () => {
            calls.freeBlobUrls += 1;
            calls.order.push("freeBlobUrls");
        },
        resetCaptureState: () => {
            calls.resetCaptureState += 1;
            calls.order.push("resetCaptureState");
        },
        getTextarea: () => textarea,
        tick: async () => {},
        getAppSettings: () => ({}),
        setAppSettings: () => {},
    });

    return { actions, calls };
}

test("Cmd+Backspace triggers handleClearCapture and consumes the event", async () => {
    const { actions, calls } = createActionsHarness();
    let clearCalled = false;
    const guarded = new Proxy(actions, {
        get(target, prop) {
            if (prop === "handleClearCapture") {
                return () => {
                    clearCalled = true;
                    target.handleClearCapture();
                };
            }
            return target[prop];
        },
    });

    const event = createKeydownEvent({ key: "Backspace", code: "Backspace", metaKey: true });
    const handled = handleAppShortcut({
        event,
        appSettings: { clear_capture_shortcut: "Cmd+Backspace" },
        actions: guarded,
        openAppendPicker: () => {},
    });

    assert.equal(handled, true);
    assert.equal(event.prevented, true);
    assert.equal(clearCalled, true);
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(calls.resetCaptureState, 1);
    assert.equal(calls.clearDraftInvokes, 1);
    assert.equal(calls.statusMessages[0]?.message, "Capture cleared");
});

test("non-matching key combination does not trigger clear", () => {
    const event = createKeydownEvent({ key: "Backspace", code: "Backspace", metaKey: false });
    const handled = handleAppShortcut({
        event,
        appSettings: { clear_capture_shortcut: "Cmd+Backspace" },
        actions: {
            handleClearCapture: () => assert.fail("must not be called"),
        },
        openAppendPicker: () => {},
    });

    assert.equal(handled, false);
});

test("undefined setting (missing bridge wiring) does not trigger clear or throw", () => {
    const event = createKeydownEvent({ key: "Backspace", code: "Backspace", metaKey: true });
    const handled = handleAppShortcut({
        event,
        appSettings: {},
        actions: {
            handleClearCapture: () => assert.fail("must not be called"),
        },
        openAppendPicker: () => {},
    });

    assert.equal(handled, false);
});

test("empty setting disables the clear shortcut", () => {
    const event = createKeydownEvent({ key: "Backspace", code: "Backspace", metaKey: true });
    const handled = handleAppShortcut({
        event,
        appSettings: { clear_capture_shortcut: "" },
        actions: {
            handleClearCapture: () => assert.fail("must not be called"),
        },
        openAppendPicker: () => {},
    });

    assert.equal(handled, false);
});

test("guard: clear is skipped while a save or OCR operation is running", async () => {
    const { actions, calls } = createActionsHarness({ isLoading: true });
    await actions.handleClearCapture();
    assert.equal(calls.resetCaptureState, 0);
    assert.equal(calls.clearDraftInvokes, 0);
    assert.equal(calls.statusMessages.length, 0);
});

test("empty window: clear still runs (idempotent, D8)", async () => {
    const { actions, calls } = createActionsHarness();
    await actions.handleClearCapture();
    assert.equal(calls.resetCaptureState, 1);
    assert.equal(calls.clearDraftInvokes, 1);
    assert.equal(calls.statusMessages[0]?.message, "Capture cleared");
});

test("draft clear failure shows error status instead of success banner", async () => {
    const { actions, calls } = createActionsHarness({
        clearDraftError: new Error("remove failed"),
    });
    await actions.handleClearCapture();
    assert.equal(calls.resetCaptureState, 1);
    assert.equal(calls.statusMessages.length, 1);
    assert.equal(
        calls.statusMessages[0].message,
        "Capture cleared, but the saved draft could not be removed",
    );
    assert.equal(calls.statusMessages[0].type, "error");
});

test("handleClearCapture frees blob urls before resetting capture state", async () => {
    const { actions, calls } = createActionsHarness();
    await actions.handleClearCapture();
    assert.deepEqual(calls.order, ["freeBlobUrls", "resetCaptureState", "invoke"]);
});

test("cmd+comma (settings) takes precedence over a conflicting clear shortcut", () => {
    let clearCalled = false;
    const event = createKeydownEvent({ key: ",", code: "Comma", metaKey: true });
    const handled = handleAppShortcut({
        event,
        appSettings: { clear_capture_shortcut: "Cmd+Comma" },
        actions: {
            openSettings: () => {},
            handleClearCapture: () => {
                clearCalled = true;
            },
        },
        openAppendPicker: () => {},
    });

    assert.equal(handled, true);
    assert.equal(event.prevented, true);
    assert.equal(clearCalled, false);
});

test("buildInitialAppSettings exposes the clear shortcut default", () => {
    const appSettings = buildInitialAppSettings({});
    assert.equal(appSettings.clear_capture_shortcut, "Cmd+Backspace");
});

test("applySettingsUpdate forwards and preserves clear_capture_shortcut", () => {
    const current = buildInitialAppSettings({});
    const forwarded = applySettingsUpdate(current, { clear_capture_shortcut: "Cmd+X" });
    assert.equal(forwarded.clear_capture_shortcut, "Cmd+X");

    const preserved = applySettingsUpdate(current, {});
    assert.equal(preserved.clear_capture_shortcut, "Cmd+Backspace");
});
