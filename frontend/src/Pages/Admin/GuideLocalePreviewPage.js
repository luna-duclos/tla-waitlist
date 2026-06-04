import { Link, useParams, useLocation } from "react-router-dom";
import { Content } from "../../Components/Page";
import { GuideMarkdown } from "../Guide/GuideViewer";
import { guidePath } from "../Guide/useGuides";
import { LocaleDraftPreviewShell } from "./LocaleDraftPreviewLayout";

export function GuideLocalePreviewPage() {
  const { slug, locale } = useParams();
  const location = useLocation();
  const params = new URLSearchParams(location.search);
  const filename = params.get("file") || `${slug}.${locale}.json`;

  if (!slug || !locale) {
    return (
      <Content style={{ marginTop: "3em" }}>
        <h2>Invalid preview URL</h2>
        <p>
          <Link to="/admin/data-files">Back to Data Files</Link>
        </p>
      </Content>
    );
  }

  return (
    <LocaleDraftPreviewShell
      filename={filename}
      getCompareHref={(fields) => {
        const section = fields.section === "fc" ? "fc" : "public";
        return guidePath({ slug, section });
      }}
    >
      {(fields) => <GuideMarkdown slug={slug} body={fields.body} />}
    </LocaleDraftPreviewShell>
  );
}
