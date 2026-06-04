import React from "react";
import styled from "styled-components";
import { useTranslation } from "react-i18next";
import { Content } from "../../Components/Page";
import { Markdown } from "../../Components/Markdown";
import { ToastContext } from "../../contexts";
import { errorToaster } from "../../api";
import { replaceTitle, parseMarkdownTitle } from "../../Util/title";
import { localeCode } from "../../i18n/localeCode";

export function resolveGuideAsset(slug, src) {
  if (!src || src.startsWith("http://") || src.startsWith("https://") || src.startsWith("/")) {
    return src;
  }
  const name = src.replace(/^\.\/?/, "");
  return `/api/v2/guides/${slug}/assets/${encodeURIComponent(name)}`;
}

const GuideContent = styled(Content)`
  max-width: 800px;

  img {
    max-width: 100%;
    height: auto;
  }
`;

export function GuideMarkdown({ slug, body }) {
  React.useEffect(() => {
    if (typeof body === "string") {
      replaceTitle(parseMarkdownTitle(body));
    }
  }, [body]);

  if (typeof body !== "string") {
    return (
      <GuideContent>
        <em>Loading...</em>
      </GuideContent>
    );
  }

  return (
    <GuideContent>
      <Markdown
        transformLinkUri={null}
        transformImageUri={(src) => resolveGuideAsset(slug, src)}
      >
        {body}
      </Markdown>
    </GuideContent>
  );
}

export function GuideViewer({ slug }) {
  const toastContext = React.useContext(ToastContext);
  const { i18n } = useTranslation();
  const locale = localeCode(i18n.resolvedLanguage);
  const [body, setBody] = React.useState(null);
  const [missing, setMissing] = React.useState(false);

  React.useEffect(() => {
    setBody(null);
    setMissing(false);
    const title = document.title;

    const loadGuide = async (guideLocale) => {
      const response = await fetch(`/api/v2/locales/${slug}/${guideLocale}`);
      if (response.ok) {
        return response.json();
      }
      if (guideLocale !== "en") {
        const fallback = await fetch(`/api/v2/locales/${slug}/en`);
        if (fallback.ok) {
          return fallback.json();
        }
      }
      throw new Error(`HTTP ${response.status}`);
    };

    errorToaster(
      toastContext,
      loadGuide(locale)
        .then((data) => {
          if (typeof data.body !== "string") {
            throw new Error("Guide content missing body field");
          }
          setBody(data.body);
          replaceTitle(parseMarkdownTitle(data.body));
        })
        .catch(() => {
          setMissing(true);
        })
    );

    return () => {
      document.title = title;
    };
  }, [toastContext, slug, locale]);

  if (missing) {
    return (
      <GuideContent>
        <strong>Not found!</strong> The guide could not be loaded.
      </GuideContent>
    );
  }

  if (!body) {
    return (
      <GuideContent>
        <em>Loading...</em>
      </GuideContent>
    );
  }

  return <GuideMarkdown slug={slug} body={body} />;
}
