import React from "react";
import { Link } from "react-router-dom";
import styled from "styled-components";
import { Content } from "../../Components/Page";
import { loadLocaleDraft, localeDraftStorageKey } from "./localePreviewStorage";

export const DraftBanner = styled.div`
  background: ${(props) => props.theme.colors.accent2}44;
  border-bottom: 1px solid ${(props) => props.theme.colors.accent2};
  padding: 0.75em 1em;
  text-align: center;
  font-size: 0.95em;
  a {
    font-weight: bold;
    margin-left: 0.5em;
  }
`;

export function useLocaleDraft(filename) {
  const [draft, setDraft] = React.useState(() => loadLocaleDraft(filename));

  React.useEffect(() => {
    setDraft(loadLocaleDraft(filename));
    const onStorage = (event) => {
      if (event.key === localeDraftStorageKey(filename)) {
        setDraft(loadLocaleDraft(filename));
      }
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, [filename]);

  return draft;
}

export function LocaleDraftPreviewEmpty({ filename }) {
  return (
    <Content style={{ marginTop: "3em" }}>
      <h2>No draft preview available</h2>
      <p>
        Open the editor from{" "}
        <Link to="/admin/data-files">Admin → Data Files</Link>, edit <code>{filename}</code>, then
        click <strong>Open full page preview</strong>.
      </p>
      <p style={{ marginTop: "1em", opacity: 0.85 }}>
        If you already did that, try clicking preview again from the editor (draft is stored when you
        open preview).
      </p>
    </Content>
  );
}

export function LocaleDraftPreviewShell({ filename, getCompareHref, children }) {
  const draft = useLocaleDraft(filename);
  if (!draft) {
    return <LocaleDraftPreviewEmpty filename={filename} />;
  }

  const updated = new Date(draft.updatedAt).toLocaleString();
  const compareHref = getCompareHref ? getCompareHref(draft.data) : null;

  return (
    <>
      <DraftBanner>
        <strong>Draft preview</strong> — not published ({filename}). Last updated from editor:{" "}
        {updated}. Refresh this tab after editing to see changes.
        <Link to="/admin/data-files">Back to Data Files</Link>
        {compareHref && (
          <a href={compareHref} target="_blank" rel="noopener noreferrer">
            Compare with live page
          </a>
        )}
      </DraftBanner>
      {children(draft.data)}
    </>
  );
}
