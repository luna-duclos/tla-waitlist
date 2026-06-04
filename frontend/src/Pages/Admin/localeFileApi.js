import {
  buildGenericLocaleTemplate,
  buildHomeLocaleTemplate,
} from "./localeHomeFields";
import { parseLocaleAdminFilename, isGuideLocalePage } from "./groupDataFiles";
import { buildNavLocaleTemplate } from "./localeNavFields";
import { buildGuideLocaleTemplate } from "./guideLocalePages";

export function localeFilename(page, locale) {
  return `${page}.${locale}.json`;
}

export function existingLocalesForPage(files, page) {
  return files
    .map((file) => parseLocaleAdminFilename(file.name))
    .filter((parsed) => parsed && parsed.page === page)
    .map((parsed) => parsed.locale);
}

export async function fetchLocaleFileContent(filename) {
  const response = await fetch(`/api/admin/data-files/${filename}`, {
    credentials: "include",
  });
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(errorText || `HTTP ${response.status}`);
  }
  return response.text();
}

export async function createLocaleFile({ page, locale, copyFromLocale = null, asGuide = false }) {
  const filename = localeFilename(page, locale);
  let content;

  if (copyFromLocale) {
    content = await fetchLocaleFileContent(localeFilename(page, copyFromLocale));
  } else if (page === "home") {
    content = buildHomeLocaleTemplate();
  } else if (page === "nav") {
    content = buildNavLocaleTemplate();
  } else if (asGuide || isGuideLocalePage(page)) {
    content = buildGuideLocaleTemplate(page);
  } else {
    content = buildGenericLocaleTemplate();
  }

  const url = asGuide
    ? `/api/admin/data-files/${filename}?kind=guide`
    : `/api/admin/data-files/${filename}`;

  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "text/plain" },
    credentials: "include",
    body: content,
  });
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(errorText || `HTTP ${response.status}`);
  }
  return filename;
}

export async function deleteLocaleFile(filename) {
  const response = await fetch(`/api/admin/data-files/${filename}`, {
    method: "DELETE",
    credentials: "include",
  });
  if (!response.ok) {
    const errorText = await response.text();
    throw new Error(errorText || `HTTP ${response.status}`);
  }
}
