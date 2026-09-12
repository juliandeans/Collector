const DEFAULT_CONDITION = {
    type: "hashtag",
    value: "#tag",
    caseSensitive: false,
};

const DEFAULT_ACTION = {
    type: "append",
    note: "",
    heading: "",
    position: "bottom",
    createHeadingIfMissing: false,
    stripTrigger: true,
    showTimestamp: true,
};

export function createCaptureRule() {
    return {
        // crypto.randomUUID exists only from Safari 15.4 (macOS 12.3); the
        // app supports macOS 11 — fall back to a timestamp-based id there.
        id:
            crypto?.randomUUID?.() ??
            `rule-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`,
        name: "New Rule",
        enabled: true,
        conditions: [{ ...DEFAULT_CONDITION }],
        match: "any",
        actions: [{ ...DEFAULT_ACTION }],
    };
}

export function normalizeCaptureRule(rule = {}) {
    const fallback = createCaptureRule();
    const conditions =
        Array.isArray(rule.conditions) && rule.conditions.length > 0
            ? rule.conditions
            : fallback.conditions;
    const actions =
        Array.isArray(rule.actions) && rule.actions.length > 0
            ? rule.actions
            : fallback.actions;

    return {
        ...rule,
        id:
            typeof rule.id === "string" && rule.id.trim()
                ? rule.id
                : fallback.id,
        name: typeof rule.name === "string" ? rule.name : fallback.name,
        enabled: rule.enabled !== false,
        conditions,
        match: rule.match === "all" ? "all" : "any",
        actions,
    };
}
