export function handleShortcutKeyDown(event, field, settings, onChange = () => {}) {
    event.preventDefault();

    if (
        ["Control", "Shift", "Alt", "Meta", "Command"].includes(event.key)
    ) {
        return;
    }

    const modifiers = [];
    if (event.metaKey) modifiers.push("Cmd");
    if (event.ctrlKey) modifiers.push("Ctrl");
    if (event.shiftKey) modifiers.push("Shift");
    if (event.altKey) modifiers.push("Option");

    let key = event.code;

    const keyMap = {
        Space: "Space",
        Escape: "Escape",
        Enter: "Enter",
        Backspace: "Backspace",
        Delete: "Delete",
        ArrowUp: "Up",
        ArrowDown: "Down",
        ArrowLeft: "Left",
        ArrowRight: "Right",
        Tab: "Tab",
        Home: "Home",
        End: "End",
        PageUp: "PageUp",
        PageDown: "PageDown",
    };

    if (keyMap[key]) {
        key = keyMap[key];
    } else if (key.startsWith("Key")) {
        key = key.substring(3).toUpperCase();
    } else if (key.startsWith("Digit")) {
        key = key.substring(5);
    } else if (key.startsWith("F") && key.length <= 3) {
        key = key;
    } else {
        return;
    }

    if (modifiers.length === 0) {
        return;
    }

    const shortcut = [...modifiers, key].join("+");
    settings[field] = shortcut;
    onChange();
}
