import React from "react";
import { Link, useParams, useLocation } from "react-router-dom";
import i18n from "i18next";
import { I18nextProvider, initReactI18next, useTranslation } from "react-i18next";
import { Content } from "../../Components/Page";
import { NavButton, InputGroup, SelectAlt } from "../../Components/Form";
import { HomeContent } from "../Home/HomeContent";
import { LocaleDraftPreviewShell } from "./LocaleDraftPreviewLayout";

function createPreviewI18n(locale, translations) {
  const instance = i18n.createInstance();
  instance.use(initReactI18next).init({
    lng: locale,
    fallbackLng: locale,
    supportedLngs: [locale],
    resources: { [locale]: { home: translations } },
    ns: ["home"],
    defaultNS: "home",
    interpolation: { escapeValue: false },
    initImmediate: false,
    react: { useSuspense: false },
  });
  return instance;
}

function PreviewHomeLayout({ locale }) {
  const { t, ready } = useTranslation("home");
  if (!ready) {
    return <p style={{ padding: "2em" }}>Loading preview…</p>;
  }
  return (
    <>
      <InputGroup style={{ marginTop: "5em", alignItems: "center", gap: "0.75em" }}>
        <label htmlFor="preview-home-language">{t("language")}</label>
        <SelectAlt id="preview-home-language" value={locale} disabled>
          <option value="en">English</option>
          <option value="de">Deutsch</option>
        </SelectAlt>
        <NavButton to="/legal">{t("legal")}</NavButton>
      </InputGroup>
      <HomeContent />
    </>
  );
}

function DraftHomePreview({ locale, translations }) {
  const previewI18n = React.useMemo(
    () => createPreviewI18n(locale, translations),
    [locale, translations]
  );

  return (
    <I18nextProvider i18n={previewI18n}>
      <PreviewHomeLayout locale={locale} />
    </I18nextProvider>
  );
}

export function HomeLocalePreviewPage() {
  const { locale } = useParams();
  const location = useLocation();
  const params = new URLSearchParams(location.search);
  const filename = params.get("file") || `home.${locale}.json`;

  if (!locale) {
    return (
      <Content style={{ marginTop: "3em" }}>
        <h2>Unknown locale</h2>
        <p>
          <Link to="/admin/data-files">Back to Data Files</Link>
        </p>
      </Content>
    );
  }

  return (
    <LocaleDraftPreviewShell filename={filename} compareHref="/">
      {(translations) => <DraftHomePreview locale={locale} translations={translations} />}
    </LocaleDraftPreviewShell>
  );
}
