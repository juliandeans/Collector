import test from "node:test";
import assert from "node:assert/strict";
import { markdownToHtml } from "../src/lib/reader/contentProcessing.js";
import { paragraphToMarkdownLine } from "../src/lib/reader/editorSerialization.js";

test("an indented paragraph keeps its leading whitespace in the rendered text", () => {
    assert.equal(markdownToHtml("    Milch"), "<p>    Milch</p>");
});

test("saving keeps the indentation of a paragraph under a list item", () => {
    assert.equal(paragraphToMarkdownLine("    Milch nicht vergessen"), "    Milch nicht vergessen");
    assert.equal(paragraphToMarkdownLine("\tMilch"), "\tMilch");
    assert.equal(paragraphToMarkdownLine("\t  Milch"), "\t  Milch");
});

test("saving still drops trailing whitespace", () => {
    assert.equal(paragraphToMarkdownLine("  text   "), "  text");
    assert.equal(paragraphToMarkdownLine("text "), "text");
});

test("a whitespace-only paragraph saves as an empty line", () => {
    assert.equal(paragraphToMarkdownLine("   "), "");
    assert.equal(paragraphToMarkdownLine("\t "), "");
    assert.equal(paragraphToMarkdownLine(""), "");
});

test("non-breaking spaces from the editor become plain spaces in the indentation", () => {
    assert.equal(paragraphToMarkdownLine("  Milch"), "  Milch");
    assert.equal(paragraphToMarkdownLine(" a b"), " a b");
});

test("unindented paragraphs are unchanged", () => {
    assert.equal(paragraphToMarkdownLine("plain text"), "plain text");
});
