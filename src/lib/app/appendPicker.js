import { formatEntryHeader, getVaultNotePath, parseHeadings } from "../capture/capture-utils.js";

export function computeClosedPickerState() {
    return {
        showAppendPicker: false,
        appendPickerStep: 1,
        appendPickerSelectedNote: null,
        appendPickerHeadings: [],
        appendPickerQuery: "",
        appendPickerSelectedIndex: 0,
        appendPickerHeadingIndex: 0,
    };
}

export function computeStep1PickerState() {
    return {
        appendPickerStep: 1,
        appendPickerSelectedNote: null,
        appendPickerHeadings: [],
        appendPickerHeadingIndex: 0,
    };
}

export function computeOpenPickerState() {
    return {
        showAppendPicker: true,
        appendPickerStep: 1,
        appendPickerSelectedNote: null,
        appendPickerHeadings: [],
        appendPickerQuery: "",
        appendPickerSelectedIndex: 0,
        appendPickerHeadingIndex: 0,
    };
}

export function loadVaultNotesIfEmpty({ invoke, getNotes, setNotes, showStatus }) {
    if (getNotes().length !== 0) return;

    invoke("list_vault_notes")
        .then((notes) => {
            setNotes(notes);
        })
        .catch(() => {
            showStatus("✗ Could not load vault notes", "error");
        });
}

export function preloadVaultNotes({ invoke, setNotes }) {
    // Preload vault notes so the append picker can open immediately.
    invoke("list_vault_notes")
        .then((notes) => {
            setNotes(notes);
        })
        .catch(() => {
            // Non-fatal: the picker can still lazy-load later.
        });
}

export async function insertAfterHeading({
    invoke,
    notePath,
    heading,
    text,
    getEntryHeaderTemplate,
}) {
    const fileContent = await invoke("read_note_file", { path: notePath });
    const lines = fileContent.split("\n");

    let insertAt = lines.length;
    for (let i = heading.lineIndex + 1; i < lines.length; i += 1) {
        const match = lines[i].match(/^(#{1,6})\s/);
        if (match && match[1].length <= heading.level) {
            insertAt = i;
            break;
        }
    }

    const entryLines = [
        formatEntryHeader(getEntryHeaderTemplate()),
        ...text.split("\n"),
        "",
    ];
    const separator =
        insertAt > 0 && lines[insertAt - 1]?.trim() !== "" ? [""] : [];

    lines.splice(insertAt, 0, ...separator, ...entryLines);

    await invoke("write_note_file", {
        path: notePath,
        content: lines.join("\n"),
    });
}

export async function selectNote(
    note,
    {
        invoke,
        getSelectedNote,
        setSelectedNote,
        setHeadings,
        setHeadingIndex,
        setStep,
    },
) {
    setSelectedNote(note);
    const notePath = getVaultNotePath(note);

    if (!notePath) {
        setHeadings([]);
        setHeadingIndex(0);
        setStep(2);
        return;
    }

    // Load headings BEFORE transitioning to step 2.
    // This prevents a race condition where the IPC call and UI transition
    // happen simultaneously, which can freeze WKWebView on macOS.
    let loadedHeadings = [];
    try {
        const fileContent = await invoke("read_note_file", { path: notePath });
        // Check if user closed picker or selected a different note while loading
        if (getVaultNotePath(getSelectedNote()) !== notePath) return;
        loadedHeadings = parseHeadings(fileContent);
    } catch (e) {
        if (getVaultNotePath(getSelectedNote()) !== notePath) return;
        loadedHeadings = [];
    }

    setHeadings(loadedHeadings);
    setHeadingIndex(0);
    // setTimeout(0) gives WKWebView a single event-loop tick
    // between loading headings and transitioning to step 2.
    // Without this pause, the simultaneous Svelte DOM update
    // and IPC completion cause WKWebView to freeze on macOS.
    await new Promise((resolve) => setTimeout(resolve, 0));
    setStep(2);
}

export async function appendToNote(
    note,
    heading,
    {
        invoke,
        getContent,
        isLoading,
        setLoading,
        getEntryHeaderTemplate,
        freeBlobUrls,
        resetCaptureState,
        deferHideCapture,
        showStatus,
    },
) {
    const notePath = getVaultNotePath(note);
    const content = getContent();
    if (!notePath || !content.trim() || isLoading) return;

    setLoading(true);

    try {
        if (!heading) {
            await invoke("append_to_note", {
                path: notePath,
                text: content.trim(),
            });
        } else {
            await insertAfterHeading({
                invoke,
                notePath,
                heading,
                text: content.trim(),
                getEntryHeaderTemplate,
            });
        }

        freeBlobUrls();
        resetCaptureState();
        deferHideCapture();
    } catch (e) {
        showStatus("✗ " + e.toString(), "error");
        setLoading(false);
    }
}
