import test from "node:test";
import assert from "node:assert/strict";
import { markdownToHtml } from "../src/lib/reader/contentProcessing.js";
import { listItemToMarkdown } from "../src/lib/reader/editorSerialization.js";
import { parseListLine } from "../src/lib/capture/list-continuation.js";

test("nested list from the user feedback renders both levels as list items", () => {
    assert.equal(
        markdownToHtml("- parent\n   - child"),
        '<p class="list-item" data-indent="" data-marker="-" data-spacing=" " style="--list-indent: 0">parent</p>' +
            '<p class="list-item" data-indent="   " data-marker="-" data-spacing=" " style="--list-indent: 3">child</p>',
    );
});

test("star, plus and numbered markers are list items", () => {
    const html = markdownToHtml("* a\n+ b\n1. c");
    assert.match(html, /data-marker="\*"[^>]*>a<\/p>/);
    assert.match(html, /data-marker="\+"[^>]*>b<\/p>/);
    assert.match(html, /data-marker="1\."[^>]*>c<\/p>/);
});

test("tab indentation counts as 4 columns and is kept raw", () => {
    assert.equal(
        markdownToHtml("\t- tab child"),
        '<p class="list-item" data-indent="\t" data-marker="-" data-spacing=" " style="--list-indent: 4">tab child</p>',
    );
});

test("task items keep checkbox state and raw parts", () => {
    assert.equal(
        markdownToHtml("  - [X] done"),
        '<p class="list-item task" data-indent="  " data-marker="-" data-spacing=" " style="--list-indent: 2" data-checkbox="X" data-checkbox-spacing=" ">' +
            '<input type="checkbox" class="md-checkbox" contenteditable="false" checked> done</p>',
    );
    assert.match(markdownToHtml("- [ ] open"), /data-checkbox=" "[^>]*><input type="checkbox" class="md-checkbox" contenteditable="false"> open<\/p>/);
});

test("list-like lines that are not lists stay as before", () => {
    assert.equal(markdownToHtml("---"), "<hr>");
    assert.doesNotMatch(markdownToHtml("-foo"), /list-item/);
    assert.doesNotMatch(markdownToHtml("1.5 kg"), /list-item/);
    assert.doesNotMatch(markdownToHtml("**bold** text"), /list-item/);
});

// Minimal stand-in for element.dataset: reads data-* attributes from the
// rendered HTML string (tests have no DOM).
function datasetOf(html) {
    const dataset = {};
    for (const [, name, value] of html.matchAll(/data-([\w-]+)="([^"]*)"/g)) {
        const key = name.replace(/-(\w)/g, (_, c) => c.toUpperCase());
        dataset[key] = value.replace(/&quot;/g, '"').replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");
    }
    return dataset;
}

const SERIALIZE_ROUND_TRIP = [
    ["- parent", null, "parent"],
    ["   - child", null, "child"],
    ["\t\t* deep", null, "deep"],
    ["+   wide", null, "wide"],
    ["3. third", null, "third"],
    ["  - [ ] open", false, "open"],
    ["- [X] done", true, "done"],
    ["- [x]tight", true, "tight"],
    ["- [x]  wide box", true, "wide box"],
    ["- [ ]\ttab box", false, "tab box"],
];

test("task item text never starts with whitespace", () => {
    assert.equal(parseListLine("- [x]   text").text, "text");
    assert.equal(parseListLine("- [x]   text").checkboxSpacing, "   ");
});

for (const [line, checked, text] of SERIALIZE_ROUND_TRIP) {
    test(`render + serialize round trip: ${JSON.stringify(line)}`, () => {
        const dataset = datasetOf(markdownToHtml(line));
        assert.equal(listItemToMarkdown(dataset, { checked, text }), line);
    });
}

test("toggling a task checkbox writes the new state", () => {
    const openSet = datasetOf(markdownToHtml("- [ ] a"));
    assert.equal(listItemToMarkdown(openSet, { checked: true, text: "a" }), "- [x] a");
    const doneSet = datasetOf(markdownToHtml("- [X] a"));
    assert.equal(listItemToMarkdown(doneSet, { checked: false, text: "a" }), "- [ ] a");
});

test("elements without list data fall back to a plain dash item", () => {
    assert.equal(listItemToMarkdown({}, { checked: null, text: "x" }), "- x");
    assert.equal(listItemToMarkdown({}, { checked: true, text: "x" }), "- [x] x");
});
