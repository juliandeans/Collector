import test from "node:test";
import assert from "node:assert/strict";
import { createIndentUnitRefresher, indentUnitFrom, loadIndentUnit } from "../src/lib/obsidianIndent.js";
import { computeListContinuation, computeListIndentation } from "../src/lib/capture/list-continuation.js";
import { computeListKeyEdit } from "../src/lib/reader/editorListKeys.js";

test("indentUnitFrom follows Obsidian: tab by default, 4 spaces when tabs are off", () => {
    assert.equal(indentUnitFrom({ use_tab: true }), "\t");
    assert.equal(indentUnitFrom({ use_tab: false }), "    ");
    assert.equal(indentUnitFrom({ use_tab: "no" }), "\t");
    assert.equal(indentUnitFrom({}), "\t");
    assert.equal(indentUnitFrom(null), "\t");
    assert.equal(indentUnitFrom(undefined), "\t");
});

test("loadIndentUnit falls back to a tab when the command fails", async () => {
    assert.equal(await loadIndentUnit(async () => ({ use_tab: false })), "    ");
    assert.equal(await loadIndentUnit(async () => { throw new Error("no vault"); }), "\t");
});

test("createIndentUnitRefresher applies only the newest result", async () => {
    const applied = [];
    let resolveSlow;
    const responses = [
        new Promise((resolve) => { resolveSlow = resolve; }),
        Promise.resolve({ use_tab: true }),
    ];
    let call = 0;
    const refresh = createIndentUnitRefresher({
        invoke: () => responses[call++],
        setUnit: (unit) => applied.push(unit),
    });
    const older = refresh();
    const newer = refresh();
    await newer;
    resolveSlow({ use_tab: false });
    await older;
    assert.deepEqual(applied, ["\t"]);
});

function tab(content, { shiftKey = false, indentUnit, caret = content.length } = {}) {
    return computeListIndentation({
        content,
        selectionStart: caret,
        selectionEnd: caret,
        key: "Tab",
        shiftKey,
        indentUnit,
    });
}

test("Tab inserts a tab character by default", () => {
    assert.deepEqual(tab("- a"), { nextContent: "\t- a", nextCursorPosition: 4 });
});

test("Tab inserts the configured spaces", () => {
    assert.deepEqual(tab("- a", { indentUnit: "  " }), { nextContent: "  - a", nextCursorPosition: 5 });
    assert.deepEqual(tab("1. a\n2. b", { indentUnit: "    " }), {
        nextContent: "1. a\n    1. b",
        nextCursorPosition: 13,
    });
});

test("Tab adds one more level to an already indented item", () => {
    assert.deepEqual(tab("- a\n\t- b"), { nextContent: "- a\n\t\t- b", nextCursorPosition: 9 });
});

test("Shift+Tab without a parent removes one trailing tab", () => {
    assert.deepEqual(tab("\t\t- a", { shiftKey: true }), { nextContent: "\t- a", nextCursorPosition: 4 });
});

test("Shift+Tab without a parent removes one unit of spaces", () => {
    assert.deepEqual(tab("      - a", { shiftKey: true, indentUnit: "    " }), {
        nextContent: "  - a",
        nextCursorPosition: 5,
    });
    assert.deepEqual(tab("    - a", { shiftKey: true, indentUnit: "  " }), {
        nextContent: "  - a",
        nextCursorPosition: 5,
    });
});

test("Shift+Tab with a parent still lands on the parent level", () => {
    assert.deepEqual(tab("- a\n\t\t- b", { shiftKey: true }), {
        nextContent: "- a\n- b",
        nextCursorPosition: 7,
    });
});

test("Enter on an empty nested item forwards the unit (outdent via parent)", () => {
    const content = "1. a\n\t1. b\n\t2. ";
    assert.deepEqual(
        computeListContinuation({
            content,
            selectionStart: content.length,
            selectionEnd: content.length,
            key: "Enter",
            indentUnit: "  ",
        }),
        { nextContent: "1. a\n\t1. b\n2. ", nextCursorPosition: 14 },
    );
});

test("Reader forwards the indent unit", () => {
    assert.deepEqual(
        computeListKeyEdit({
            lineText: "- a",
            selectionStart: 3,
            selectionEnd: 3,
            key: "Tab",
            indentUnit: "  ",
        }),
        { lines: ["  - a"], caretLine: 0, caretOffset: 5 },
    );
});
