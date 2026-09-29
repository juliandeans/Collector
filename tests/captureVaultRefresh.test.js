import test from "node:test";
import assert from "node:assert/strict";
import { refreshPickerNotes } from "../src/lib/app/appendPicker.js";
import { applySettingsUpdate, buildInitialAppSettings } from "../src/lib/app/settingsBridge.js";

test("capture settings carry vault_path", () => {
    const initial = buildInitialAppSettings({ vault_path: "/vault/a" });
    assert.equal(initial.vault_path, "/vault/a");
    assert.equal(applySettingsUpdate(initial, { vault_path: "/vault/b" }).vault_path, "/vault/b");
    assert.equal(applySettingsUpdate(initial, {}).vault_path, "/vault/a");
});

test("refreshPickerNotes requests an ifStale refresh", async () => {
    const modes = [];
    await refreshPickerNotes({
        refresh: async (mode) => {
            modes.push(mode);
            return [];
        },
        getNotes: () => [],
        showStatus: () => assert.fail("no status expected"),
    });

    assert.deepEqual(modes, ["ifStale"]);
});

test("refresh error with existing notes shows no status", async () => {
    const statuses = [];
    await refreshPickerNotes({
        refresh: async () => {
            throw new Error("offline");
        },
        getNotes: () => [{ name: "A" }],
        showStatus: (...args) => statuses.push(args),
    });

    assert.deepEqual(statuses, []);
});

test("refresh error with no notes shows the load error", async () => {
    const statuses = [];
    await refreshPickerNotes({
        refresh: async () => {
            throw new Error("offline");
        },
        getNotes: () => [],
        showStatus: (...args) => statuses.push(args),
    });

    assert.deepEqual(statuses, [["✗ Could not load vault notes", "error"]]);
});
