import test from "node:test";
import assert from "node:assert/strict";
import { computeDraftAction } from "../src/lib/capture/draftState.js";

test("returns ignore when not running in Tauri", () => {
    assert.deepEqual(
        computeDraftAction({ content: "hello", isTauri: false, isRestoringDraft: false }),
        { action: "ignore" }
    );
    assert.deepEqual(
        computeDraftAction({ content: "", isTauri: false, isRestoringDraft: false }),
        { action: "ignore" }
    );
});

test("returns ignore during draft restore or programmatic reset", () => {
    assert.deepEqual(
        computeDraftAction({ content: "", isTauri: true, isRestoringDraft: true }),
        { action: "ignore" }
    );
    assert.deepEqual(
        computeDraftAction({ content: "hello", isTauri: true, isRestoringDraft: true }),
        { action: "ignore" }
    );
});

test("returns clear when content is empty or whitespace-only", () => {
    assert.deepEqual(
        computeDraftAction({ content: "", isTauri: true, isRestoringDraft: false }),
        { action: "clear" }
    );
    assert.deepEqual(
        computeDraftAction({ content: "   ", isTauri: true, isRestoringDraft: false }),
        { action: "clear" }
    );
    assert.deepEqual(
        computeDraftAction({ content: "\n\t  \n", isTauri: true, isRestoringDraft: false }),
        { action: "clear" }
    );
});

test("returns save with content when content has non-whitespace text", () => {
    assert.deepEqual(
        computeDraftAction({ content: "hello", isTauri: true, isRestoringDraft: false }),
        { action: "save", content: "hello" }
    );
    assert.deepEqual(
        computeDraftAction({ content: "  hello world  ", isTauri: true, isRestoringDraft: false }),
        { action: "save", content: "  hello world  " }
    );
});
