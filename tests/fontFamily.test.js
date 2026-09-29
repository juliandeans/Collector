import test from "node:test";
import assert from "node:assert/strict";
import {
  SYSTEM_FONT_VALUE,
  FALLBACK_FONT_FAMILIES,
  fontFamilyCss,
  buildFontOptions,
} from "../src/lib/fontFamily.js";

const SYSTEM_CSS = '-apple-system, BlinkMacSystemFont, "SF Pro Display", sans-serif';

test("fontFamilyCss maps system value, empty and non-string to the system stack", () => {
  assert.equal(fontFamilyCss(SYSTEM_FONT_VALUE), SYSTEM_CSS);
  assert.equal(fontFamilyCss(""), SYSTEM_CSS);
  assert.equal(fontFamilyCss("   "), SYSTEM_CSS);
  assert.equal(fontFamilyCss(undefined), SYSTEM_CSS);
  assert.equal(fontFamilyCss(null), SYSTEM_CSS);
});

test("fontFamilyCss quotes a single family and appends the system fallback", () => {
  assert.equal(fontFamilyCss("Inter"), '"Inter", -apple-system, BlinkMacSystemFont, sans-serif');
  assert.equal(
    fontFamilyCss("JetBrains Mono"),
    '"JetBrains Mono", -apple-system, BlinkMacSystemFont, sans-serif',
  );
  assert.equal(fontFamilyCss("SF Pro"), '"SF Pro", -apple-system, BlinkMacSystemFont, sans-serif');
});

test("fontFamilyCss leaves a comma list unquoted", () => {
  assert.equal(fontFamilyCss("Inter, sans-serif"), "Inter, sans-serif");
});

test("fontFamilyCss strips characters that could break out of the CSS value", () => {
  assert.equal(
    fontFamilyCss("Menlo; color: red"),
    '"Menlo color: red", -apple-system, BlinkMacSystemFont, sans-serif',
  );
  assert.equal(
    fontFamilyCss('"}<script>\\\'\n'),
    '"script", -apple-system, BlinkMacSystemFont, sans-serif',
  );
  assert.equal(fontFamilyCss('"}<>'), SYSTEM_CSS);
});

test("buildFontOptions puts System first, then installed families", () => {
  const options = buildFontOptions(["Arial", "Inter"], SYSTEM_FONT_VALUE);
  assert.deepEqual(options, [
    { value: SYSTEM_FONT_VALUE, label: "System (Default)" },
    { value: "Arial", label: "Arial" },
    { value: "Inter", label: "Inter" },
  ]);
});

test("buildFontOptions keeps a saved but missing family visible", () => {
  const options = buildFontOptions(["Arial"], "Gone Font");
  assert.deepEqual(options, [
    { value: SYSTEM_FONT_VALUE, label: "System (Default)" },
    { value: "Gone Font", label: "Gone Font (not installed)" },
    { value: "Arial", label: "Arial" },
  ]);
});

test("buildFontOptions does not add an extra entry for an installed or empty value", () => {
  assert.equal(buildFontOptions(["Arial"], "Arial").length, 2);
  assert.equal(buildFontOptions(["Arial"], "").length, 2);
  assert.equal(buildFontOptions(["Arial"], undefined).length, 2);
});

test("buildFontOptions falls back to the fixed list when nothing is installed", () => {
  for (const installed of [[], null, undefined, "nope"]) {
    const options = buildFontOptions(installed, SYSTEM_FONT_VALUE);
    assert.equal(options[0].value, SYSTEM_FONT_VALUE);
    assert.deepEqual(
      options.slice(1).map((o) => o.value),
      FALLBACK_FONT_FAMILIES,
    );
  }
});

test("buildFontOptions keeps legacy values from the fixed list selectable", () => {
  const options = buildFontOptions([], "Menlo");
  assert.ok(options.some((o) => o.value === "Menlo" && o.label === "Menlo"));
  assert.ok(!options.some((o) => o.label.includes("not installed")));
  const commaList = buildFontOptions(["Arial"], "Inter, sans-serif");
  assert.ok(commaList.some((o) => o.value === "Inter, sans-serif"));
});
