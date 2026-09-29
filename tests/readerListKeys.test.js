import test from "node:test";
import assert from "node:assert/strict";
import { computeListKeyEdit, splitLineEdit } from "../src/lib/reader/editorListKeys.js";

function key(lineText, keyName, extra = {}) {
    const caret = extra.caret ?? lineText.length;
    return computeListKeyEdit({
        lineText,
        selectionStart: caret,
        selectionEnd: extra.selectionEnd ?? caret,
        key: keyName,
        shiftKey: false,
        metaKey: false,
        ctrlKey: false,
        altKey: false,
        isComposing: false,
        ...extra,
    });
}

test("Enter at end of a nested item continues with the same indent", () => {
    assert.deepEqual(key("   - child", "Enter"), {
        lines: ["   - child", "   - "],
        caretLine: 1,
        caretOffset: 5,
    });
});

test("Enter in the middle moves the rest to the new item", () => {
    assert.deepEqual(key("- abcdef", "Enter", { caret: 5 }), {
        lines: ["- abc", "- def"],
        caretLine: 1,
        caretOffset: 2,
    });
});

test("Enter on a numbered item increments", () => {
    assert.deepEqual(key("1. one", "Enter").lines, ["1. one", "2. "]);
});

test("Enter on an empty item ends the list", () => {
    assert.deepEqual(key("   - ", "Enter"), { lines: [""], caretLine: 0, caretOffset: 0 });
});

test("Tab indents by one tab, Shift+Tab outdents", () => {
    assert.deepEqual(key("- item", "Tab", { caret: 3 }), {
        lines: ["\t- item"],
        caretLine: 0,
        caretOffset: 4,
    });
    assert.deepEqual(key("  - item", "Tab", { caret: 5, shiftKey: true }), {
        lines: ["- item"],
        caretLine: 0,
        caretOffset: 3,
    });
});

test("non-list lines, modifiers and IME composition are not handled", () => {
    assert.equal(key("plain text", "Enter"), null);
    assert.equal(key("plain text", "Tab"), null);
    assert.equal(key("- item", "Enter", { shiftKey: true }), null);
    assert.equal(key("- item", "Enter", { metaKey: true }), null);
    assert.equal(key("- item", "Enter", { isComposing: true }), null);
    assert.equal(key("- item", "a"), null);
});

test("a block containing a newline is not handled", () => {
    assert.equal(key("- a\n- b", "Enter"), null);
});

test("splitLineEdit keeps single-line results on line 0", () => {
    assert.deepEqual(splitLineEdit({ nextContent: "abc", nextCursorPosition: 2 }), {
        lines: ["abc"],
        caretLine: 0,
        caretOffset: 2,
    });
});
