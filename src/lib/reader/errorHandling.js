export function normalizeError(error) {
  return typeof error === "string"
    ? error
    : error?.message || error?.toString?.() || "Unknown error";
}

export function isFileMissingError(error) {
  const text = normalizeError(error).toLowerCase();
  return (
    text.includes("no such file") ||
    text.includes("not found") ||
    text.includes("cannot find")
  );
}

export function getMissingState(error) {
  const message = normalizeError(error);
  const missing = isFileMissingError(error);
  return {
    message,
    missing,
    missingMessage: missing ? "File not found - will be created on first save" : message,
  };
}
