import { buildFieldsTemplate, fieldsToJson, parseLocaleJson } from "./localeJson";
import { isGuideLocalePage, parseLocaleAdminFilename } from "./groupDataFiles";

export { fieldsToJson, parseLocaleJson };

export function isGuideLocaleFile(filename, fileMeta = null) {
  const parsed = parseLocaleAdminFilename(filename);
  if (!parsed) return false;
  if (fileMeta?.is_guide === true) return true;
  return isGuideLocalePage(parsed.page);
}

const GUIDE_META_FIELD_DEFS = [
  {
    key: "title",
    label: "Title",
    hint: "Shown on the guides index and FC menu. Falls back to the first # heading in body if empty.",
  },
  {
    key: "subtitle",
    label: "Subtitle",
    hint: "Short line under the title on guide cards.",
  },
  {
    key: "icon",
    label: "Icon URL",
    hint: "Optional image URL for the guide card (e.g. an EVE type icon).",
  },
  {
    key: "section",
    label: "Section",
    hint: "public = /guide index; fc = FC dashboard and /fc/slug routes.",
    options: ["public", "fc"],
  },
  {
    key: "access",
    label: "Access permission",
    hint: 'Required permission for FC guides (e.g. waitlist-tag:HQ-FC). Leave empty for public guides.',
  },
];

export const GUIDE_BODY_FIELD = {
  key: "body",
  label: "Body (markdown)",
  multiline: true,
  minHeight: "420px",
  hint: "Main guide content. The first # heading is used as the page title when no title field is set.",
};

/** Field order for the guide editor (metadata, then body, then any extra keys). */
export const GUIDE_EDITOR_FIELDS = [...GUIDE_META_FIELD_DEFS, GUIDE_BODY_FIELD];

export const GUIDE_PARSE_OPTIONS = { requiredKeys: ["body"] };

export function buildGuideLocaleTemplate(page = "guide") {
  const title = page.charAt(0).toUpperCase() + page.slice(1);
  return buildFieldsTemplate(GUIDE_EDITOR_FIELDS, {
    title: `${title} Guide`,
    subtitle: "",
    icon: "",
    section: "public",
    access: "",
    body: `# ${title}\n\n`,
  });
}
