import test from "node:test";
import assert from "node:assert/strict";

import {
    evaluateCaptureRules,
    stripTriggers,
    describeRuleTarget,
} from "../src/lib/capture/rules.js";
import {
    buildInitialAppSettings,
    applySettingsUpdate,
} from "../src/lib/app/settingsBridge.js";

const rule = (overrides = {}) => ({
    id: "r1",
    name: "Test",
    enabled: true,
    conditions: [{ type: "hashtag", value: "#heute" }],
    match: "any",
    actions: [
        {
            type: "append",
            note: "To-do-Liste.md",
            heading: "## Heute",
            position: "bottom",
            createHeadingIfMissing: false,
            stripTrigger: true,
        },
    ],
    ...overrides,
});

// ── evaluateCaptureRules ────────────────────────────────────────────────

test("first matching rule wins in list order", () => {
    const rules = [
        rule({ id: "a", conditions: [{ type: "contains", value: "xyz" }] }),
        rule({ id: "b" }),
    ];
    const result = evaluateCaptureRules("etwas #heute", rules);
    assert.equal(result.matched, true);
    assert.equal(result.rule.id, "b");
});

test("disabled rules are skipped", () => {
    const result = evaluateCaptureRules("mit #heute", [
        rule({ enabled: false }),
    ]);
    assert.equal(result.matched, false);
});

test("empty rules array matches nothing", () => {
    assert.equal(evaluateCaptureRules("#heute", []).matched, false);
});

test("non-array rules argument is defensive, matches nothing", () => {
    assert.equal(evaluateCaptureRules("#heute", undefined).matched, false);
    assert.equal(evaluateCaptureRules("#heute", null).matched, false);
    assert.equal(evaluateCaptureRules("#heute", "x").matched, false);
});

test("contains is plain substring, case-insensitive by default", () => {
    const rules = [
        rule({ conditions: [{ type: "contains", value: "Renovierung" }] }),
    ];
    assert.equal(
        evaluateCaptureRules("zur RENOVIERUNG Notiz", rules).matched,
        true,
    );
});

test("contains honors caseSensitive", () => {
    const rules = [
        rule({
            conditions: [
                { type: "contains", value: "Renovierung", caseSensitive: true },
            ],
        }),
    ];
    assert.equal(evaluateCaptureRules("zur RENOVIERUNG", rules).matched, false);
    assert.equal(evaluateCaptureRules("zur Renovierung", rules).matched, true);
});

test("contains treats regex metacharacters literally", () => {
    const rules = [rule({ conditions: [{ type: "contains", value: "a.b" }] })];
    assert.equal(evaluateCaptureRules("pa", rules).matched, false);
    assert.equal(evaluateCaptureRules("axb", rules).matched, false);
    assert.equal(evaluateCaptureRules("a.b", rules).matched, true);
});

test("hashtag is a token match, not a substring match", () => {
    const rules = [rule()];
    assert.equal(
        evaluateCaptureRules("mache ich #heute.", rules).matched,
        true,
    );
    assert.equal(evaluateCaptureRules("#heuteistgut", rules).matched, false);
    assert.equal(
        evaluateCaptureRules("##heute ist ein heading", rules).matched,
        false,
    );
    assert.equal(
        evaluateCaptureRules("siehe #heute#dort", rules).matched,
        false,
    );
});

test("hashtag works with unicode characters", () => {
    const rules = [
        rule({ conditions: [{ type: "hashtag", value: "#Übermorgen" }] }),
    ];
    assert.equal(
        evaluateCaptureRules("erst #Übermorgen!", rules).matched,
        true,
    );
    assert.equal(
        evaluateCaptureRules("nur #Übermorgenunddan", rules).matched,
        false,
    );
});

test("match all requires every condition", () => {
    const rules = [
        rule({
            match: "all",
            conditions: [
                { type: "hashtag", value: "#heute" },
                { type: "contains", value: "Einkaufen" },
            ],
        }),
    ];
    assert.equal(
        evaluateCaptureRules("Einkaufen mit #heute", rules).matched,
        true,
    );
    assert.equal(evaluateCaptureRules("nur #heute", rules).matched, false);
});

test("match any satisfies with one condition", () => {
    const rules = [
        rule({
            match: "any",
            conditions: [
                { type: "hashtag", value: "#heute" },
                { type: "contains", value: "Einkaufen" },
            ],
        }),
    ];
    assert.equal(evaluateCaptureRules("Einkaufen", rules).matched, true);
});

test("unknown condition type never matches", () => {
    const rules = [
        rule({ conditions: [{ type: "time_between", value: "21:00" }] }),
    ];
    assert.equal(evaluateCaptureRules("beliebig", rules).matched, false);
});

test("prototype-chain condition types never match", () => {
    // Object.prototype keys duerften nie als Handler aufloesen und matchen
    const forToString = [rule({ conditions: [{ type: "toString" }] })];
    assert.equal(evaluateCaptureRules("beliebig", forToString).matched, false);

    const forConstructor = [rule({ conditions: [{ type: "constructor" }] })];
    assert.equal(
        evaluateCaptureRules("beliebig", forConstructor).matched,
        false,
    );
});

test("rule with empty conditions never matches", () => {
    assert.equal(
        evaluateCaptureRules("beliebig", [rule({ conditions: [] })]).matched,
        false,
    );
});

// ── stripTriggers ───────────────────────────────────────────────────────

test("strips hashtag occurrences and collapses whitespace", () => {
    const result = stripTriggers("Kauf Milch #heute", [
        { type: "hashtag", value: "#heute" },
    ]);
    assert.equal(result, "Kauf Milch");
});

test("strips multiple occurrences", () => {
    const result = stripTriggers("#heute Kaufen #heute", [
        { type: "hashtag", value: "#heute" },
    ]);
    assert.equal(result, "Kaufen");
});

test("strips only token-bounded hashtag occurrences", () => {
    const result = stripTriggers("#heuteistgut aber #heute ok", [
        { type: "hashtag", value: "#heute" },
    ]);
    assert.equal(result, "#heuteistgut aber ok");
});

test("strips exact contains string with case-insensitive default", () => {
    const result = stripTriggers("zur RENOVIERUNG bitte", [
        { type: "contains", value: "renovierung" },
    ]);
    assert.equal(result, "zur bitte");
});

test("contains strip honors caseSensitive", () => {
    const result = stripTriggers("zur Renovierung bitte", [
        { type: "contains", value: "Renovierung", caseSensitive: true },
    ]);
    assert.equal(result, "zur bitte");
});

test("drops lines that became empty after strip", () => {
    const result = stripTriggers("Erste Zeile\n#heute\nZweite Zeile", [
        { type: "hashtag", value: "#heute" },
    ]);
    assert.equal(result, "Erste Zeile\nZweite Zeile");
});

test("empty remainder keeps original text", () => {
    const result = stripTriggers("#heute", [
        { type: "hashtag", value: "#heute" },
    ]);
    assert.equal(result, "#heute");
});

test("non-strippable conditions leave text cleaned but intact", () => {
    const result = stripTriggers("  text  ", [
        { type: "time_between", value: "21:00" },
    ]);
    assert.equal(result, "text");
});

// ── describeRuleTarget ──────────────────────────────────────────────────

test("describeRuleTarget with heading", () => {
    assert.equal(describeRuleTarget(rule()), "To-do-Liste.md / ## Heute");
});

test("describeRuleTarget without heading", () => {
    const r = rule({ actions: [{ type: "append", note: "Random.md" }] });
    assert.equal(describeRuleTarget(r), "Random.md");
});

test("describeRuleTarget skips unknown actions", () => {
    const r = rule({
        actions: [
            { type: "frontmatter", key: "tags" },
            { type: "append", note: "N.md" },
        ],
    });
    assert.equal(describeRuleTarget(r), "N.md");
});

test("describeRuleTarget of action-less rule is empty", () => {
    assert.equal(describeRuleTarget(rule({ actions: [] })), "");
});

// ── settingsBridge forwarding (P1 fix) ─────────────────────────────────

test("buildInitialAppSettings forwards capture_rules", () => {
    const rules = [rule()];
    const out = buildInitialAppSettings({ capture_rules: rules });
    assert.deepEqual(out.capture_rules, rules);
});

test("buildInitialAppSettings defaults capture_rules to empty array", () => {
    assert.deepEqual(buildInitialAppSettings({}).capture_rules, []);
    assert.deepEqual(
        buildInitialAppSettings({ capture_rules: "broken" }).capture_rules,
        [],
    );
});

test("applySettingsUpdate forwards capture_rules on incoming", () => {
    const rules = [rule()];
    const out = applySettingsUpdate(
        { capture_rules: [] },
        { capture_rules: rules },
    );
    assert.deepEqual(out.capture_rules, rules);
});

test("applySettingsUpdate keeps current rules when incoming lacks the field", () => {
    const rules = [rule()];
    const out = applySettingsUpdate({ capture_rules: rules }, {});
    assert.deepEqual(out.capture_rules, rules);
});

test("applySettingsUpdate defaults to empty array when neither side has rules", () => {
    const out = applySettingsUpdate({}, {});
    assert.deepEqual(out.capture_rules, []);
});
