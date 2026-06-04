import React from "react";
import i18n from "./index";
import { applySupportedLanguages, bundledLanguageNames, localeCode, translationLocale } from "./localeCode";
import { loadLocaleOverridesForLanguage } from "./loadLocaleOverrides";

function mergeLanguageLabels(apiLabels) {
  if (!apiLabels || typeof apiLabels !== "object") {
    return { ...bundledLanguageNames };
  }
  return { ...bundledLanguageNames, ...apiLabels };
}

export function useAvailableLanguages() {
  const [languages, setLanguages] = React.useState(["en"]);
  const [labels, setLabels] = React.useState(bundledLanguageNames);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    let cancelled = false;

    fetch("/api/v2/locales/languages")
      .then((response) => {
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        return response.json();
      })
      .then(async (data) => {
        if (cancelled) return;
        const langs = Array.isArray(data.languages) && data.languages.length > 0 ? data.languages : ["en"];
        setLanguages(langs);
        setLabels(mergeLanguageLabels(data.labels));
        applySupportedLanguages(langs);
        const locale = localeCode(i18n.resolvedLanguage ?? i18n.language, langs);
        await loadLocaleOverridesForLanguage(locale);
        if (translationLocale(i18n.language) !== locale) {
          await i18n.changeLanguage(locale);
        }
        i18n.emit("loaded");
      })
      .catch(async () => {
        if (cancelled) return;
        setLanguages(["en"]);
        setLabels(bundledLanguageNames);
        applySupportedLanguages(["en"]);
        await loadLocaleOverridesForLanguage("en");
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return { languages, labels, loading };
}

export function AvailableLanguagesProvider({ children }) {
  const value = useAvailableLanguages();
  return (
    <AvailableLanguagesContext.Provider value={value}>{children}</AvailableLanguagesContext.Provider>
  );
}

export const AvailableLanguagesContext = React.createContext({
  languages: ["en"],
  labels: bundledLanguageNames,
  loading: true,
});

export function useAvailableLanguagesContext() {
  return React.useContext(AvailableLanguagesContext);
}
