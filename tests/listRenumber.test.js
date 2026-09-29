import test from "node:test";
import assert from "node:assert/strict";
import { computeListIndentation } from "../src/lib/capture/list-continuation.js";
import { computeListKeyEdit } from "../src/lib/reader/editorListKeys.js";

function tabAtEnd(content, shiftKey = false) {
    return computeListIndentation({
        content,
        selectionStart: content.length,
        selectionEnd: content.length,
        key: "Tab",
        shiftKey,
    });
}

test("Tab on a numbered item restarts the nested list at 1.", () => {
    assert.deepEqual(tabAtEnd("1. a\n2. b"), {
        nextContent: "1. a\n   1. b",
        nextCursorPosition: 12,
    });
});

test("Tab on a two-digit numbered item restarts at 1. and keeps the caret after the text", () => {
    assert.deepEqual(tabAtEnd("9. x\n10. y"), {
        nextContent: "9. x\n    1. y",
        nextCursorPosition: 13,
    });
});

test("Tab with the caret at the line start keeps the caret before the marker", () => {
    const content = "1. a\n2. b";
    assert.deepEqual(
        computeListIndentation({
            content,
            selectionStart: 5,
            selectionEnd: 5,
            key: "Tab",
        }),
        { nextContent: "1. a\n   1. b", nextCursorPosition: 8 },
    );
});

test("Shift+Tab continues the numbering of the outer list", () => {
    assert.deepEqual(tabAtEnd("1. a\n2. b\n   1. c\n   2. d", true), {
        nextContent: "1. a\n2. b\n   1. c\n3. d",
        nextCursorPosition: 22,
    });
});

test("Shift+Tab one level out of a three-level list continues the middle level", () => {
    assert.deepEqual(tabAtEnd("1. a\n   1. b\n      1. c\n      2. d", true), {
        nextContent: "1. a\n   1. b\n      1. c\n   2. d",
        nextCursorPosition: 31,
    });
});

test("Shift+Tab under a bullet at the target level restarts at 1.", () => {
    assert.deepEqual(tabAtEnd("- a\n   5. b", true), {
        nextContent: "- a\n1. b",
        nextCursorPosition: 8,
    });
});

test("Shift+Tab does not look past a blank line", () => {
    assert.deepEqual(tabAtEnd("1. a\n\n   5. b", true), {
        nextContent: "1. a\n\n1. b",
        nextCursorPosition: 10,
    });
});

test("bullets keep their marker on Tab and Shift+Tab", () => {
    assert.deepEqual(tabAtEnd("1. a\n- b"), {
        nextContent: "1. a\n  - b",
        nextCursorPosition: 10,
    });
    assert.deepEqual(tabAtEnd("1. a\n  - b", true), {
        nextContent: "1. a\n- b",
        nextCursorPosition: 8,
    });
});

function readerKey(lineText, precedingLines, shiftKey = false) {
    return computeListKeyEdit({
        lineText,
        precedingLines,
        selectionStart: lineText.length,
        selectionEnd: lineText.length,
        key: "Tab",
        shiftKey,
    });
}

test("Reader: Tab on a numbered block restarts at 1.", () => {
    assert.deepEqual(readerKey("2. b", ["1. a"]), {
        lines: ["   1. b"],
        caretLine: 0,
        caretOffset: 7,
    });
});

test("Reader: Shift+Tab uses the preceding blocks to continue the outer numbering", () => {
    assert.deepEqual(readerKey("   2. d", ["1. a", "2. b", "   1. c"], true), {
        lines: ["3. d"],
        caretLine: 0,
        caretOffset: 4,
    });
});

test("Reader: without preceding lines Shift+Tab starts at 1.", () => {
    assert.deepEqual(readerKey("   2. d", undefined, true), {
        lines: ["1. d"],
        caretLine: 0,
        caretOffset: 4,
    });
});

test("Reader: outdenting a long marker with the caret inside it never yields a negative caret", () => {
    const edit = computeListKeyEdit({
        lineText: "   100. b",
        precedingLines: ["1. a"],
        selectionStart: 4,
        selectionEnd: 4,
        key: "Tab",
        shiftKey: true,
    });
    assert.deepEqual(edit.lines, ["2. b"]);
    assert.ok(edit.caretOffset >= 0 && edit.caretOffset <= edit.lines[0].length);
});

test("Tab then Shift+Tab on item 10 returns to the original level", () => {
    const indented = tabAtEnd("9. x\n10. y");
    assert.equal(indented.nextContent, "9. x\n    1. y");
    assert.deepEqual(tabAtEnd(indented.nextContent, true), {
        nextContent: "9. x\n10. y",
        nextCursorPosition: 10,
    });
});

test("Shift+Tab lands on the parent level even when the marker widths differ", () => {
    assert.deepEqual(tabAtEnd("- a\n    - b", true), {
        nextContent: "- a\n- b",
        nextCursorPosition: 7,
    });
});

test("Tab with the caret inside a two-digit marker puts it right after the new 1.", () => {
    assert.deepEqual(
        computeListIndentation({
            content: "10. y",
            selectionStart: 1,
            selectionEnd: 1,
            key: "Tab",
        }),
        { nextContent: "    1. y", nextCursorPosition: 6 },
    );
});

test("Shift+Tab with mixed tab/space indentation keeps the caret after the marker", () => {
    // "\t1. b" (4 columns) under "   1. a" (3 columns) -> parent indent "   ".
    const content = "   1. a\n\t1. b";
    const caretAfterMarker = 8 + 3; // "\t1." -> caret right after the marker
    assert.deepEqual(
        computeListIndentation({
            content,
            selectionStart: caretAfterMarker,
            selectionEnd: caretAfterMarker,
            key: "Tab",
            shiftKey: true,
        }),
        { nextContent: "   1. a\n   2. b", nextCursorPosition: 8 + 5 },
    );
});
