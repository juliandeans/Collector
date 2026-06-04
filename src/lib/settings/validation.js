const PATH_TOKEN_PATTERN = /(YYYY|MM|DD|HH|hh|mm|ss)/;
const DAILY_DATE_TOKEN_PATTERN = /(YYYY|MM|DD)/;
const HEX_COLOR_PATTERN = /^#[0-9A-Fa-f]{6}$/;

const VALID_MODIFIERS = new Set([
    "cmd",
    "command",
    "commandorcontrol",
    "ctrl",
    "control",
    "shift",
    "alt",
    "option",
    "opt",
    "super",
]);

const VALID_KEYS = new Set([
    ..."ABCDEFGHIJKLMNOPQRSTUVWXYZ".split(""),
    ..."0123456789".split(""),
    "Space",
    "Tab",
    "Enter",
    "Escape",
    "Backspace",
    "Delete",
    "Up",
    "Down",
    "Left",
    "Right",
    "Home",
    "End",
    "PageUp",
    "PageDown",
]);

for (let i = 1; i <= 12; i += 1) {
    VALID_KEYS.add(`F${i}`);
}

function addIssue(fields, field, type, message) {
    const existing = fields[field];
    if (existing?.type === "error" && type !== "error") return;
    fields[field] = { type, message };
}

function getString(value) {
    return String(value ?? "").trim();
}

function isAbsolutePath(path) {
    return (
        path.startsWith("/") ||
        path.startsWith("\\\\") ||
        /^[A-Za-z]:[\\/]/.test(path)
    );
}

function hasParentSegment(path) {
    return path
        .replace(/\\/g, "/")
        .split("/")
        .some((segment) => segment === "..");
}

function validateRelativeFolder(fields, settings, field, label, options = {}) {
    const value = getString(settings[field]);
    if (!value) {
        if (options.emptyWarning) {
            addIssue(fields, field, "warning", options.emptyWarning);
        }
        return;
    }

    if (value.includes("\\")) {
        addIssue(fields, field, "error", `${label} must use forward slashes.`);
        return;
    }

    if (isAbsolutePath(value) || hasParentSegment(value)) {
        addIssue(fields, field, "error", `${label} must stay inside the vault.`);
    }
}

function validateFilenameTemplate(fields, settings, field, label, options = {}) {
    const value = getString(settings[field]);
    if (!value) {
        addIssue(fields, field, "error", `${label} cannot be empty.`);
        return;
    }

    if (value.includes("/") || value.includes("\\") || value.includes("..")) {
        addIssue(
            fields,
            field,
            "error",
            `${label} must be a filename without path separators.`,
        );
        return;
    }

    if (options.dateTokensOnly && !DAILY_DATE_TOKEN_PATTERN.test(value)) {
        addIssue(
            fields,
            field,
            "warning",
            `${label} has no date token; files may reuse the same name.`,
        );
        return;
    }

    if (!options.dateTokensOnly && !PATH_TOKEN_PATTERN.test(value)) {
        addIssue(
            fields,
            field,
            "warning",
            `${label} has no date or time token; files may collide.`,
        );
        return;
    }

    if (options.warnWithoutSeconds && !value.includes("ss")) {
        addIssue(
            fields,
            field,
            "warning",
            `${label} has no seconds token; rapid captures may collide.`,
        );
    }
}

function validateNumberRange(fields, settings, field, label, min, max) {
    const rawValue = settings[field];
    if (rawValue === "" || rawValue === null || rawValue === undefined) {
        addIssue(fields, field, "error", `${label} is required.`);
        return;
    }

    const value = Number(rawValue);
    if (!Number.isFinite(value) || !Number.isInteger(value)) {
        addIssue(fields, field, "error", `${label} must be a whole number.`);
        return;
    }

    if (value < min || value > max) {
        addIssue(fields, field, "error", `${label} must be between ${min} and ${max}.`);
    }
}

function validateColor(fields, settings, field, label) {
    const value = getString(settings[field]);
    if (!HEX_COLOR_PATTERN.test(value)) {
        addIssue(fields, field, "error", `${label} must be a hex color like #8b5cf6.`);
    }
}

function normalizeModifier(token) {
    const normalized = token.toLowerCase();
    if (normalized === "command") return "cmd";
    if (normalized === "control") return "ctrl";
    if (normalized === "option" || normalized === "opt") return "alt";
    return normalized;
}

function parseShortcut(shortcut) {
    const parts = getString(shortcut)
        .split("+")
        .map((part) => part.trim())
        .filter(Boolean);

    if (parts.length < 2) {
        return { ok: false, reason: "Shortcut must contain a modifier and a key." };
    }

    const modifiers = [];
    let key = "";

    for (const part of parts) {
        const lower = part.toLowerCase();
        if (VALID_MODIFIERS.has(lower)) {
            modifiers.push(normalizeModifier(part));
            continue;
        }

        if (key) {
            return { ok: false, reason: "Shortcut must contain only one key." };
        }

        key = part.length === 1 ? part.toUpperCase() : part;
    }

    if (modifiers.length === 0) {
        return { ok: false, reason: "Shortcut must include at least one modifier." };
    }

    if (!key) {
        return { ok: false, reason: "Shortcut must include a key." };
    }

    if (!VALID_KEYS.has(key)) {
        return { ok: false, reason: `"${key}" is not a supported shortcut key.` };
    }

    return {
        ok: true,
        key,
        modifiers: [...new Set(modifiers)].sort(),
    };
}

function canonicalShortcut(value, { commandOrControl = false } = {}) {
    const parsed = parseShortcut(value);
    if (!parsed.ok) return "";

    const modifiers = parsed.modifiers.map((modifier) => {
        if (
            commandOrControl &&
            (modifier === "cmd" ||
                modifier === "ctrl" ||
                modifier === "commandorcontrol")
        ) {
            return "primary";
        }
        return modifier;
    });

    return [...new Set(modifiers)].sort().join("+") + "+" + parsed.key.toLowerCase();
}

function validateShortcut(fields, settings, field, label) {
    const value = getString(settings[field]);
    if (!value) return;

    const parsed = parseShortcut(value);
    if (!parsed.ok) {
        addIssue(fields, field, "error", `${label}: ${parsed.reason}`);
    }
}

function addDuplicateShortcutErrors(fields, settings, pairs) {
    for (const [leftField, rightField, message] of pairs) {
        const left = getString(settings[leftField]);
        const right = getString(settings[rightField]);
        if (!left || !right) continue;

        if (
            canonicalShortcut(left, { commandOrControl: true }) ===
            canonicalShortcut(right, { commandOrControl: true })
        ) {
            addIssue(fields, leftField, "error", message);
            addIssue(fields, rightField, "error", message);
        }
    }
}

function addShortcutGroupCollisions(fields, settings, entries, message) {
    const seen = new Map();

    for (const { field } of entries) {
        const value = getString(settings[field]);
        if (!value || fields[field]?.type === "error") continue;

        const canonical = canonicalShortcut(value);
        if (!canonical) continue;

        const otherField = seen.get(canonical);
        if (otherField) {
            addIssue(fields, field, "error", message);
            addIssue(fields, otherField, "error", message);
        } else {
            seen.set(canonical, field);
        }
    }
}

function addReaderPrecedenceWarnings(fields, settings, entries) {
    const reservedKeys = new Set(["w", "f", "s"]);

    for (const { field, label } of entries) {
        const value = getString(settings[field]);
        if (!value || fields[field]?.type === "error") continue;

        const parsed = parseShortcut(value);
        if (!parsed.ok) continue;

        const hasPrimary =
            parsed.modifiers.includes("cmd") ||
            parsed.modifiers.includes("ctrl") ||
            parsed.modifiers.includes("commandorcontrol");
        const hasOnlyPrimary =
            hasPrimary &&
            !parsed.modifiers.includes("shift") &&
            !parsed.modifiers.includes("alt") &&
            !parsed.modifiers.includes("super") &&
            !parsed.modifiers.includes("commandorcontrol");
        const key = parsed.key.toLowerCase();

        if (
            hasOnlyPrimary &&
            (reservedKeys.has(key) || /^[1-9]$/.test(key))
        ) {
            addIssue(
                fields,
                field,
                "warning",
                `${label} is already used by a built-in Reader shortcut.`,
            );
        }
    }
}

export function validateSettings(settings = {}) {
    const fields = {};

    validateNumberRange(fields, settings, "window_width", "Capture width", 200, 800);
    validateNumberRange(fields, settings, "window_height", "Capture height", 200, 1200);
    validateNumberRange(fields, settings, "reader_width", "Reader width", 200, 800);
    validateNumberRange(fields, settings, "reader_height", "Reader height", 200, 1200);
    validateNumberRange(
        fields,
        settings,
        "edge_reaction_time_ms",
        "Edge reaction time",
        50,
        1000,
    );
    validateNumberRange(
        fields,
        settings,
        "note_edge_open_delay_ms",
        "Capture edge delay",
        50,
        10000,
    );
    validateNumberRange(
        fields,
        settings,
        "reader_edge_open_delay_ms",
        "Reader edge delay",
        50,
        10000,
    );
    validateNumberRange(
        fields,
        settings,
        "daily_note_create_timeout_ms",
        "Daily note wait timeout",
        1000,
        60000,
    );
    validateNumberRange(
        fields,
        settings,
        "autocomplete_results",
        "Max autocomplete results",
        5,
        50,
    );
    validateNumberRange(
        fields,
        settings,
        "compression_max_kb",
        "Max image size",
        50,
        2000,
    );
    validateNumberRange(fields, settings, "border_radius", "Corner radius", 0, 30);
    validateNumberRange(fields, settings, "font_size", "Font size", 10, 24);
    validateNumberRange(fields, settings, "overlay_strength", "Overlay strength", 0, 100);
    validateNumberRange(fields, settings, "window_blur", "Background blur", 0, 200);
    validateNumberRange(
        fields,
        settings,
        "window_saturation",
        "Background saturation",
        0,
        300,
    );
    validateNumberRange(
        fields,
        settings,
        "window_brightness",
        "Background brightness",
        -100,
        100,
    );

    if (!getString(settings.vault_name)) {
        addIssue(fields, "vault_name", "error", "Vault name cannot be empty.");
    }

    const vaultPath = getString(settings.vault_path);
    if (!vaultPath) {
        addIssue(fields, "vault_path", "error", "Choose your Obsidian vault folder.");
    } else if (!isAbsolutePath(vaultPath)) {
        addIssue(fields, "vault_path", "error", "Vault path must be a full folder path.");
    }

    validateRelativeFolder(fields, settings, "daily_note_folder", "Daily note path", {
        emptyWarning: "Daily-note actions will fail until this path is configured.",
    });
    validateRelativeFolder(fields, settings, "notes_folder", "Notes folder", {
        emptyWarning: "New notes will be created in the vault root.",
    });
    validateRelativeFolder(fields, settings, "screenshot_path", "Image folder", {
        emptyWarning: "Images will be saved in the vault root.",
    });

    const pinnedNotes = Array.isArray(settings.pinned_notes)
        ? settings.pinned_notes
        : [];
    if (
        pinnedNotes.some((note) => {
            const path = typeof note === "string" ? note : note?.path;
            const cleanPath = getString(path);
            return (
                !cleanPath ||
                isAbsolutePath(cleanPath) ||
                hasParentSegment(cleanPath)
            );
        })
    ) {
        addIssue(
            fields,
            "pinned_notes",
            "error",
            "Pinned notes must be vault-relative Markdown files.",
        );
    }

    validateFilenameTemplate(fields, settings, "daily_note_format", "Daily note format", {
        dateTokensOnly: true,
    });
    validateFilenameTemplate(
        fields,
        settings,
        "note_filename_template",
        "Note filename template",
    );
    validateFilenameTemplate(fields, settings, "image_filename", "Image filename template", {
        warnWithoutSeconds: true,
    });

    const imageWidth = getString(settings.default_image_width);
    if (imageWidth && (!/^\d+$/.test(imageWidth) || Number(imageWidth) === 0)) {
        addIssue(
            fields,
            "default_image_width",
            "error",
            "Default image width must be empty or a positive number.",
        );
    }

    for (const [field, label] of [
        ["overlay_color", "Overlay color"],
        ["accent_color", "Accent color"],
        ["internal_link_color", "Internal link color"],
        ["external_link_color", "External link color"],
        ["text_color", "Text color"],
    ]) {
        validateColor(fields, settings, field, label);
    }

    const shortcutFields = [
        { field: "global_shortcut", label: "Open Capture Window" },
        { field: "global_close_shortcut", label: "Close Capture Window" },
        { field: "reader_shortcut", label: "Open Reader Window" },
        { field: "reader_close_shortcut", label: "Close Reader Window" },
        {
            field: "reader_open_in_obsidian_shortcut",
            label: "Open Note in Obsidian",
        },
        { field: "reader_navigate_back_shortcut", label: "Navigate Back" },
        {
            field: "reader_command_palette_shortcut",
            label: "Open Command Palette",
        },
        { field: "capture_text_shortcut", label: "Copy Text to Collector" },
        { field: "save_to_daily_shortcut", label: "Save to Daily Note" },
        { field: "save_as_note_shortcut", label: "Create New Note" },
        { field: "append_to_note_shortcut", label: "Append to Note" },
    ];

    for (const shortcut of shortcutFields) {
        validateShortcut(fields, settings, shortcut.field, shortcut.label);
    }

    if (!settings.global_shortcut_closes_window) {
        addDuplicateShortcutErrors(fields, settings, [
            [
                "global_shortcut",
                "global_close_shortcut",
                "Capture open and close shortcuts must be different.",
            ],
        ]);
    }

    if (!settings.reader_shortcut_closes_window) {
        addDuplicateShortcutErrors(fields, settings, [
            [
                "reader_shortcut",
                "reader_close_shortcut",
                "Reader open and close shortcuts must be different.",
            ],
        ]);
    }

    addShortcutGroupCollisions(
        fields,
        settings,
        [
            { field: "reader_open_in_obsidian_shortcut" },
            { field: "reader_navigate_back_shortcut" },
            { field: "reader_command_palette_shortcut" },
        ],
        "Reader action shortcuts must be unique.",
    );
    addShortcutGroupCollisions(
        fields,
        settings,
        [
            { field: "save_to_daily_shortcut" },
            { field: "save_as_note_shortcut" },
            { field: "append_to_note_shortcut" },
        ],
        "Save action shortcuts must be unique.",
    );
    addReaderPrecedenceWarnings(fields, settings, [
        {
            field: "reader_open_in_obsidian_shortcut",
            label: "Open Note in Obsidian",
        },
        { field: "reader_navigate_back_shortcut", label: "Navigate Back" },
        {
            field: "reader_command_palette_shortcut",
            label: "Open Command Palette",
        },
    ]);

    for (const [field, label] of [
        ["edge_side", "Capture edge"],
        ["reader_edge_side", "Reader edge"],
    ]) {
        const value = getString(settings[field]);
        if (value !== "left" && value !== "right") {
            addIssue(fields, field, "error", `${label} must be left or right.`);
        }
    }

    const fieldValues = Object.values(fields);
    return {
        fields,
        hasErrors: fieldValues.some((issue) => issue.type === "error"),
        errorCount: fieldValues.filter((issue) => issue.type === "error").length,
        warningCount: fieldValues.filter((issue) => issue.type === "warning").length,
    };
}
