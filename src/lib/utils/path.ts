const INVALID_SEGMENT_CHARS = /[<>:"/\\|?*\u0000-\u001f]/g;
const RESERVED_SEGMENT_NAMES = new Set([
  "CON",
  "PRN",
  "AUX",
  "NUL",
  "COM1",
  "COM2",
  "COM3",
  "COM4",
  "COM5",
  "COM6",
  "COM7",
  "COM8",
  "COM9",
  "LPT1",
  "LPT2",
  "LPT3",
  "LPT4",
  "LPT5",
  "LPT6",
  "LPT7",
  "LPT8",
  "LPT9",
]);

function uniqueChars(value: string): string[] {
  const matches = value.match(INVALID_SEGMENT_CHARS) ?? [];
  return [...new Set(matches)];
}

export function getSafeSegmentError(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return "";

  const issues: string[] = [];
  const invalidChars = uniqueChars(trimmed);

  if (invalidChars.length > 0) {
    issues.push(`caractères interdits ${invalidChars.join(" ")}`);
  }

  if (trimmed.includes("..")) {
    issues.push('suite interdite ".."');
  }

  if (/[. ]$/.test(trimmed)) {
    issues.push("fin interdite par un point ou un espace");
  }

  if (RESERVED_SEGMENT_NAMES.has(trimmed.toUpperCase())) {
    issues.push("nom réservé par le système");
  }

  if (issues.length === 0) return "";
  return `Nom incompatible avec un dossier/fichier : ${issues.join(" ; ")}.`;
}

export function isSafeSegment(value: string): boolean {
  return getSafeSegmentError(value) === "";
}
