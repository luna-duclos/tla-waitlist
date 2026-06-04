/** Draft previews use localStorage so a new tab can read what the editor saved. */
const STORAGE_PREFIX = "tla-locale-draft:";

export function localeDraftStorageKey(filename) {
  return `${STORAGE_PREFIX}${filename}`;
}

/** @returns {{ data: Record<string, string>, updatedAt: number } | null} */
export function loadLocaleDraft(filename) {
  const raw = window.localStorage.getItem(localeDraftStorageKey(filename));
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    const data = parsed?.data ?? parsed?.translations ?? parsed?.guide;
    if (!data || typeof data !== "object" || Array.isArray(data)) {
      return null;
    }
    if (parsed?.guide?.body != null && typeof parsed.guide.body === "string") {
      return { data: flattenGuideDraft(parsed.guide), updatedAt: parsed.updatedAt ?? 0 };
    }
    return { data, updatedAt: parsed.updatedAt ?? 0 };
  } catch {
    return null;
  }
}

function flattenGuideDraft(guide) {
  return {
    ...(guide.extra ?? {}),
    ...(guide.meta ?? {}),
    body: guide.body,
  };
}

export function saveLocaleDraft(filename, data) {
  window.localStorage.setItem(
    localeDraftStorageKey(filename),
    JSON.stringify({ data, updatedAt: Date.now() })
  );
}

export function openLocalePreviewUrl(path) {
  window.open(path, "_blank");
}

export function openHomeLocalePreview(filename, fields) {
  saveLocaleDraft(filename, fields);
  const locale = filename.match(/^home\.([a-z]{2})\.json$/)?.[1] ?? "en";
  openLocalePreviewUrl(
    `/admin/preview/home/${locale}?file=${encodeURIComponent(filename)}`
  );
}

export function openGuideLocalePreview(filename, fields) {
  saveLocaleDraft(filename, fields);
  const match = filename.match(/^([a-z][a-z0-9_-]*)\.([a-z]{2})\.json$/);
  const slug = match?.[1] ?? "guide";
  const locale = match?.[2] ?? "en";
  openLocalePreviewUrl(
    `/admin/preview/guide/${slug}/${locale}?file=${encodeURIComponent(filename)}`
  );
}
