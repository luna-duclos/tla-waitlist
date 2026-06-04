import { buildFieldsTemplate, fieldsToJson, parseLocaleJson } from "./localeJson";

export { fieldsToJson, parseLocaleJson };

/** Display order and labels for home.*.json locale files. */
export const HOME_LOCALE_FIELDS = [
  { key: "welcomeTitle", label: "Welcome heading" },
  { key: "intro", label: "Introduction", multiline: true, hint: "Use <guideLink>text</guideLink>, <discordLink>, <fitsLink>, <bold> for links and emphasis." },
  { key: "whatIsTlaTitle", label: "What is TLA — heading" },
  { key: "whatIsTlaBody", label: "What is TLA — text", multiline: true },
  { key: "armorShieldTitle", label: "Armor vs shield — heading" },
  { key: "armorShieldBody", label: "Armor vs shield — text", multiline: true },
  { key: "faqTitle", label: "FAQ section heading" },
  { key: "faqFitQuestion", label: "FAQ: different fit — question" },
  {
    key: "faqFitAnswer",
    label: "FAQ: different fit — answer",
    multiline: true,
    hint: "Use <must> for bold MUST.",
  },
  { key: "faqDpsQuestion", label: "FAQ: DPS — question" },
  { key: "faqDpsAnswer", label: "FAQ: DPS — answer", multiline: true },
  { key: "faqLogiQuestion", label: "FAQ: logi without L badge — question" },
  { key: "faqLogiAnswer", label: "FAQ: logi without L badge — answer", multiline: true },
  { key: "faqVindiQuestion", label: "FAQ: Vindicator — question" },
  {
    key: "faqVindiAnswer",
    label: "FAQ: Vindicator — answer",
    multiline: true,
    hint: "Use <dddLink>text</dddLink> for the DDD guide link.",
  },
  { key: "language", label: "Language selector label (shown on live home only)" },
  { key: "legal", label: "Legal button label (shown on live home only)" },
];

export function isHomeLocaleFile(filename) {
  return /^home\.[a-z]{2}\.json$/.test(filename);
}

export function localeFromHomeFilename(filename) {
  const match = filename.match(/^home\.([a-z]{2})\.json$/);
  return match ? match[1] : "en";
}

export function buildHomeLocaleTemplate(source = {}) {
  return buildFieldsTemplate(HOME_LOCALE_FIELDS, source);
}

export function buildGenericLocaleTemplate() {
  return JSON.stringify({ title: "", body: "" }, null, 2);
}

export function isLocaleAdminFile(filename) {
  return /^[a-z][a-z0-9_-]*\.[a-z]{2}\.json$/.test(filename);
}
