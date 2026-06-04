import React from "react";
import styled from "styled-components";
import { Button, Buttons } from "../../Components/Form";
import {
  ACCEPTED_IMAGE_TYPES,
  deleteGuideAsset,
  guideAssetMarkdown,
  guideAssetUrl,
  listGuideAssets,
  uploadGuideAsset,
} from "./guideAssetApi";

const Panel = styled.div`
  margin: 1.5em 0;
  padding-top: 1.25em;
  border-top: 1px solid ${(props) => props.theme.colors.accent2};
`;

const AssetGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 0.75em;
  margin-top: 0.75em;
`;

const AssetCard = styled.div`
  border: 1px solid ${(props) => props.theme.colors.accent2};
  border-radius: 6px;
  padding: 0.5em;
  display: flex;
  flex-direction: column;
  gap: 0.5em;
`;

const AssetPreview = styled.img`
  width: 100%;
  height: 90px;
  object-fit: contain;
  background: #111;
  border-radius: 4px;
`;

const AssetName = styled.code`
  font-size: 0.8em;
  word-break: break-all;
`;

const Hint = styled.div`
  font-size: 0.85em;
  opacity: 0.85;
  margin-top: 0.35em;
`;

const ErrorText = styled.p`
  color: #c44;
  margin: 0.5em 0 0;
`;

function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function GuideAssetsPanel({ slug }) {
  const [assets, setAssets] = React.useState([]);
  const [loading, setLoading] = React.useState(true);
  const [uploading, setUploading] = React.useState(false);
  const [error, setError] = React.useState(null);
  const fileInputRef = React.useRef(null);

  const loadAssets = React.useCallback(async () => {
    if (!slug) return;
    setLoading(true);
    setError(null);
    try {
      setAssets(await listGuideAssets(slug));
    } catch (e) {
      setError(e.message || String(e));
      setAssets([]);
    } finally {
      setLoading(false);
    }
  }, [slug]);

  React.useEffect(() => {
    loadAssets();
  }, [loadAssets]);

  const handleUpload = async (event) => {
    const files = Array.from(event.target.files ?? []);
    event.target.value = "";
    if (files.length === 0) return;

    setUploading(true);
    setError(null);
    try {
      for (const file of files) {
        await uploadGuideAsset(slug, file);
      }
      await loadAssets();
    } catch (e) {
      setError(e.message || String(e));
    } finally {
      setUploading(false);
    }
  };

  const handleDelete = async (filename) => {
    if (!window.confirm(`Delete ${filename}?`)) return;
    setError(null);
    try {
      await deleteGuideAsset(slug, filename);
      await loadAssets();
    } catch (e) {
      setError(e.message || String(e));
    }
  };

  const handleCopyMarkdown = async (filename) => {
    const markdown = guideAssetMarkdown(filename);
    try {
      await navigator.clipboard.writeText(markdown);
    } catch {
      setError("Could not copy markdown to clipboard.");
    }
  };

  if (!slug) return null;

  return (
    <Panel>
      <strong>Guide images</strong>
      <Hint>
        Upload PNG, JPG, GIF, WebP, or SVG files into <code>data/guides/{slug}/</code>. Files
        over 1 MB are resized (max 1600px); opaque PNG screenshots are saved as JPEG. Use{" "}
        <strong>Copy md</strong> for the saved filename — it may differ from the file you picked
        (e.g. <code>shot.png</code> → <code>shot.jpg</code>).
      </Hint>
      <Buttons style={{ marginTop: "0.75em" }}>
        <Button
          variant="secondary"
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
        >
          {uploading ? "Uploading…" : "Upload images"}
        </Button>
        <input
          ref={fileInputRef}
          type="file"
          accept={ACCEPTED_IMAGE_TYPES.join(",")}
          multiple
          style={{ display: "none" }}
          onChange={handleUpload}
        />
      </Buttons>
      {loading && <Hint>Loading images…</Hint>}
      {!loading && assets.length === 0 && <Hint>No images uploaded yet.</Hint>}
      {error && <ErrorText>{error}</ErrorText>}
      {assets.length > 0 && (
        <AssetGrid>
          {assets.map((asset) => (
            <AssetCard key={asset.name}>
              <AssetPreview src={guideAssetUrl(slug, asset.name)} alt={asset.name} />
              <AssetName>{asset.name}</AssetName>
              <Hint>{formatFileSize(asset.size)}</Hint>
              <Buttons>
                <Button variant="secondary" onClick={() => handleCopyMarkdown(asset.name)}>
                  Copy md
                </Button>
                <Button variant="danger" onClick={() => handleDelete(asset.name)}>
                  Delete
                </Button>
              </Buttons>
            </AssetCard>
          ))}
        </AssetGrid>
      )}
    </Panel>
  );
}
