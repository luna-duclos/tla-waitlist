/**
 * Shared JSON helpers for all locale files (home, nav, guides, etc.).
 * Backend expects a JSON object whose values are strings.
 */

export function parseLocaleJson(content, { requiredKeys = [] } = {}) {
  const parsed = typeof content === "string" ? JSON.parse(content) : content;
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    throw new Error("File must be a JSON object.");
  }
  for (const key of requiredKeys) {
    if (typeof parsed[key] !== "string") {
      throw new Error(`"${key}" must be a text value.`);
    }
  }
  for (const [key, value] of Object.entries(parsed)) {
    if (typeof value !== "string") {
      throw new Error(`"${key}" must be a text value.`);
    }
  }
  return parsed;
}

export function fieldsToJson(fields, fieldDefinitions = []) {
  const ordered = {};
  for (const { key } of fieldDefinitions) {
    if (key in fields) {
      ordered[key] = fields[key];
    }
  }
  for (const key of Object.keys(fields)) {
    if (!(key in ordered)) {
      ordered[key] = fields[key];
    }
  }
  return JSON.stringify(ordered, null, 2);
}

export function buildFieldsTemplate(fieldDefinitions, source = {}) {
  const obj = {};
  for (const { key } of fieldDefinitions) {
    obj[key] = source[key] ?? "";
  }
  return fieldsToJson(obj, fieldDefinitions);
}
