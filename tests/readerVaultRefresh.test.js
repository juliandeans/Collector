import test from "node:test";
import assert from "node:assert/strict";
import { findExactWikilinkNote, resolveWikilink } from "../src/lib/reader/navigation.js";
import {
    handleNavigateToWikilink,
    openPalette,
    refreshVaultNotesInBackground,
} from "../src/lib/reader/navigationGlue.js";
import { createTab } from "../src/lib/reader/tabState.js";

const FOO_BAR = { name: "Foo Bar", relative_path: "Foo Bar.md", path: "/v/Foo Bar.md" };
const FOO = { name: "Foo", relative_path: "Inbox/Foo.md", path: "/v/Inbox/Foo.md" };

function readerBag({ notes = [], refreshResult = notes, refreshError = null } = {}) {
    const calls = { refresh: [], status: [], loadTab: [], paletteShown: false };
    let vaultNotes = notes;
    let tabs = [createTab({ kind: "opened", path: "Start.md", label: "Start", isPinned: false })];
    const bag = {
        calls,
        getVaultNotes: () => vaultNotes,
        refreshVaultNotes: async (mode) => {
            calls.refresh.push(mode);
            if (refreshError) throw refreshError;
            vaultNotes = refreshResult;
            return refreshResult;
        },
        getTabs: () => tabs,
        setTabs: (value) => {
            tabs = value;
        },
        getActiveTabIndex: () => 0,
        getAppSettings: () => ({ vault_name: "Vault", vault_path: "/v" }),
        showStatus: (message, kind) => calls.status.push({ message, kind }),
        forceSave: async () => {},
        loadTab: async (index) => calls.loadTab.push(index),
        activateTab: async () => {},
        saveScrollPosition: () => {},
        setShowPalette: (value) => {
            calls.paletteShown = value;
        },
        setPaletteQuery: () => {},
        setSelectedPaletteIndex: () => {},
        focusPaletteInput: () => {},
    };
    return bag;
}

test("findExactWikilinkNote ignores substring matches", () => {
    assert.equal(findExactWikilinkNote("Foo", [FOO_BAR]), null);
    assert.equal(findExactWikilinkNote("foo", [FOO_BAR, FOO]), FOO);
    assert.equal(findExactWikilinkNote("Inbox/Foo", [FOO]), FOO);
    assert.equal(findExactWikilinkNote("Foo|Alias", [FOO]), FOO);
    assert.equal(resolveWikilink("Foo", [FOO_BAR]), FOO_BAR, "fuzzy fallback unchanged");
});

test("openPalette shows the palette before the refresh resolves", async () => {
    const bag = readerBag({ notes: [FOO_BAR] });
    let release;
    bag.refreshVaultNotes = (mode) => {
        bag.calls.refresh.push(mode);
        return new Promise((resolve) => {
            release = () => resolve([FOO_BAR, FOO]);
        });
    };

    openPalette(bag);

    assert.equal(bag.calls.paletteShown, true);
    assert.deepEqual(bag.calls.refresh, ["ifStale"]);
    release();
});

test("openPalette refresh error with existing notes shows no status", async () => {
    const bag = readerBag({ notes: [FOO_BAR], refreshError: new Error("offline") });

    openPalette(bag);
    await new Promise((resolve) => setImmediate(resolve));

    assert.deepEqual(bag.calls.status, []);
});

test("openPalette refresh error with no notes shows an error status", async () => {
    const bag = readerBag({ notes: [], refreshError: new Error("offline") });

    openPalette(bag);
    await new Promise((resolve) => setImmediate(resolve));

    assert.equal(bag.calls.status.length, 1);
    assert.equal(bag.calls.status[0].kind, "error");
});

test("refreshVaultNotesInBackground uses ifStale and swallows errors", async () => {
    const bag = readerBag({ notes: [FOO_BAR], refreshError: new Error("offline") });

    refreshVaultNotesInBackground(bag);
    await new Promise((resolve) => setImmediate(resolve));

    assert.deepEqual(bag.calls.refresh, ["ifStale"]);
    assert.deepEqual(bag.calls.status, []);
});

test("exact match in the current list navigates without refreshing", async () => {
    const bag = readerBag({ notes: [FOO] });

    await handleNavigateToWikilink("Foo", false, bag);

    assert.deepEqual(bag.calls.refresh, []);
    assert.equal(bag.getTabs()[0].path, "Inbox/Foo.md");
});

test("fuzzy-only match triggers forced refresh and opens the exact note", async () => {
    const bag = readerBag({ notes: [FOO_BAR], refreshResult: [FOO_BAR, FOO] });

    await handleNavigateToWikilink("Foo", false, bag);

    assert.deepEqual(bag.calls.refresh, ["force"]);
    assert.equal(bag.getTabs()[0].path, "Inbox/Foo.md");
});

test("unknown link after forced refresh reports not found", async () => {
    const bag = readerBag({ notes: [FOO_BAR], refreshResult: [FOO_BAR] });

    await handleNavigateToWikilink("Missing", false, bag);

    assert.deepEqual(bag.calls.refresh, ["force"]);
    assert.deepEqual(bag.calls.status, [{ message: "Note not found: [[Missing]]", kind: "error" }]);
});

test("forced refresh failure falls back to the current list", async () => {
    const bag = readerBag({ notes: [FOO_BAR], refreshError: new Error("offline") });

    await handleNavigateToWikilink("Foo", false, bag);

    assert.equal(bag.getTabs()[0].path, "Foo Bar.md");
});

test("forced refresh failure with an empty list shows the error", async () => {
    const bag = readerBag({ notes: [], refreshError: new Error("offline") });

    await handleNavigateToWikilink("Foo", false, bag);

    assert.equal(bag.calls.status.length, 1);
    assert.equal(bag.calls.status[0].kind, "error");
    assert.deepEqual(bag.calls.loadTab, []);
});
