import { findWikiTrigger, computeWikilinkInsertion } from "../capture/wiki-utils.js";
import { getAutocompleteResults } from "../reader/autocomplete.js";
import { clampOverlayLeft } from "../overlayPosition.js";

function getTextareaPosition(textarea, position) {
    const rect = textarea.getBoundingClientRect();
    const style = getComputedStyle(textarea);
    const mirror = document.createElement("div");
    const marker = document.createElement("span");
    const copyStyles = [
        "boxSizing",
        "borderTopWidth",
        "borderRightWidth",
        "borderBottomWidth",
        "borderLeftWidth",
        "fontFamily",
        "fontSize",
        "fontStyle",
        "fontVariant",
        "fontWeight",
        "letterSpacing",
        "lineHeight",
        "paddingTop",
        "paddingRight",
        "paddingBottom",
        "paddingLeft",
        "tabSize",
        "textIndent",
        "textTransform",
        "wordBreak",
        "wordSpacing",
    ];

    copyStyles.forEach((property) => {
        mirror.style[property] = style[property];
    });

    mirror.style.position = "absolute";
    mirror.style.visibility = "hidden";
    mirror.style.pointerEvents = "none";
    mirror.style.whiteSpace = "pre-wrap";
    mirror.style.wordWrap = "break-word";
    mirror.style.overflowWrap = "break-word";
    mirror.style.overflow = "hidden";
    mirror.style.width = `${textarea.offsetWidth}px`;
    mirror.style.top = "0";
    mirror.style.left = "-9999px";

    const textBefore = textarea.value.substring(0, position);
    mirror.textContent = textBefore.endsWith("\n")
        ? `${textBefore} `
        : textBefore;
    marker.textContent = ".";
    mirror.appendChild(marker);
    document.body.appendChild(mirror);

    try {
        const lineHeight = parseFloat(style.lineHeight) || 24;

        return {
            left: rect.left + marker.offsetLeft - textarea.scrollLeft,
            top: rect.top + marker.offsetTop - textarea.scrollTop,
            bottom:
                rect.top + marker.offsetTop - textarea.scrollTop + lineHeight,
        };
    } finally {
        mirror.remove();
    }
}

function getWikiDropdownPosition(textarea, anchorPosition, cursorPosition) {
    if (!textarea) return { left: 0, top: 0 };

    const anchorRect = getTextareaPosition(textarea, anchorPosition);
    const caretRect = getTextareaPosition(textarea, cursorPosition);

    return {
        left: clampOverlayLeft(anchorRect.left),
        top: caretRect.bottom + 6,
    };
}

export function computeClosedState() {
    return {
        open: false,
        query: "",
        matches: [],
        index: 0,
        position: { left: 0, top: 0 },
        anchorPos: 0,
    };
}

function computeOpenState({ textarea, trigger, matches }) {
    return {
        open: matches.length > 0,
        query: trigger.query,
        matches,
        index: 0,
        position: getWikiDropdownPosition(
            textarea,
            trigger.triggerIndex,
            textarea?.selectionStart ?? trigger.triggerIndex,
        ),
        anchorPos: trigger.triggerIndex,
    };
}

export function computeInputState({ textarea, notes, limit }) {
    if (!textarea) return null;

    const val = textarea.value;
    const cursor = textarea.selectionStart;
    const trigger = findWikiTrigger(val, cursor);

    if (!trigger) {
        return computeClosedState();
    }

    const matches = getAutocompleteResults(trigger.query, notes, limit);
    return computeOpenState({ textarea, trigger, matches });
}

export function computeInsertion({ textarea, query, note }) {
    if (!textarea) return null;

    const val = textarea.value;
    const cursor = textarea.selectionStart;
    const queryLen = query.length;

    if (cursor - queryLen - 2 < 0) return null;

    const { before, after, insertion, newPosition } = computeWikilinkInsertion(
        val,
        cursor,
        queryLen,
        note.name,
    );

    textarea.value = before + insertion + after;
    textarea.selectionStart = newPosition;
    textarea.selectionEnd = newPosition;
    textarea.focus();

    return textarea.value;
}

export function handleWikiKeydown({
    event,
    isOpen,
    currentIndex,
    matches,
    onSelect,
    onClose,
    setIndex,
}) {
    if (!isOpen) return false;

    if (event.key === "ArrowDown") {
        event.preventDefault();
        setIndex(Math.min(currentIndex + 1, matches.length - 1));
        return true;
    }
    if (event.key === "ArrowUp") {
        event.preventDefault();
        setIndex(Math.max(currentIndex - 1, 0));
        return true;
    }
    if (event.key === "Enter" || event.key === "Tab") {
        event.preventDefault();
        if (matches[currentIndex]) {
            onSelect(matches[currentIndex]);
        }
        return true;
    }
    if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return true;
    }
    return false;
}
