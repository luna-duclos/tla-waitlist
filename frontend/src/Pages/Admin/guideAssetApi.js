const ACCEPTED_IMAGE_TYPES = ["image/png", "image/jpeg", "image/gif", "image/webp", "image/svg+xml"];

function sanitizeFilename(name) {
  const base = name.replace(/\\/g, "/").split("/").pop() ?? name;
  const cleaned = base.replace(/[^a-zA-Z0-9._-]/g, "_");
  if (!cleaned || cleaned.startsWith(".")) {
    throw new Error("Invalid file name.");
  }
  return cleaned;
}

export async function listGuideAssets(slug) {
  const response = await fetch(`/api/admin/guides/${encodeURIComponent(slug)}/assets`, {
    credentials: "include",
  });
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(errorText || `HTTP ${response.status}`);
  }
  const data = await response.json();
  return Array.isArray(data.assets) ? data.assets : [];
}

export async function uploadGuideAsset(slug, file) {
  const filename = sanitizeFilename(file.name);
  const response = await fetch(
    `/api/admin/guides/${encodeURIComponent(slug)}/assets/${encodeURIComponent(filename)}`,
    {
      method: "POST",
      credentials: "include",
      headers: {
        "Content-Type": file.type || "application/octet-stream",
      },
      body: file,
    }
  );
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(errorText || `HTTP ${response.status}`);
  }
  const data = await response.json();
  return typeof data.filename === "string" ? data.filename : filename;
}

export async function deleteGuideAsset(slug, filename) {
  const response = await fetch(
    `/api/admin/guides/${encodeURIComponent(slug)}/assets/${encodeURIComponent(filename)}`,
    {
      method: "DELETE",
      credentials: "include",
    }
  );
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(errorText || `HTTP ${response.status}`);
  }
}

export function guideAssetUrl(slug, filename) {
  return `/api/v2/guides/${encodeURIComponent(slug)}/assets/${encodeURIComponent(filename)}`;
}

export function guideAssetMarkdown(filename) {
  return `![](${filename})`;
}

export { ACCEPTED_IMAGE_TYPES };
