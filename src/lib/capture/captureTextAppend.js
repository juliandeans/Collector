export function appendCapturedText(currentContent, newText) {
    if (!currentContent.trim()) return newText;

    const endsWithBlankLine = /\n\s*\n$/.test(currentContent);
    return currentContent + (endsWithBlankLine ? "\n" : "\n\n") + newText;
}
