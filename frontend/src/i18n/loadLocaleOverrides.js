import React from "react";
import i18n from "./index";

/**
 * Merges server-side locale files (data/locales) over bundled defaults.
 * Used for pages editable via Admin → Data Files.
 */
import { translationLocale } from "./localeCode";

const SITE_LOCALE_PAGES = ["home", "nav"];

const loadedKeys = new Set();
const inFlight = new Map();

export async function loadLocaleOverrides(page, lng) {
  const locale = translationLocale(lng);
  const key = `${page}:${locale}`;
  if (loadedKeys.has(key)) {
    return true;
  }
  if (inFlight.has(key)) {
    return inFlight.get(key);
  }

  const promise = (async () => {
    try {
      const response = await fetch(`/api/v2/locales/${page}/${locale}`);
      if (!response.ok) {
        return false;
      }
      const data = await response.json();
      i18n.addResourceBundle(locale, page, data, true, true);
      loadedKeys.add(key);
      i18n.emit("loaded");
      return true;
    } catch {
      return false;
    } finally {
      inFlight.delete(key);
    }
  })();

  inFlight.set(key, promise);
  return promise;
}

export async function loadLocaleOverridesForLanguage(lng) {
  await Promise.all(SITE_LOCALE_PAGES.map((page) => loadLocaleOverrides(page, lng)));
}

export async function changeSiteLanguage(lng) {
  const locale = translationLocale(lng);
  await loadLocaleOverridesForLanguage(locale);
  await i18n.changeLanguage(locale);
}

export function useLocaleOverrides(page) {
  React.useEffect(() => {
    const apply = (lng) => loadLocaleOverrides(page, lng);
    apply(i18n.language);
    i18n.on("languageChanged", apply);
    return () => i18n.off("languageChanged", apply);
  }, [page]);
}

export function useAppLocaleOverrides() {
  React.useEffect(() => {
    const apply = (lng) => {
      SITE_LOCALE_PAGES.forEach((page) => loadLocaleOverrides(page, lng));
    };
    apply(i18n.language);
    i18n.on("languageChanged", apply);
    return () => i18n.off("languageChanged", apply);
  }, []);
}
