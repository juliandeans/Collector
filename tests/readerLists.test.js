import test from "node:test";
import assert from "node:assert/strict";
import { markdownToHtml } from "../src/lib/reader/contentProcessing.js";

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
