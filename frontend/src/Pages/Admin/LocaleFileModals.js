import React from "react";
import styled from "styled-components";
import { Box } from "../../Components/Box";
import { Button, Buttons, Input, SelectAlt } from "../../Components/Form";
import { Modal } from "../../Components/Modal";
import { localeDisplayName } from "./groupDataFiles";

const Field = styled.div`
  margin-bottom: 1em;
`;

const Label = styled.label`
  display: block;
  font-weight: bold;
  margin-bottom: 0.35em;
`;

const Hint = styled.p`
  font-size: 0.9em;
  opacity: 0.85;
  margin: 0 0 1em;
`;

export function AddLocaleFileModal({ open, setOpen, page, existingLocales, onSubmit }) {
  const [locale, setLocale] = React.useState("");
  const [copyFrom, setCopyFrom] = React.useState("");
  const [submitting, setSubmitting] = React.useState(false);
  const [error, setError] = React.useState(null);

  React.useEffect(() => {
    if (!open) return;
    setLocale("");
    setCopyFrom(existingLocales[0] ?? "");
    setError(null);
  }, [open, existingLocales]);

  const handleSubmit = async () => {
    const code = locale.trim().toLowerCase();
    if (!/^[a-z]{2}$/.test(code)) {
      setError("Locale must be a 2-letter code (e.g. en, de, fr).");
      return;
    }
    if (existingLocales.includes(code)) {
      setError(`A file for ${code} already exists in this group.`);
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await onSubmit({ locale: code, copyFromLocale: copyFrom || null });
      setOpen(false);
    } catch (e) {
      setError(e.message || String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal open={open} setOpen={setOpen}>
      <Box>
        <h2 style={{ marginTop: 0 }}>Add language — {page}</h2>
        <Hint>
          Creates <code>{page}.XX.json</code>. Optionally copy strings from an existing language as
          a starting point.
        </Hint>
        <Field>
          <Label htmlFor="new-locale-code">Language code</Label>
          <Input
            id="new-locale-code"
            type="text"
            maxLength={2}
            placeholder="fr"
            value={locale}
            onChange={(e) => setLocale(e.target.value.toLowerCase())}
            style={{ width: "6em" }}
          />
        </Field>
        {existingLocales.length > 0 && (
          <Field>
            <Label htmlFor="copy-from-locale">Copy from (optional)</Label>
            <SelectAlt
              id="copy-from-locale"
              value={copyFrom}
              onChange={(e) => setCopyFrom(e.target.value)}
            >
              <option value="">Empty template</option>
              {existingLocales.map((loc) => (
                <option key={loc} value={loc}>
                  {localeDisplayName(loc)} ({page}.{loc}.json)
                </option>
              ))}
            </SelectAlt>
          </Field>
        )}
        {error && <p style={{ color: "#c44" }}>{error}</p>}
        <Buttons>
          <Button variant="primary" onClick={handleSubmit} disabled={submitting}>
            {submitting ? "Creating…" : "Create file"}
          </Button>
          <Button variant="secondary" onClick={() => setOpen(false)} disabled={submitting}>
            Cancel
          </Button>
        </Buttons>
      </Box>
    </Modal>
  );
}

export function AddPageGroupModal({
  open,
  setOpen,
  existingPages,
  onSubmit,
  title = "Add page translation group",
  slugHint = "Creates a new group with its first language file, e.g. legal.en.json. Use home for the home page template.",
  suggestedSlugs = [],
}) {
  const [page, setPage] = React.useState("");
  const [locale, setLocale] = React.useState("en");
  const [submitting, setSubmitting] = React.useState(false);
  const [error, setError] = React.useState(null);

  React.useEffect(() => {
    if (!open) return;
    setPage("");
    setLocale("en");
    setError(null);
  }, [open]);

  const handleSubmit = async () => {
    const slug = page.trim().toLowerCase();
    if (!/^[a-z][a-z0-9_-]*$/.test(slug)) {
      setError("Page slug must start with a letter and use lowercase letters, numbers, or hyphens.");
      return;
    }
    if (existingPages.includes(slug)) {
      setError(`Page group "${slug}" already exists. Add a language to it instead.`);
      return;
    }
    const code = locale.trim().toLowerCase();
    if (!/^[a-z]{2}$/.test(code)) {
      setError("Locale must be a 2-letter code.");
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await onSubmit({ page: slug, locale: code, copyFromLocale: null });
      setOpen(false);
    } catch (e) {
      setError(e.message || String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal open={open} setOpen={setOpen}>
      <Box>
        <h2 style={{ marginTop: 0 }}>{title}</h2>
        <Hint>{slugHint}</Hint>
        {suggestedSlugs.length > 0 && (
          <Hint>Suggested slugs: {suggestedSlugs.map((s) => `"${s}"`).join(", ")}</Hint>
        )}
        <Field>
          <Label htmlFor="new-page-slug">Page slug</Label>
          <Input
            id="new-page-slug"
            type="text"
            placeholder="legal"
            value={page}
            onChange={(e) => setPage(e.target.value.toLowerCase())}
            style={{ width: "100%" }}
          />
        </Field>
        <Field>
          <Label htmlFor="new-page-locale">First language</Label>
          <Input
            id="new-page-locale"
            type="text"
            maxLength={2}
            value={locale}
            onChange={(e) => setLocale(e.target.value.toLowerCase())}
            style={{ width: "6em" }}
          />
        </Field>
        {error && <p style={{ color: "#c44" }}>{error}</p>}
        <Buttons>
          <Button variant="primary" onClick={handleSubmit} disabled={submitting}>
            {submitting ? "Creating…" : "Create group"}
          </Button>
          <Button variant="secondary" onClick={() => setOpen(false)} disabled={submitting}>
            Cancel
          </Button>
        </Buttons>
      </Box>
    </Modal>
  );
}
