import test from "node:test";
import assert from "node:assert/strict";
import { computeMarkdownWrap } from "../src/lib/capture/markdownWrap.js";

function base(content, selectionStart = 0, selectionEnd = content.length, key = "b", style = "bold") {
    return { content, selectionStart, selectionEnd, key, style, metaKey: true };
}

test("bold wraps the selection", () => {
    const result = computeMarkdownWrap(base("hello world", 6, 11));
    assert.deepEqual(result, {
        nextContent: "hello **world**",
        nextSelectionStart: 8,
        nextSelectionEnd: 13,
    });
});

test("italic wraps the selection with single asterisks", () => {
    const result = computeMarkdownWrap({
        content: "hello world",
        selectionStart: 6,
        selectionEnd: 11,
        key: "i",
        style: "italic",
        metaKey: true,
    });
    assert.deepEqual(result, {
        nextContent: "hello *world*",
        nextSelectionStart: 7,
        nextSelectionEnd: 12,
    });
});

test("bold toggles off when markers sit outside the selection", () => {
    const result = computeMarkdownWrap(base("hello **world**", 8, 13));
    assert.deepEqual(result, {
        nextContent: "hello world",
        nextSelectionStart: 6,
        nextSelectionEnd: 11,
    });
});

test("bold toggles off when the selection carries the markers", () => {
    const result = computeMarkdownWrap(base("hello **world**", 6, 15));
    assert.deepEqual(result, {
        nextContent: "hello world",
        nextSelectionStart: 8,
        nextSelectionEnd: 13,
    });
});

test("collapsed selection inserts an empty pair with cursor between", () => {
    const result = computeMarkdownWrap(base("hello ", 6, 6));
    assert.deepEqual(result, {
        nextContent: "hello ****",
        nextSelectionStart: 8,
        nextSelectionEnd: 8,
    });
});

test("multi-line selection is not handled", () => {
    assert.equal(
        computeMarkdownWrap(base("- a\n- b", 2, 5)),
        null,
    );
});

test("without primary modifier nothing is handled", () => {
    assert.equal(
        computeMarkdownWrap({ ...base("hello", 0, 5), metaKey: false, ctrlKey: false }),
        null,
    );
});

test("shift or alt is rejected", () => {
    assert.equal(computeMarkdownWrap({ ...base("hello", 0, 5), shiftKey: true }), null);
    assert.equal(computeMarkdownWrap({ ...base("hello", 0, 5), altKey: true }), null);
});

test("other keys are rejected", () => {
    assert.equal(computeMarkdownWrap({ ...base("hello", 0, 5), key: "x" }), null);
    assert.equal(computeMarkdownWrap({ ...base("hello", 0, 5), key: "u", style: "italic" }), null);
});

test("upper-case key still toggles", () => {
    const result = computeMarkdownWrap({ ...base("a", 0, 1), key: "B" });
    assert.deepEqual(result, {
        nextContent: "**a**",
        nextSelectionStart: 2,
        nextSelectionEnd: 3,
    });
});

test("ctrl works as primary modifier", () => {
    const result = computeMarkdownWrap({ ...base("hello", 0, 5), metaKey: false, ctrlKey: true });
    assert.deepEqual(result, {
        nextContent: "**hello**",
        nextSelectionStart: 2,
        nextSelectionEnd: 7,
    });
});
