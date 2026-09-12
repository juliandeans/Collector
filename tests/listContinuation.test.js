import test from "node:test";
import assert from "node:assert/strict";
import { computeListContinuation } from "../src/lib/capture/list-continuation.js";

function cursorAt(content, index = content.length) {
    return {
        content,
        selectionStart: index,
        selectionEnd: index,
        key: "Enter",
    };
}

test("bullet list: Enter continues with the same marker", () => {
    const result = computeListContinuation(cursorAt("- erste Zeile"));
    assert.deepEqual(result, {
        nextContent: "- erste Zeile\n- ",
        nextCursorPosition: 16,
    });
});

test("numbered list: Enter increments the counter", () => {
    const result = computeListContinuation(cursorAt("1. erster Punkt"));
    assert.deepEqual(result, {
        nextContent: "1. erster Punkt\n2. ",
        nextCursorPosition: 19,
    });
});

test("indentation is preserved for nested items", () => {
    const result = computeListContinuation(cursorAt("  - eingerückt"));
    assert.deepEqual(result, {
        nextContent: "  - eingerückt\n  - ",
        nextCursorPosition: 19,
    });
});

test("checkbox items restart as unchecked", () => {
    const result = computeListContinuation(cursorAt("- [ ] aufgabe"));
    assert.deepEqual(result, {
        nextContent: "- [ ] aufgabe\n- [ ] ",
        nextCursorPosition: 20,
    });
});

test("checked checkbox items also restart as unchecked", () => {
    const result = computeListContinuation(cursorAt("- [x] fertig"));
    assert.deepEqual(result, {
        nextContent: "- [x] fertig\n- [ ] ",
        nextCursorPosition: 19,
    });
});

test("empty bullet item is removed instead of continued", () => {
    const result = computeListContinuation(cursorAt("- erstes\n- "));
    assert.deepEqual(result, {
        nextContent: "- erstes\n",
        nextCursorPosition: 9,
    });
});

test("empty numbered item is removed instead of continued", () => {
    const result = computeListContinuation(cursorAt("1. a\n2. "));
    assert.deepEqual(result, {
        nextContent: "1. a\n",
        nextCursorPosition: 5,
    });
});

test("empty checkbox item is removed instead of continued", () => {
    const result = computeListContinuation(cursorAt("- erste\n- [ ] "));
    assert.deepEqual(result, {
        nextContent: "- erste\n",
        nextCursorPosition: 8,
    });
});

test("checked empty checkbox item is removed too", () => {
    const result = computeListContinuation(cursorAt("- erste\n- [x] "));
    assert.deepEqual(result, {
        nextContent: "- erste\n",
        nextCursorPosition: 8,
    });
});

test("checkbox followed by text is still continued", () => {
    const result = computeListContinuation(cursorAt("- [ ] aufgabe"));
    assert.deepEqual(result, {
        nextContent: "- [ ] aufgabe\n- [ ] ",
        nextCursorPosition: 20,
    });
});

test("cursor mid-line splits the item", () => {
    const content = "- hello world";
    const result = computeListContinuation(cursorAt(content, 7));
    // the leading space of the remainder stays with the remainder (no trim)
    assert.deepEqual(result, {
        nextContent: "- hello\n-  world",
        nextCursorPosition: 10,
    });
});

test("plain text without list marker is not handled", () => {
    assert.equal(computeListContinuation(cursorAt("nur text")), null);
});

test("marker without following space is not a list item", () => {
    assert.equal(computeListContinuation(cursorAt("-keine liste")), null);
});

test("modifier keys are not handled", () => {
    for (const extra of [{ metaKey: true }, { ctrlKey: true }, { shiftKey: true }, { altKey: true }]) {
        assert.equal(computeListContinuation({ ...cursorAt("- item"), ...extra }), null);
    }
});

test("non-Enter keys are not handled", () => {
    assert.equal(computeListContinuation({ ...cursorAt("- item"), key: "Tab" }), null);
});

test("collapsed selection is required", () => {
    assert.equal(
        computeListContinuation({
            content: "- item",
            selectionStart: 2,
            selectionEnd: 4,
            key: "Enter",
        }),
        null,
    );
});

test("enter between two list items continues the touched item", () => {
    const content = "- erste\n- zweite";
    const result = computeListContinuation(cursorAt(content, 7));
    assert.deepEqual(result, {
        nextContent: "- erste\n- \n- zweite",
        nextCursorPosition: 10,
    });
});

test("out-of-range cursor is rejected", () => {
    assert.equal(
        computeListContinuation({ content: "- x", selectionStart: 99, selectionEnd: 99, key: "Enter" }),
        null,
    );
});
