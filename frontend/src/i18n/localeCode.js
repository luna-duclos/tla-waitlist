import i18n from "./index";
import bundledLanguageNames from "../locales/language-names.json";

const DEFAULT_LANGUAGES = ["en"];

export function localeDisplayName(code, labels = bundledLanguageNames) {
  return labels[code] ?? bundledLanguageNames[code] ?? code.toUpperCase();
}

export function translationLocale(lng) {
  if (!lng) {
    return "en";
  }
  const short = lng.slice(0, 2).toLowerCase();
  return /^[a-z]{2}$/.test(short) ? short : "en";
}

export function localeCode(lng, supported = null) {
  const langs = Array.isArray(supported)
    ? supported
    : Array.isArray(i18n.options?.supportedLngs) && i18n.options.supportedLngs !== false
    ? i18n.options.supportedLngs
    : DEFAULT_LANGUAGES;
  if (!lng) {
    return langs[0] ?? "en";
  }
  const short = lng.slice(0, 2).toLowerCase();
  if (langs.includes(short)) {
    return short;
  }
  if (langs.includes(lng)) {
    return lng;
  }
  return langs[0] ?? "en";
}

export function languagesWithEnglishFirst(languages) {
  if (!Array.isArray(languages) || !languages.includes("en")) {
    return languages;
  }
  return ["en", ...languages.filter((code) => code !== "en")];
}

export function applySupportedLanguages(languages) {
  const langs = languages.length > 0 ? languages : DEFAULT_LANGUAGES;
  i18n.options.supportedLngs = langs;
  const current = localeCode(i18n.resolvedLanguage ?? i18n.language, langs);
  if (!langs.includes(current)) {
    i18n.changeLanguage(langs[0] ?? "en");
  }
}

export { bundledLanguageNames };
