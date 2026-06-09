import React from "react";
import { NavLink } from "react-router-dom";
import styled from "styled-components";
import { apiCall, toaster } from "../../api";
import { AuthContext, ToastContext } from "../../contexts";
import { Content, PageTitle } from "../../Components/Page";
import { Table, Row, Cell, TableHead, TableBody, CellHead } from "../../Components/Table";
import { Button, Buttons } from "../../Components/Form";
import { usePageTitle } from "../../Util/title";
import { Modal, Confirm } from "../../Components/Modal";
import { isHomeLocaleFile, HOME_LOCALE_FIELDS } from "./localeHomeFields";
import { isNavLocaleFile, NAV_LOCALE_FIELDS } from "./localeNavFields";
import { isGuideLocaleFile, GUIDE_EDITOR_FIELDS, GUIDE_PARSE_OPTIONS } from "./guideLocalePages";
import { LocaleFileEditor } from "./LocaleFileEditor";
import { GuideAssetsPanel } from "./GuideAssetsPanel";
import { openGuideLocalePreview, openHomeLocalePreview } from "./localePreviewStorage";
import { parseLocaleAdminFilename } from "./groupDataFiles";
import { groupDataFiles, localeDisplayName } from "./groupDataFiles";
import { createLocaleFile, deleteLocaleFile, existingLocalesForPage } from "./localeFileApi";
import { AddLocaleFileModal, AddPageGroupModal } from "./LocaleFileModals";

const SectionHeader = styled.div`
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 0.75em;
  margin: 1.5em 0 0.75em;
  &:first-of-type {
    margin-top: 0;
  }
`;

const SectionTitle = styled.h3`
  margin: 0;
  font-weight: bold;
`;

const GroupToggleRow = styled(Row)`
  cursor: pointer;
  user-select: none;
  font-weight: 600;
  background-color: ${(props) => props.theme.colors.accent1} !important;
  &:hover {
    filter: brightness(1.05);
  }
`;

const NestedCell = styled(Cell)`
  padding-left: 2em;
`;

const ToggleIcon = styled.span`
  display: inline-block;
  width: 1.25em;
  margin-right: 0.35em;
`;

function formatFileSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function ConfigFileRow({ file, onEdit, onReload }) {
  return (
    <Row key={file.name}>
      <Cell>{file.name}</Cell>
      <Cell>{file.file_type}</Cell>
      <Cell>{formatFileSize(file.size)}</Cell>
      <Cell>{file.requires_reload ? "Yes" : "No"}</Cell>
      <Cell>
        <Buttons>
          <Button onClick={() => onEdit(file.name)}>Edit</Button>
          {file.requires_reload && <Button onClick={() => onReload(file.name)}>Reload</Button>}
        </Buttons>
      </Cell>
    </Row>
  );
}

function LocaleFileGroup({
  group,
  expanded,
  onToggle,
  onEdit,
  onDelete,
  onAddLanguage,
}) {
  return (
    <>
      <GroupToggleRow noAlternating>
        <Cell colSpan={4} onClick={onToggle}>
          <ToggleIcon>{expanded ? "▼" : "▶"}</ToggleIcon>
          {group.label}
          <span style={{ fontWeight: "normal", opacity: 0.85 }}>
            {" "}
            — {group.files.length} {group.files.length === 1 ? "language" : "languages"}
          </span>
        </Cell>
        <Cell>
          <Button
            variant="secondary"
            onClick={(e) => {
              e.stopPropagation();
              onAddLanguage(group.page);
            }}
          >
            Add language
          </Button>
        </Cell>
      </GroupToggleRow>
      {expanded &&
        group.files.map((file) => (
          <Row key={file.name} noAlternating>
            <NestedCell>
              {localeDisplayName(file.locale)}
              <span style={{ opacity: 0.65, marginLeft: "0.5em" }}>({file.name})</span>
            </NestedCell>
            <Cell>{file.file_type}</Cell>
            <Cell>{formatFileSize(file.size)}</Cell>
            <Cell>{file.requires_reload ? "Yes" : "No"}</Cell>
            <Cell>
              <Buttons>
                <Button onClick={() => onEdit(file.name)}>
                  {isHomeLocaleFile(file.name)
                    ? "Edit with preview"
                    : isGuideLocaleFile(file.name, file)
                    ? "Edit with preview"
                    : isNavLocaleFile(file.name)
                    ? "Edit labels"
                    : "Edit"}
                </Button>
                <Button variant="danger" onClick={() => onDelete(file.name)}>
                  Delete
                </Button>
              </Buttons>
            </Cell>
          </Row>
        ))}
    </>
  );
}

export function DataFiles() {
  const authContext = React.useContext(AuthContext);
  if (!authContext) {
    return (
      <Content>
        <b>Login Required!</b>
      </Content>
    );
  }
  return <DataFilesAdmin />;
}

function GuideLocaleEditorWrapper({ filename, initialContent, onSave, onClose }) {
  const guideSlug = parseLocaleAdminFilename(filename)?.page ?? null;
  return (
    <LocaleFileEditor
      filename={filename}
      initialContent={initialContent}
      onSave={onSave}
      onClose={onClose}
      fieldDefinitions={GUIDE_EDITOR_FIELDS}
      parseOptions={GUIDE_PARSE_OPTIONS}
      preview={{ openPreview: openGuideLocalePreview }}
      maxWidth="960px"
      draftNotice={
        <>
          <strong>Changes are draft until you save.</strong> Use{" "}
          <strong>Open full page preview</strong> to see the rendered guide in a new tab (uploaded
          images in <code>data/guides/{guideSlug ?? "slug"}/</code> appear in preview). Published
          copy updates only after <strong>Save & publish</strong>.
        </>
      }
      toolbarExtra={<GuideAssetsPanel slug={guideSlug} />}
    />
  );
}

function DataFilesAdmin() {
  const toastContext = React.useContext(ToastContext);
  const [files, setFiles] = React.useState(null);
  const [editingFile, setEditingFile] = React.useState(null);
  const [fileContent, setFileContent] = React.useState("");
  const [modalOpen, setModalOpen] = React.useState(false);
  const [expandedGroups, setExpandedGroups] = React.useState({});
  const [addLanguagePage, setAddLanguagePage] = React.useState(null);
  const [addPageOpen, setAddPageOpen] = React.useState(false);
  const [addGuideOpen, setAddGuideOpen] = React.useState(false);
  const [deleteTarget, setDeleteTarget] = React.useState(null);

  usePageTitle("Admin - Data Files");

  const loadFiles = React.useCallback(async () => {
    try {
      const data = await apiCall("/api/admin/data-files", {});
      setFiles(data.files);
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  }, [toastContext]);

  React.useEffect(() => {
    loadFiles();
  }, [loadFiles]);

  const grouped = React.useMemo(() => (files ? groupDataFiles(files) : null), [files]);

  React.useEffect(() => {
    if (!grouped) return;
    setExpandedGroups((prev) => {
      const next = { ...prev };
      for (const group of [...grouped.pageGroups, ...grouped.guideGroups]) {
        if (!(group.page in next)) {
          next[group.page] = true;
        }
      }
      return next;
    });
  }, [grouped]);

  const closeEditor = () => {
    setModalOpen(false);
    setEditingFile(null);
    setFileContent("");
  };

  const loadFileContent = async (filename) => {
    try {
      const response = await fetch(`/api/admin/data-files/${filename}`, {
        credentials: "include",
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      const content = await response.text();
      setFileContent(content);
      setEditingFile(filename);
      setModalOpen(true);
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const saveFileContent = async (body) => {
    const response = await fetch(`/api/admin/data-files/${editingFile}`, {
      method: "POST",
      headers: { "Content-Type": "text/plain" },
      credentials: "include",
      body,
    });
    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(errorText || `HTTP ${response.status}`);
    }
    const result = await response.text();
    toaster(toastContext, Promise.resolve(result || "File saved successfully"));
    loadFiles();
  };

  const saveFile = async () => {
    if (!editingFile) return;
    try {
      await saveFileContent(fileContent);
      closeEditor();
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const reloadFile = async (filename) => {
    try {
      const response = await fetch(`/api/admin/data-files/${filename}/reload`, {
        method: "POST",
        credentials: "include",
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `HTTP ${response.status}`);
      }

      const result = await response.text();
      toaster(toastContext, Promise.resolve(result || "File reloaded successfully"));
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const handleCreateLocale = async ({ page, locale, copyFromLocale, asGuide = false }) => {
    await createLocaleFile({ page, locale, copyFromLocale, asGuide });
    toaster(toastContext, Promise.resolve(`Created ${page}.${locale}.json`));
    setExpandedGroups((prev) => ({ ...prev, [page]: true }));
    await loadFiles();
  };

  const confirmDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteLocaleFile(deleteTarget);
      toaster(toastContext, Promise.resolve(`Deleted ${deleteTarget}`));
      setDeleteTarget(null);
      loadFiles();
    } catch (error) {
      toaster(toastContext, Promise.reject(error));
    }
  };

  const toggleGroup = (page) => {
    setExpandedGroups((prev) => ({ ...prev, [page]: !prev[page] }));
  };

  if (!grouped) {
    return (
      <Content>
        <PageTitle>Admin - Data Files</PageTitle>
        <p>Loading...</p>
      </Content>
    );
  }

  const editingHomeLocale = editingFile && isHomeLocaleFile(editingFile);
  const editingNavLocale = editingFile && isNavLocaleFile(editingFile);
  const editingFileMeta = editingFile ? files?.find((f) => f.name === editingFile) : null;
  const editingGuideLocale = editingFile && isGuideLocaleFile(editingFile, editingFileMeta);
  const existingPages = [...grouped.pageGroups, ...grouped.guideGroups].map((g) => g.page);

  const renderLocaleTable = (groups, emptyMessage) =>
    groups.length === 0 ? (
      <p style={{ opacity: 0.85 }}>{emptyMessage}</p>
    ) : (
      <Table fullWidth>
        <TableHead>
          <Row>
            <CellHead>Page / language</CellHead>
            <CellHead>Type</CellHead>
            <CellHead>Size</CellHead>
            <CellHead>Requires Reload</CellHead>
            <CellHead>Actions</CellHead>
          </Row>
        </TableHead>
        <TableBody>
          {groups.map((group) => (
            <LocaleFileGroup
              key={group.page}
              group={group}
              expanded={expandedGroups[group.page] !== false}
              onToggle={() => toggleGroup(group.page)}
              onEdit={loadFileContent}
              onDelete={setDeleteTarget}
              onAddLanguage={setAddLanguagePage}
            />
          ))}
        </TableBody>
      </Table>
    );

  return (
    <Content>
      <PageTitle>Admin - Data Files</PageTitle>
      <p style={{ marginBottom: "1em" }}>
        Page translations and guides use grouped <code>slug.en.json</code> files. Home page files
        include a form editor and full-page preview; guides support the same preview for markdown
        content; navigation labels live in{" "}
        <code>nav.*.json</code>; guides store markdown in the <code>body</code> field. A language
        appears in the site language selector only when both <code>home</code> and <code>nav</code>{" "}
        files exist for that locale. Changes go live after <strong>Save</strong>.
        ALSO fc training guide restricted for fc and fc guides restricted to hq-fc have special icons when they exist.
      </p>
      <Buttons style={{ marginBottom: "1em" }}>
        <NavLink to="/admin/skillplans" style={{ textDecoration: "none" }}>
          <Button>Skill Plans Admin</Button>
        </NavLink>
      </Buttons>

      {grouped.configFiles.length > 0 && (
        <>
          <SectionHeader>
            <SectionTitle>Configuration</SectionTitle>
          </SectionHeader>
          <Table fullWidth>
            <TableHead>
              <Row>
                <CellHead>File Name</CellHead>
                <CellHead>Type</CellHead>
                <CellHead>Size</CellHead>
                <CellHead>Requires Reload</CellHead>
                <CellHead>Actions</CellHead>
              </Row>
            </TableHead>
            <TableBody>
              {grouped.configFiles.map((file) => (
                <ConfigFileRow
                  key={file.name}
                  file={file}
                  onEdit={loadFileContent}
                  onReload={reloadFile}
                />
              ))}
            </TableBody>
          </Table>
        </>
      )}

      <SectionHeader>
        <SectionTitle>Page translations</SectionTitle>
        <Button variant="secondary" onClick={() => setAddPageOpen(true)}>
          Add page group
        </Button>
      </SectionHeader>

      {renderLocaleTable(grouped.pageGroups, "No page translation groups yet.")}

      <SectionHeader>
        <SectionTitle>Guides</SectionTitle>
        <Button variant="secondary" onClick={() => setAddGuideOpen(true)}>
          Add guide group
        </Button>
      </SectionHeader>

      {renderLocaleTable(
        grouped.guideGroups,
        "No guide groups yet. Add ddd, marauder, documentation, or trainee."
      )}

      <AddPageGroupModal
        open={addPageOpen}
        setOpen={setAddPageOpen}
        existingPages={existingPages}
        onSubmit={handleCreateLocale}
        title="Add page translation group"
        slugHint="Examples: home, legal"
      />

      <AddPageGroupModal
        open={addGuideOpen}
        setOpen={setAddGuideOpen}
        existingPages={existingPages}
        onSubmit={(args) => handleCreateLocale({ ...args, asGuide: true })}
        title="Add guide group"
        slugHint="Creates data/guides/slug/slug.en.json. Optional JSON fields: title, subtitle, icon, section (public|fc), access."
        suggestedSlugs={["ddd", "marauder", "documentation", "trainee"]}
      />

      <AddLocaleFileModal
        open={addLanguagePage !== null}
        setOpen={(open) => !open && setAddLanguagePage(null)}
        page={addLanguagePage ?? ""}
        existingLocales={
          addLanguagePage && files ? existingLocalesForPage(files, addLanguagePage) : []
        }
        onSubmit={({ locale, copyFromLocale }) =>
          handleCreateLocale({
            page: addLanguagePage,
            locale,
            copyFromLocale,
          })
        }
      />

      <Confirm
        open={deleteTarget !== null}
        setOpen={(open) => !open && setDeleteTarget(null)}
        title="Delete translation file?"
        onConfirm={confirmDelete}
      >
        <p>
          Delete <code>{deleteTarget}</code>? This removes the file from the server (a backup is kept
          on disk). This cannot be undone from the website.
        </p>
      </Confirm>

      <Modal
        open={modalOpen}
        setOpen={(open) => {
          if (!open) closeEditor();
        }}
        fill
      >
        {editingHomeLocale ? (
          <LocaleFileEditor
            filename={editingFile}
            initialContent={fileContent}
            onSave={saveFileContent}
            onClose={closeEditor}
            fieldDefinitions={HOME_LOCALE_FIELDS}
            preview={{ openPreview: openHomeLocalePreview }}
            draftNotice={
              <>
                <strong>Changes are draft until you save.</strong> Use{" "}
                <strong>Open full page preview</strong> to see the home page in its own tab (resize the
                browser window to check mobile/tablet). Published copy updates only after{" "}
                <strong>Save & publish</strong>.
              </>
            }
          />
        ) : editingNavLocale ? (
          <LocaleFileEditor
            filename={editingFile}
            initialContent={fileContent}
            onSave={saveFileContent}
            onClose={closeEditor}
            fieldDefinitions={NAV_LOCALE_FIELDS}
          />
        ) : editingGuideLocale ? (
          <GuideLocaleEditorWrapper
            filename={editingFile}
            initialContent={fileContent}
            onSave={saveFileContent}
            onClose={closeEditor}
          />
        ) : (
          editingFile && (
            <div style={{ minWidth: "800px", maxWidth: "90vw" }}>
              <h2>Edit {editingFile}</h2>
              <textarea
                value={fileContent}
                onChange={(e) => setFileContent(e.target.value)}
                style={{
                  width: "100%",
                  minHeight: "500px",
                  fontFamily: "monospace",
                  padding: "0.5em",
                  fontSize: "14px",
                }}
              />
              <Buttons style={{ marginTop: "1em" }}>
                <Button onClick={saveFile}>Save</Button>
                <Button onClick={closeEditor}>Cancel</Button>
              </Buttons>
            </div>
          )
        )}
      </Modal>
    </Content>
  );
}
