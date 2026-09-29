import test from "node:test";
import assert from "node:assert/strict";
import {
    parseListLine,
    formatListLine,
} from "../src/lib/capture/list-continuation.js";

const ROUND_TRIP_LINES = [
    "- item",
    "* item",
    "+ item",
    "1. item",
    "12. item",
    "   - child",
    "\t- tab child",
    "\t\t* two tabs",
    "-   wide spacing",
    "-\ttab after marker",
    "- [ ] open task",
    "- [x] done task",
    "- [X] done upper",
    "  * [ ] nested task",
    "- [x]no space",
    "- [x]  two spaces",
    "- [ ]\ttab after box",
    "- ",
    "- **bold** and [[link]]",
];

for (const line of ROUND_TRIP_LINES) {
    test(`round trip: ${JSON.stringify(line)}`, () => {
        const parts = parseListLine(line);
        assert.notEqual(parts, null);
        assert.equal(formatListLine(parts), line);
    });
}

test("parseListLine splits indent, marker, spacing, checkbox and text", () => {
    assert.deepEqual(parseListLine("\t  1. [x] done"), {
        indent: "\t  ",
        marker: "1.",
        spacing: " ",
        checkbox: "x",
        checkboxSpacing: " ",
        text: "done",
    });
    assert.deepEqual(parseListLine("   - child"), {
        indent: "   ",
        marker: "-",
        spacing: " ",
        checkbox: null,
        checkboxSpacing: "",
        text: "child",
    });
});

for (const line of ["---", "***", "**bold** text", "-foo", "1.5 kg", "plain", "", "# heading", "> quote"]) {
    test(`not a list line: ${JSON.stringify(line)}`, () => {
        assert.equal(parseListLine(line), null);
    });
}

test("parseListLine rejects non-strings", () => {
    assert.equal(parseListLine(undefined), null);
    assert.equal(parseListLine(null), null);
});

test("formatListLine fills defaults", () => {
    assert.equal(formatListLine({ text: "x" }), "- x");
    assert.equal(formatListLine({ checkbox: " ", checkboxSpacing: " ", text: "t" }), "- [ ] t");
});
