import test from "node:test";
import assert from "node:assert/strict";
import { findTagTrigger, getTagAutocompleteResults } from "../src/lib/tags.js";
import { computeTagInsertion } from "../src/lib/capture/wiki-utils.js";
import { computeInputState } from "../src/lib/app/wikiAutocomplete.js";

test("findTagTrigger detects valid boundaries and rejects invalid ones", () => {
    assert.deepEqual(findTagTrigger("#alpha", 6), {
        triggerIndex: 0,
        query: "alpha",
    });
    assert.deepEqual(findTagTrigger("text (#alpha", 12), {
        triggerIndex: 6,
        query: "alpha",
    });
    assert.equal(findTagTrigger("word#alpha", 10), null);
    assert.equal(findTagTrigger("##heading", 9), null);
    assert.equal(findTagTrigger("#alpha beta", 11), null);
});

test("findTagTrigger allows an empty query for a bare # like Obsidian", () => {
    assert.deepEqual(findTagTrigger("note #", 6), {
        triggerIndex: 5,
        query: "",
    });
    assert.deepEqual(findTagTrigger("note #a", 6), {
        triggerIndex: 5,
        query: "",
    });
    assert.equal(findTagTrigger("note # ", 6), null);
    assert.equal(findTagTrigger("note ##", 6), null);
    assert.equal(findTagTrigger("#\nnext", 1), null);
});

test("computeTagInsertion replaces the trigger and adds a trailing space", () => {
    assert.deepEqual(computeTagInsertion("hello #ta tail", 9, 2, "task"), {
        before: "hello ",
        after: " tail",
        insertion: "#task ",
        newPosition: 12,
    });

    assert.deepEqual(computeTagInsertion("#a", 2, 1, "alpha"), {
        before: "",
        after: "",
        insertion: "#alpha ",
        newPosition: 7,
    });
});

test("getTagAutocompleteResults preserves cache order while filtering and limiting", () => {
    const tags = ["Parent/Child", "alpha", "Beta"];

    assert.deepEqual(getTagAutocompleteResults("", tags, 2), [
        "Parent/Child",
        "alpha",
    ]);
    assert.deepEqual(getTagAutocompleteResults("PARENT", tags), [
        "Parent/Child",
    ]);
    assert.deepEqual(getTagAutocompleteResults("al", tags, 1), ["alpha"]);
});

test("computeInputState selects the later tag trigger and normalizes tag matches", () => {
    const originalWindow = globalThis.window;
    const originalDocument = globalThis.document;
    const originalGetComputedStyle = globalThis.getComputedStyle;
    const createElement = () => ({
        style: {},
        offsetLeft: 0,
        offsetTop: 0,
        appendChild() {},
        remove() {},
    });

    globalThis.window = { innerWidth: 1000 };
    globalThis.document = {
        createElement,
        body: { appendChild() {} },
    };
    globalThis.getComputedStyle = () =>
        new Proxy(
            {},
            {
                get: () => "0px",
            },
        );

    try {
        const value = "[[Old #ta";
        const state = computeInputState({
            textarea: {
                value,
                selectionStart: value.length,
                offsetWidth: 200,
                scrollLeft: 0,
                getBoundingClientRect: () => ({ left: 10, top: 20 }),
            },
            notes: [],
            tags: ["task", "target"],
            limit: 20,
        });

        assert.equal(state.mode, "tag");
        assert.deepEqual(state.matches, [
            { name: "task", path: "task" },
            { name: "target", path: "target" },
        ]);
        assert.equal(state.anchorPos, 6);
    } finally {
        globalThis.window = originalWindow;
        globalThis.document = originalDocument;
        globalThis.getComputedStyle = originalGetComputedStyle;
    }
});
