import test from "node:test";
import assert from "node:assert/strict";
import {
    createVaultNotesRefresher,
    findNoteIndex,
    vaultPathChanged,
} from "../src/lib/vaultNotesRefresh.js";

function deferred() {
    let resolve;
    let reject;
    const promise = new Promise((res, rej) => {
        resolve = res;
        reject = rej;
    });
    return { promise, resolve, reject };
}

test("applies the fetched notes and passes the mode through", async () => {
    const modes = [];
    let applied = null;
    const refresh = createVaultNotesRefresher({
        fetchNotes: async (mode) => {
            modes.push(mode);
            return [{ name: "A" }];
        },
        setNotes: (notes) => {
            applied = notes;
        },
    });

    const result = await refresh("force");

    assert.deepEqual(modes, ["force"]);
    assert.deepEqual(applied, [{ name: "A" }]);
    assert.deepEqual(result, [{ name: "A" }]);
});

test("defaults to ifStale", async () => {
    const modes = [];
    const refresh = createVaultNotesRefresher({
        fetchNotes: async (mode) => {
            modes.push(mode);
            return [];
        },
        setNotes: () => {},
    });

    await refresh();

    assert.deepEqual(modes, ["ifStale"]);
});

test("older response arriving late is not applied", async () => {
    const first = deferred();
    const second = deferred();
    const pending = [first, second];
    const appliedLists = [];
    const refresh = createVaultNotesRefresher({
        fetchNotes: () => pending.shift().promise,
        setNotes: (notes) => appliedLists.push(notes),
    });

    const olderCall = refresh("ifStale");
    const newerCall = refresh("ifStale");
    second.resolve([{ name: "New" }]);
    await newerCall;
    first.resolve([{ name: "Old" }]);
    const olderResult = await olderCall;

    assert.deepEqual(appliedLists, [[{ name: "New" }]]);
    assert.deepEqual(olderResult, [{ name: "Old" }]);
});

test("errors propagate and leave the list untouched", async () => {
    let setCalls = 0;
    const refresh = createVaultNotesRefresher({
        fetchNotes: async () => {
            throw new Error("walk failed");
        },
        setNotes: () => {
            setCalls += 1;
        },
    });

    await assert.rejects(refresh("ifStale"), /walk failed/);
    assert.equal(setCalls, 0);
});

test("findNoteIndex keeps the selected note by path", () => {
    const a = { name: "A", path: "/v/A.md" };
    const b = { name: "B", path: "/v/B.md" };
    const inserted = { name: "Aa", path: "/v/Aa.md" };

    assert.equal(findNoteIndex([a, inserted, b], { ...b }), 2);
    assert.equal(findNoteIndex([a, inserted], b), 0, "selected note gone");
    assert.equal(findNoteIndex([a, b], undefined), 0, "nothing selected");
});

test("vaultPathChanged compares vault_path only", () => {
    assert.equal(vaultPathChanged({ vault_path: "/a" }, { vault_path: "/a", font_size: 3 }), false);
    assert.equal(vaultPathChanged({ vault_path: "/a" }, { vault_path: "/b" }), true);
    assert.equal(vaultPathChanged({}, { vault_path: "" }), false);
    assert.equal(vaultPathChanged(undefined, { vault_path: "/b" }), true);
});
