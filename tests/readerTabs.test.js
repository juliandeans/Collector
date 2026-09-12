import test from "node:test";
import assert from "node:assert/strict";
import { rebuildTabsFromSettings } from "../src/lib/reader/tabState.js";

const PINNED = [
    { path: "Notes/alpha.md", label: "alpha", icon: "" },
    { path: "Notes/beta.md", label: "beta", icon: "" },
];

test("default (setting undefined, legacy config): daily tab is present and first", () => {
    const state = rebuildTabsFromSettings({ pinned_notes: PINNED }, [], {
        preserveOpened: false,
        dailyPath: "Daily/2026-09-13.md",
    });

    assert.equal(state.tabs[0].kind, "daily");
    assert.equal(state.tabs.length, 3);
});

test("reader_show_daily_note: true keeps the daily tab first", () => {
    const state = rebuildTabsFromSettings(
        { pinned_notes: PINNED, reader_show_daily_note: true },
        [],
        { preserveOpened: false, dailyPath: "Daily/2026-09-13.md" },
    );

    assert.equal(state.tabs[0].kind, "daily");
    assert.equal(state.tabs.length, 3);
});

test("reader_show_daily_note: false hides the daily tab", () => {
    const state = rebuildTabsFromSettings(
        { pinned_notes: PINNED, reader_show_daily_note: false },
        [],
        { preserveOpened: false, dailyPath: "Daily/2026-09-13.md" },
    );

    assert.equal(state.tabs.some((tab) => tab.kind === "daily"), false);
    assert.deepEqual(
        state.tabs.map((tab) => tab.path),
        ["Notes/alpha.md", "Notes/beta.md"],
    );
    assert.equal(state.activeTabIndex, 0);
});

test("pinned tab order follows the settings order (sorting support)", () => {
    const state = rebuildTabsFromSettings(
        { pinned_notes: [PINNED[1], PINNED[0]], reader_show_daily_note: false },
        [],
        { preserveOpened: false, dailyPath: "" },
    );

    assert.deepEqual(
        state.tabs.map((tab) => tab.path),
        ["Notes/beta.md", "Notes/alpha.md"],
    );
});

test("disabling daily note preserves opened tabs and drops the daily tab", () => {
    const currentTabs = [
        {
            kind: "daily",
            path: "Daily/2026-09-13.md",
            label: "Daily",
            isPinned: true,
            loaded: true,
            content: "daily",
            history: [],
        },
        {
            kind: "opened",
            path: "Notes/open.md",
            label: "open",
            isPinned: false,
            loaded: true,
            content: "opened content",
            history: [],
        },
    ];

    const state = rebuildTabsFromSettings(
        { pinned_notes: [], reader_show_daily_note: false },
        currentTabs,
        { preserveOpened: true, dailyPath: "Daily/2026-09-13.md" },
    );

    assert.equal(state.tabs.some((tab) => tab.kind === "daily"), false);
    assert.deepEqual(
        state.tabs.map((tab) => tab.kind),
        ["opened"],
    );
});

test("empty-day path with daily hidden: no phantom daily tab", () => {
    const state = rebuildTabsFromSettings(
        { pinned_notes: [], reader_show_daily_note: false },
        [],
        { preserveOpened: false, dailyPath: "" },
    );

    assert.equal(state.tabs.length, 0);
});
