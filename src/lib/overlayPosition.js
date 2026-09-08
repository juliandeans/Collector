const DEFAULT_MARGIN = 12;
const DEFAULT_WIDTH = 320;

export function clampOverlayLeft(left, { width = DEFAULT_WIDTH, margin = DEFAULT_MARGIN } = {}) {
    const pickerWidth = Math.min(width, window.innerWidth - margin * 2);
    const maxLeft = window.innerWidth - pickerWidth - margin;

    return Math.max(margin, Math.min(left, maxLeft));
}
