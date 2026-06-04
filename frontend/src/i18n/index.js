import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import enHome from "../locales/en/home.json";
import deHome from "../locales/de/home.json";
import enNav from "../locales/en/nav.json";
import deNav from "../locales/de/nav.json";

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { home: enHome, nav: enNav },
      de: { home: deHome, nav: deNav },
    },
    fallbackLng: "en",
    supportedLngs: false,
    nonExplicitSupportedLngs: true,
    partialBundledLanguages: true,
    ns: ["home", "nav"],
    defaultNS: "home",
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: "locale",
      caches: ["localStorage"],
    },
    react: {
      bindI18n: "languageChanged loaded",
    },
  });

export default i18n;
