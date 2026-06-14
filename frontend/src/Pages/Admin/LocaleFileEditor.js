import React from "react";
import styled from "styled-components";
import { Box } from "../../Components/Box";
import { Button, Buttons, Input, SelectAlt, Textarea } from "../../Components/Form";
import { EMPTY_PARSE_OPTIONS, fieldsToJson, parseLocaleJson } from "./localeJson";
import { saveLocaleDraft } from "./localePreviewStorage";

const FieldBlock = styled.div`
  margin-bottom: 1em;
`;

const FieldLabel = styled.label`
  display: block;
  font-weight: bold;
  margin-bottom: 0.35em;
`;

const FieldHint = styled.div`
  font-size: 0.85em;
  opacity: 0.85;
  margin-top: 0.35em;
`;

const DraftNotice = styled.p`
  margin: 0 0 1em;
  padding: 0.5em 0.75em;
  border-radius: 6px;
  background: ${(props) => props.theme.colors.accent2}33;
  font-size: 0.95em;
`;

const ErrorText = styled.p`
  color: #c44;
  margin: 0.5em 0;
`;

export function LocaleFileEditor({
  filename,
  initialContent,
  onSave,
  onClose,
  fieldDefinitions,
  parseOptions = EMPTY_PARSE_OPTIONS,
  preview,
  draftNotice,
  toolbarExtra,
  maxWidth = "900px",
}) {
  const [fields, setFields] = React.useState({});
  const [rawMode, setRawMode] = React.useState(false);
  const [rawContent, setRawContent] = React.useState("");
  const [parseError, setParseError] = React.useState(null);
  const [saving, setSaving] = React.useState(false);

  const parseDraft = React.useCallback(
    (content) => parseLocaleJson(content, parseOptions),
    [parseOptions]
  );

  React.useEffect(() => {
    try {
      const parsed = parseLocaleJson(initialContent, parseOptions);
      setFields(parsed);
      setRawContent(fieldsToJson(parsed, fieldDefinitions));
      setParseError(null);
      setRawMode(false);
    } catch (e) {
      setParseError(e.message);
      setRawContent(initialContent);
      setRawMode(true);
    }
  }, [initialContent, filename, fieldDefinitions, parseOptions]);

  const draftFields = React.useMemo(() => {
    if (rawMode) {
      try {
        return parseDraft(rawContent);
      } catch {
        return fields;
      }
    }
    return fields;
  }, [rawMode, rawContent, fields, parseDraft]);

  React.useEffect(() => {
    if (preview && Object.keys(draftFields).length > 0) {
      saveLocaleDraft(filename, draftFields);
    }
  }, [filename, draftFields, preview]);

  const updateField = (key, value) => {
    setFields((prev) => {
      const next = { ...prev, [key]: value };
      setRawContent(fieldsToJson(next, fieldDefinitions));
      return next;
    });
    setParseError(null);
  };

  const toggleRawMode = () => {
    if (!rawMode) {
      setRawContent(fieldsToJson(fields, fieldDefinitions));
      setRawMode(true);
      return;
    }
    try {
      setFields(parseDraft(rawContent));
      setParseError(null);
      setRawMode(false);
    } catch (e) {
      setParseError(e.message);
    }
  };

  const openFullPreview = () => {
    try {
      const payload = rawMode ? parseDraft(rawContent) : fields;
      preview.openPreview(filename, payload);
    } catch (e) {
      setParseError(e.message);
    }
  };

  const handleSave = async () => {
    let payload;
    try {
      payload = rawMode ? rawContent : fieldsToJson(fields, fieldDefinitions);
      parseDraft(payload);
    } catch (e) {
      setParseError(e.message);
      return;
    }
    setSaving(true);
    try {
      await onSave(payload);
      onClose();
    } catch (e) {
      setParseError(e.message || String(e));
    } finally {
      setSaving(false);
    }
  };

  const defaultDraftNotice = preview ? (
    <>
      <strong>Changes are draft until you save.</strong> Use{" "}
      <strong>Open full page preview</strong> to see rendered content in a new tab. Published copy
      updates only after <strong>Save & publish</strong>.
    </>
  ) : (
    <>Changes go live after <strong>Save & publish</strong>.</>
  );

  return (
    <Box style={{ minWidth: "min(720px, 95vw)", maxWidth }}>
      <h2 style={{ marginTop: 0 }}>Edit {filename}</h2>
      <DraftNotice>{draftNotice ?? defaultDraftNotice}</DraftNotice>
      {parseError && <ErrorText>{parseError}</ErrorText>}

      <Buttons style={{ marginBottom: "1em" }}>
        <Button
          variant={rawMode ? "secondary" : "primary"}
          onClick={() => {
            if (rawMode) toggleRawMode();
          }}
        >
          Simple editor
        </Button>
        <Button
          variant={rawMode ? "primary" : "secondary"}
          onClick={() => {
            if (!rawMode) toggleRawMode();
          }}
        >
          Raw JSON
        </Button>
        {preview && (
          <Button variant="secondary" onClick={openFullPreview}>
            Open full page preview
          </Button>
        )}
      </Buttons>

      {toolbarExtra}

      {rawMode ? (
        <Textarea
          value={rawContent}
          onChange={(e) => {
            setRawContent(e.target.value);
            setParseError(null);
          }}
          style={{ width: "100%", minHeight: "520px", fontFamily: "monospace", fontSize: "14px" }}
        />
      ) : (
        <div style={{ maxHeight: "65vh", overflow: "auto", paddingRight: "0.5em" }}>
          {fieldDefinitions.map(({ key, label, multiline, hint, options, minHeight }) => (
            <FieldBlock key={key}>
              <FieldLabel htmlFor={`locale-${key}`}>{label}</FieldLabel>
              {options ? (
                <SelectAlt
                  id={`locale-${key}`}
                  value={fields[key] ?? options[0]}
                  onChange={(e) => updateField(key, e.target.value)}
                  style={{ width: "100%" }}
                >
                  {options.map((opt) => (
                    <option key={opt} value={opt}>
                      {opt}
                    </option>
                  ))}
                </SelectAlt>
              ) : multiline ? (
                <Textarea
                  id={`locale-${key}`}
                  value={fields[key] ?? ""}
                  onChange={(e) => updateField(key, e.target.value)}
                  style={{
                    width: "100%",
                    minHeight: minHeight ?? "4em",
                    fontFamily: key === "body" ? "monospace" : undefined,
                    fontSize: key === "body" ? "14px" : undefined,
                  }}
                />
              ) : (
                <Input
                  id={`locale-${key}`}
                  type="text"
                  value={fields[key] ?? ""}
                  onChange={(e) => updateField(key, e.target.value)}
                  style={{ width: "100%" }}
                />
              )}
              {hint && <FieldHint>{hint}</FieldHint>}
            </FieldBlock>
          ))}
        </div>
      )}

      <Buttons style={{ marginTop: "1.5em" }}>
        <Button variant="primary" onClick={handleSave} disabled={saving}>
          {saving ? "Saving…" : "Save & publish"}
        </Button>
        <Button variant="secondary" onClick={onClose} disabled={saving}>
          Cancel
        </Button>
      </Buttons>
    </Box>
  );
}
