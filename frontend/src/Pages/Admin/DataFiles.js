import React from "react";
import { NavLink } from "react-router-dom";
import { apiCall, toaster } from "../../api";
import { AuthContext, ToastContext } from "../../contexts";
import { Content, PageTitle } from "../../Components/Page";
import { Table, Row, Cell, TableHead, TableBody, CellHead } from "../../Components/Table";
import { Button, Buttons } from "../../Components/Form";
import { usePageTitle } from "../../Util/title";
import { Modal } from "../../Components/Modal";

export function DataFiles() {
  const authContext = React.useContext(AuthContext);
  if (!authContext) {
    return (
      <Content>
        <b>Login Required!</b>
      </Content>
    );
  }
  return <DataFilesAdmin authContext={authContext} />;
}

function DataFilesAdmin({ authContext }) {
  const toastContext = React.useContext(ToastContext);
  const [files, setFiles] = React.useState(null);
  const [editingFile, setEditingFile] = React.useState(null);
  const [fileContent, setFileContent] = React.useState("");
  const [modalOpen, setModalOpen] = React.useState(false);

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

  const saveFile = async () => {
    if (!editingFile) return;

    try {
      const response = await fetch(`/api/admin/data-files/${editingFile}`, {
        method: "POST",
        headers: {
          "Content-Type": "text/plain",
        },
        credentials: "include",
        body: fileContent,
      });

      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || `HTTP ${response.status}`);
      }

      const result = await response.text();
      toaster(toastContext, Promise.resolve(result || "File saved successfully"));
      setModalOpen(false);
      setEditingFile(null);
      setFileContent("");
      loadFiles(); // Reload file list
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

  const formatFileSize = (bytes) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  if (!files) {
    return (
      <Content>
        <PageTitle>Admin - Data Files</PageTitle>
        <p>Loading...</p>
      </Content>
    );
  }

  return (
    <Content>
      <PageTitle>Admin - Data Files</PageTitle>
      <Buttons style={{ marginBottom: "1em" }}>
        <NavLink to="/admin/skillplans" style={{ textDecoration: "none" }}>
          <Button>Skill Plans Admin</Button>
        </NavLink>
      </Buttons>
      <Table>
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
          {files.map((file) => (
            <Row key={file.name}>
              <Cell>{file.name}</Cell>
              <Cell>{file.file_type}</Cell>
              <Cell>{formatFileSize(file.size)}</Cell>
              <Cell>{file.requires_reload ? "Yes" : "No"}</Cell>
              <Cell>
                <Buttons>
                  <Button onClick={() => loadFileContent(file.name)}>Edit</Button>
                  {file.requires_reload && (
                    <Button onClick={() => reloadFile(file.name)}>Reload</Button>
                  )}
                </Buttons>
              </Cell>
            </Row>
          ))}
        </TableBody>
      </Table>

      <Modal open={modalOpen} setOpen={setModalOpen}>
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
            <Button onClick={() => setModalOpen(false)}>Cancel</Button>
          </Buttons>
        </div>
      </Modal>
    </Content>
  );
}

