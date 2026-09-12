<script>
    import Checkbox from "./Checkbox.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import { describeRuleTarget } from "../capture/rules.js";
    import { filterPaletteNotes } from "../reader/paletteLogic.js";
    import { createCaptureRule, normalizeCaptureRule } from "./rules-utils.js";

    let {
        settings = $bindable(),
        validation = {},
        vaultNotes = [],
        onChange = () => {},
    } = $props();

    let editingRuleId = $state(null);
    let notePickerOpen = $state(false);
    let noteQuery = $state("");

    let rules = $derived(
        Array.isArray(settings.capture_rules) ? settings.capture_rules : [],
    );
    let editingRule = $derived(
        rules.find((rule) => rule?.id === editingRuleId) ?? null,
    );
    let editingIndex = $derived(
        editingRule ? rules.indexOf(editingRule) : -1,
    );
    let editingCondition = $derived(editingRule?.conditions?.[0] ?? {});
    let editingAction = $derived(editingRule?.actions?.[0] ?? {});
    let filteredNotes = $derived(
        filterPaletteNotes(
            vaultNotes,
            noteQuery,
            settings.autocomplete_results ?? 20,
        ),
    );

    function setRules(nextRules) {
        settings = { ...settings, capture_rules: nextRules };
        onChange();
    }

    function updateRuleAt(index, updater) {
        const rule = rules[index];
        if (!rule) return;

        setRules(
            rules.map((candidate, candidateIndex) =>
                candidateIndex === index ? updater(candidate) : candidate,
            ),
        );
    }

    function updateEditingRule(updater) {
        if (editingIndex < 0) return;
        updateRuleAt(editingIndex, (rule) => updater(normalizeCaptureRule(rule)));
    }

    function addRule() {
        const rule = createCaptureRule();
        setRules([...rules, rule]);
        editingRuleId = rule.id;
        notePickerOpen = false;
    }

    function editRule(index) {
        const rule = normalizeCaptureRule(rules[index]);
        setRules(
            rules.map((candidate, candidateIndex) =>
                candidateIndex === index ? rule : candidate,
            ),
        );
        editingRuleId = rule.id;
        notePickerOpen = false;
    }

    function closeEditor() {
        editingRuleId = null;
        notePickerOpen = false;
    }

    function deleteRule(index) {
        if (rules[index]?.id === editingRuleId) closeEditor();
        setRules(rules.filter((_, candidateIndex) => candidateIndex !== index));
    }

    function moveRule(index, offset) {
        const targetIndex = index + offset;
        if (targetIndex < 0 || targetIndex >= rules.length) return;

        const nextRules = [...rules];
        [nextRules[index], nextRules[targetIndex]] = [
            nextRules[targetIndex],
            nextRules[index],
        ];
        setRules(nextRules);
    }

    function setRuleEnabled(index, enabled) {
        updateRuleAt(index, (rule) => ({ ...rule, enabled }));
    }

    function updateName(value) {
        updateEditingRule((rule) => ({ ...rule, name: value }));
    }

    function updateCondition(field, value) {
        updateEditingRule((rule) => {
            const conditions = [...rule.conditions];
            conditions[0] = { ...conditions[0], [field]: value };
            return { ...rule, conditions };
        });
    }

    function updateAction(field, value) {
        updateEditingRule((rule) => {
            const actions = [...rule.actions];
            actions[0] = { ...actions[0], [field]: value };
            return { ...rule, actions };
        });

        if (field === "note") {
            noteQuery = value;
            notePickerOpen = true;
        }
    }

    function openNotePicker() {
        noteQuery = editingAction.note ?? "";
        notePickerOpen = true;
    }

    function chooseNote(note) {
        updateAction("note", note.relative_path);
        notePickerOpen = false;
    }

    function handleNoteKeydown(event) {
        if (event.key === "Escape") {
            event.preventDefault();
            notePickerOpen = false;
        } else if (event.key === "Enter" && filteredNotes[0]) {
            event.preventDefault();
            chooseNote(filteredNotes[0]);
        }
    }

    function conditionSummary(rule) {
        const condition = rule?.conditions?.[0];
        const type = condition?.type || "contains";
        const value = typeof condition?.value === "string" ? condition.value : "";
        return `${type} "${value || "(empty)"}"`;
    }

    function ruleKey(rule, index) {
        return rule?.id || `capture-rule-${index}`;
    }
</script>

<div class="capture-rules">
    {#if rules.length === 0}
        <div class="capture-rules-empty">
            <p>
                Rules route captures with certain content to a target note.
                Without rules, captures use the standard destination.
            </p>
            <button class="capture-rule-action-button" type="button" onclick={addRule}>+ Add Rule</button>
        </div>
    {:else}
        <div class="capture-rules-list">
            {#each rules as rule, index (ruleKey(rule, index))}
                <div class="capture-rule-item">
                    <div class="capture-rule-summary">
                        <div class="capture-rule-copy">
                            <div class="capture-rule-name">
                                {rule.name || "Unnamed rule"}
                                {#if rule.enabled === false}
                                    <span class="capture-rule-disabled">Disabled</span>
                                {/if}
                            </div>
                            <div class="capture-rule-detail">{conditionSummary(rule)}</div>
                            <div class="capture-rule-detail capture-rule-target">
                                → {describeRuleTarget(rule) || "No target note"}
                            </div>
                        </div>
                        <div class="capture-rule-controls">
                            <Checkbox
                                checked={rule.enabled !== false}
                                onChange={(event) =>
                                    setRuleEnabled(index, event.currentTarget.checked)}
                            >
                                Enabled
                            </Checkbox>
                            <div class="capture-rule-reorder">
                                <button
                                    class="capture-rule-icon-button"
                                    type="button"
                                    title="Move rule up"
                                    aria-label="Move rule up"
                                    disabled={index === 0}
                                    onclick={() => moveRule(index, -1)}
                                >
                                    ↑
                                </button>
                                <button
                                    class="capture-rule-icon-button"
                                    type="button"
                                    title="Move rule down"
                                    aria-label="Move rule down"
                                    disabled={index === rules.length - 1}
                                    onclick={() => moveRule(index, 1)}
                                >
                                    ↓
                                </button>
                            </div>
                            <button
                                class="capture-rule-action-button"
                                type="button"
                                onclick={() => editRule(index)}
                            >
                                Edit
                            </button>
                            <button
                                class="capture-rule-action-button capture-rule-delete"
                                type="button"
                                onclick={() => deleteRule(index)}
                            >
                                Delete
                            </button>
                        </div>
                    </div>

                    {#if editingRuleId === rule.id}
                        <div class="capture-rule-editor">
                            <div class="capture-rule-editor-grid">
                                <div class="field">
                                    <label for={`capture-rule-${rule.id}-name`}>Name</label>
                                    <input
                                        type="text"
                                        id={`capture-rule-${rule.id}-name`}
                                        value={editingRule.name ?? ""}
                                        oninput={(event) => updateName(event.currentTarget.value)}
                                    />
                                    <FieldIssue
                                        issue={validation[`capture_rules.${editingIndex}.name`]}
                                    />
                                </div>

                                <div class="field">
                                    <label for={`capture-rule-${rule.id}-condition-type`}
                                        >Condition</label
                                    >
                                    <div class="capture-rule-condition-fields">
                                        <select
                                            id={`capture-rule-${rule.id}-condition-type`}
                                            value={editingCondition.type ?? "hashtag"}
                                            onchange={(event) =>
                                                updateCondition(
                                                    "type",
                                                    event.currentTarget.value,
                                                )}
                                        >
                                            <option value="contains">Contains</option>
                                            <option value="hashtag">Hashtag</option>
                                        </select>
                                        <input
                                            type="text"
                                            value={editingCondition.value ?? ""}
                                            placeholder="#tag"
                                            oninput={(event) =>
                                                updateCondition(
                                                    "value",
                                                    event.currentTarget.value,
                                                )}
                                        />
                                    </div>
                                    <FieldIssue
                                        issue={
                                            validation[
                                                `capture_rules.${editingIndex}.condition.value`
                                            ]
                                        }
                                    />
                                </div>
                            </div>

                            <div class="capture-rule-editor-options">
                                <Checkbox
                                    checked={editingCondition.caseSensitive === true}
                                    onChange={(event) =>
                                        updateCondition(
                                            "caseSensitive",
                                            event.currentTarget.checked,
                                        )}
                                >
                                    Case-sensitive
                                </Checkbox>
                            </div>

                            <div class="field capture-rule-note-field">
                                <label for={`capture-rule-${rule.id}-note`}>Target note</label>
                                <input
                                    type="text"
                                    id={`capture-rule-${rule.id}-note`}
                                    value={editingAction.note ?? ""}
                                    placeholder="Target note.md"
                                    autocomplete="off"
                                    onfocus={openNotePicker}
                                    onkeydown={handleNoteKeydown}
                                    oninput={(event) =>
                                        updateAction("note", event.currentTarget.value)}
                                />
                                <FieldIssue
                                    issue={validation[`capture_rules.${editingIndex}.action.note`]}
                                />
                                {#if notePickerOpen && filteredNotes.length > 0}
                                    <div class="capture-rule-note-picker" role="listbox">
                                        {#each filteredNotes as note (note.relative_path)}
                                            <button
                                                class="capture-rule-note-option"
                                                type="button"
                                                onmousedown={(event) => {
                                                    event.preventDefault();
                                                    chooseNote(note);
                                                }}
                                            >
                                                <span>{note.name}</span>
                                                {#if settings.show_note_paths !== false}
                                                    <small>{note.relative_path}</small>
                                                {/if}
                                            </button>
                                        {/each}
                                    </div>
                                {/if}
                            </div>

                            <div class="capture-rule-editor-grid">
                                <div class="field">
                                    <label for={`capture-rule-${rule.id}-heading`}>Heading</label>
                                    <input
                                        type="text"
                                        id={`capture-rule-${rule.id}-heading`}
                                        value={editingAction.heading ?? ""}
                                        placeholder="## Heute"
                                        oninput={(event) =>
                                            updateAction(
                                                "heading",
                                                event.currentTarget.value,
                                            )}
                                    />
                                    <small>Leave empty to append to the end of the note.</small>
                                </div>
                                <div class="field">
                                    <label for={`capture-rule-${rule.id}-position`}>Position</label>
                                    <select
                                        id={`capture-rule-${rule.id}-position`}
                                        value={editingAction.position ?? "bottom"}
                                        onchange={(event) =>
                                            updateAction(
                                                "position",
                                                event.currentTarget.value,
                                            )}
                                    >
                                        <option value="bottom">Bottom of section</option>
                                        <option value="top">Top of section</option>
                                    </select>
                                </div>
                            </div>

                            <div class="capture-rule-editor-options">
                                <Checkbox
                                    checked={editingAction.createHeadingIfMissing === true}
                                    onChange={(event) =>
                                        updateAction(
                                            "createHeadingIfMissing",
                                            event.currentTarget.checked,
                                        )}
                                >
                                    Create heading if missing
                                </Checkbox>
                                <Checkbox
                                    checked={editingAction.showTimestamp !== false}
                                    onChange={(event) =>
                                        updateAction(
                                            "showTimestamp",
                                            event.currentTarget.checked,
                                        )}
                                >
                                    Show timestamp
                                </Checkbox>
                                <Checkbox
                                    checked={editingAction.stripTrigger !== false}
                                    onChange={(event) =>
                                        updateAction(
                                            "stripTrigger",
                                            event.currentTarget.checked,
                                        )}
                                >
                                    Remove trigger from saved text
                                </Checkbox>
                            </div>

                            <div class="capture-rule-example">
                                A capture with <code>{editingCondition.value || "#tag"}</code>
                                would be saved to
                                <strong>
                                    {describeRuleTarget(editingRule) || "the selected note"}
                                </strong>.
                            </div>
                            <button
                                class="capture-rule-action-button capture-rule-done"
                                type="button"
                                onclick={closeEditor}
                            >
                                Done
                            </button>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
        <button class="capture-rule-action-button" type="button" onclick={addRule}>+ Add Rule</button>
    {/if}
</div>
