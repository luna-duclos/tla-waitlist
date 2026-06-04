import React from "react";
import styled from "styled-components";
import { useTranslation } from "react-i18next";
import { SelectAlt } from "./Form";
import { localeCode, localeDisplayName, languagesWithEnglishFirst } from "../i18n/localeCode";
import { useAvailableLanguagesContext } from "../i18n/useAvailableLanguages";
import { changeSiteLanguage } from "../i18n/loadLocaleOverrides";

const LangSelect = styled(SelectAlt)`
  border: unset;
  width: auto;
  min-width: 5em;
  max-width: 12em;
  padding: 0 0.5em;
`;

export function LanguageSelect() {
  const { i18n } = useTranslation();
  const { languages, labels, loading } = useAvailableLanguagesContext();
  const sortedLanguages = React.useMemo(
    () => languagesWithEnglishFirst(languages),
    [languages]
  );
  const resolved = localeCode(i18n.resolvedLanguage, languages);
  const [selected, setSelected] = React.useState(resolved);

  React.useEffect(() => {
    setSelected(resolved);
  }, [resolved]);

  if (loading || languages.length <= 1) {
    return null;
  }

  return (
    <LangSelect
      title="Language"
      aria-label="Language"
      value={selected}
      onChange={async (evt) => {
        const lng = evt.target.value;
        setSelected(lng);
        await changeSiteLanguage(lng);
      }}
    >
      {sortedLanguages.map((code) => (
        <option key={code} value={code}>
          {localeDisplayName(code, labels)}
        </option>
      ))}
    </LangSelect>
  );
}
