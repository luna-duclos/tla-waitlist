import { localeDisplayName as localeLabel, bundledLanguageNames } from "../../i18n/localeCode";

const LOCALE_PAGE_LABELS = {
  home: "Home page",
  nav: "Navigation",
};

const GUIDE_PAGE_LABELS = {
  ddd: "DDD guide",
  marauder: "Marauder guide",
  documentation: "FC documentation",
  trainee: "FC trainee guide",
};

export function isGuideLocalePage(page, guideSlugs = null) {
  if (page === "home" || page === "nav") {
    return false;
  }
  if (guideSlugs instanceof Set) {
    return guideSlugs.has(page);
  }
  if (Array.isArray(guideSlugs)) {
    return guideSlugs.includes(page);
  }
  return page in GUIDE_PAGE_LABELS;
}

export function localePageLabel(page) {
  if (page in LOCALE_PAGE_LABELS) return LOCALE_PAGE_LABELS[page];
  if (page in GUIDE_PAGE_LABELS) return GUIDE_PAGE_LABELS[page];
  return page;
}

/** Matches admin locale files like `home.en.json`. */
export function parseLocaleAdminFilename(filename) {
  const match = filename.match(/^([a-z][a-z0-9_-]*)\.([a-z]{2})\.json$/);
  if (!match) return null;
  return { page: match[1], locale: match[2] };
}

export function groupDataFiles(files) {
  const pageGroups = new Map();
  const guideGroups = new Map();
  const configFiles = [];

  for (const file of files) {
    const parsed = parseLocaleAdminFilename(file.name);
    if (parsed) {
      const isGuide = file.is_guide === true || isGuideLocalePage(parsed.page);
      const target = isGuide ? guideGroups : pageGroups;
      if (!target.has(parsed.page)) {
        target.set(parsed.page, {
          page: parsed.page,
          label: localePageLabel(parsed.page),
          files: [],
        });
      }
      target.get(parsed.page).files.push({ ...file, locale: parsed.locale });
    } else {
      configFiles.push(file);
    }
  }

  const sortGroups = (groups) => {
    for (const group of groups.values()) {
      group.files.sort((a, b) => a.locale.localeCompare(b.locale));
    }
    return [...groups.values()].sort((a, b) => a.page.localeCompare(b.page));
  };

  return {
    configFiles: configFiles.sort((a, b) => a.name.localeCompare(b.name)),
    pageGroups: sortGroups(pageGroups),
    guideGroups: sortGroups(guideGroups),
  };
}

export function localeDisplayName(locale, labels = bundledLanguageNames) {
  return localeLabel(locale, labels);
}
