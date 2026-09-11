// Capture/Reader windows dock flush against a physical screen edge
// (edge_side/reader_edge_side). By default only the two corners facing the
// screen's interior are rounded, so the flush edge stays a clean straight
// line instead of a rounded corner visually floating against the screen
// bezel. round_all_corners opts back into rounding all four.
//
// Shared between App.svelte (Capture) and Reader.svelte — both need the
// exact same shorthand to stay in sync with the native window's own corner
// mask (src-tauri/src/window_style.rs's masked_corners).
export function cornerRadiusCss(radius, edgeSide, roundAllCorners) {
  const px = `${radius}px`;
  if (roundAllCorners) return px;

  // CSS border-radius shorthand order: top-left top-right bottom-right bottom-left.
  if (edgeSide === "left") {
    return `0px ${px} ${px} 0px`;
  }
  return `${px} 0px 0px ${px}`;
}
