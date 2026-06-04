import { buildFieldsTemplate, fieldsToJson, parseLocaleJson } from "./localeJson";

export { fieldsToJson, parseLocaleJson };

export const NAV_LOCALE_FIELDS = [
  { key: "home", label: "Home link" },
  { key: "waitlist", label: "Waitlist link" },
  { key: "guides", label: "Guides link" },
  { key: "fits", label: "Fits link" },
  { key: "skills", label: "Skills link" },
  { key: "iskPerHour", label: "ISK/h link" },
  { key: "fleet", label: "Fleet link" },
  { key: "fc", label: "FC link" },
  { key: "search", label: "Search link" },
  { key: "logIn", label: "Log in button" },
  { key: "logOut", label: "Log out button" },
];

export function isNavLocaleFile(filename) {
  return /^nav\.[a-z]{2}\.json$/.test(filename);
}

export function buildNavLocaleTemplate(source = {}) {
  return buildFieldsTemplate(NAV_LOCALE_FIELDS, source);
}
