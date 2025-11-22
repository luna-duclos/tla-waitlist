import React from "react";
import { AuthContext } from "../../contexts";
import { usePageTitle } from "../../Util/title";
import { PageTitle } from "../../Components/Page";
import { Box } from "../../Components/Box";
import { Button, Input, NavButton, Radio, Textarea } from "../../Components/Form";
import { useLocation } from "react-router-dom";

export function SRPSubmit() {
  const authContext = React.useContext(AuthContext);
  const location = useLocation();

  const [killmailLink, setKillmailLink] = React.useState("");
  const [description, setDescription] = React.useState("");
  const [lootReturned, setLootReturned] = React.useState(null);
  const [charCount, setCharCount] = React.useState(0);
  const [isEditMode, setIsEditMode] = React.useState(false);
  const [editKillmailId, setEditKillmailId] = React.useState(null);
  const [loading, setLoading] = React.useState(false);

  usePageTitle("Submit SRP Report");

  // Check if we're in edit mode
  React.useEffect(() => {
    const params = new URLSearchParams(location.search);
    const editId = params.get("edit");
    if (editId) {
      setIsEditMode(true);
      setEditKillmailId(editId);
      loadReportForEdit(editId);
    }
  }, [location.search]);

  const loadReportForEdit = async (killmailId) => {
    setLoading(true);
    try {
      const response = await fetch(`/api/fc/srp/report/${killmailId}`);
      if (response.ok) {
        const data = await response.json();
        if (data.success && data.report) {
          setKillmailLink(data.report.killmail_link);
          setDescription(data.report.description || "");
          setLootReturned(data.report.loot_returned);
          setCharCount((data.report.description || "").length);
        }
      } else {
        alert("Error loading report for editing");
        window.location.href = "/fc/srp";
      }
    } catch (error) {
      alert("Error loading report for editing: " + error.message);
      window.location.href = "/fc/srp";
    } finally {
      setLoading(false);
    }
  };

  // Check access
  if (!authContext || !authContext.access["fleet-view"]) {
    return (
      <>
        <PageTitle>Submit SRP Report</PageTitle>
        <Box>
          <p>You do not have permission to submit SRP reports.</p>
        </Box>
      </>
    );
  }

  const handleDescriptionChange = (e) => {
    const value = e.target.value;
    if (value.length <= 1000) {
      setDescription(value);
      setCharCount(value.length);
    }
  };

  const handleKillmailLinkChange = (e) => {
    const value = e.target.value;
    if (value.length <= 1000) {
      setKillmailLink(value);
    }
  };

  const handleSubmit = async () => {
    if (!killmailLink.trim()) {
      alert("Please enter a killmail link.");
      return;
    }

    if (!description.trim()) {
      alert("Please enter a description.");
      return;
    }

    if (lootReturned === null) {
      alert("Please select whether loot was returned or not.");
      return;
    }

    const confirmMessage = isEditMode
      ? "Are you sure you want to update this SRP report?"
      : "Are you sure you want to submit this SRP report?";

    if (!window.confirm(confirmMessage)) {
      return;
    }

    try {
      const url = isEditMode ? `/api/fc/srp/update/${editKillmailId}` : "/api/fc/srp/submit";

      const body = isEditMode
        ? JSON.stringify({
            description: description.trim(),
            loot_returned: lootReturned,
          })
        : JSON.stringify({
            killmail_link: killmailLink.trim(),
            description: description.trim(),
            loot_returned: lootReturned,
          });

      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: body,
      });

      const result = await response.json();

      if (response.ok && result.success) {
        const message = isEditMode
          ? "SRP report updated successfully!"
          : "SRP report submitted successfully!";
        alert(message);
        window.location.href = "/fc/srp";
      } else {
        alert(
          "Error " +
            (isEditMode ? "updating" : "submitting") +
            " SRP report: " +
            (result.message || "Unknown error")
        );
      }
    } catch (error) {
      alert("Error " + (isEditMode ? "updating" : "submitting") + " SRP report: " + error.message);
    }
  };

  return (
    <>
      <PageTitle>{isEditMode ? "Update SRP Report" : "Submit SRP Report"}</PageTitle>

      <Box>
        <p>
          {isEditMode
            ? "Update the details of this SRP report."
            : "Use this form to submit a Ship Replacement Program (SRP) report for fleet losses."}
        </p>
        {!isEditMode && (
          <div style={{ marginBottom: "1em" }}>
            <label
              htmlFor="killmailLink"
              style={{ display: "block", marginBottom: "0.5em", fontWeight: "bold" }}
            >
              Killmail Link *
            </label>
            <Input
              id="killmailLink"
              type="text"
              value={killmailLink}
              onChange={handleKillmailLinkChange}
              placeholder="https://esi.evetech.net/latest/killmails/..."
              style={{ width: "100%" }}
              maxLength={1000}
            />
            <div style={{ fontSize: "12px", color: "#666", marginTop: "0.25em" }}>
              {killmailLink.length}/1000 characters
            </div>
          </div>
        )}

        <div style={{ marginBottom: "1em" }}>
          <label
            htmlFor="description"
            style={{ display: "block", marginBottom: "0.5em", fontWeight: "bold" }}
          >
            Description *
          </label>
          <Textarea
            id="description"
            value={description}
            onChange={handleDescriptionChange}
            placeholder="Describe the circumstances of the loss, what happened, anything that can help us get better in the future."
            style={{ width: "100%" }}
            // cols="100"
            rows="5"
            maxLength={1000}
          />
          <div style={{ fontSize: "12px", color: "#666", marginTop: "0.25em" }}>
            {charCount}/1000 characters
          </div>
        </div>

        <div style={{ marginBottom: "2em" }}>
          <div style={{ marginBottom: "0.5em", fontWeight: "bold" }}>
            Was loot returned to the pilot? *
          </div>
          <div style={{ display: "flex", gap: "1em" }}>
            <label style={{ display: "flex", alignItems: "center", cursor: "pointer" }}>
              <Radio
                name="lootReturned"
                value="true"
                checked={lootReturned === true}
                onChange={(e) => setLootReturned(e.target.value === "true")}
                style={{ marginRight: "0.5em" }}
              />
              <span>Yes</span>
            </label>
            <label style={{ display: "flex", alignItems: "center", cursor: "pointer" }}>
              <Radio
                name="lootReturned"
                value="false"
                checked={lootReturned === false}
                onChange={(e) => setLootReturned(e.target.value === "true")}
                style={{ marginRight: "0.5em" }}
              />
              <span>No</span>
            </label>
          </div>
        </div>

        <div style={{ display: "flex", gap: "1em" }}>
          <Button
            onClick={handleSubmit}
            disabled={loading}
            variant="success"
            style={{
              cursor: loading ? "not-allowed" : "pointer",
              opacity: loading ? 0.6 : 1,
            }}
          >
            {loading ? "Loading..." : isEditMode ? "Update SRP Report" : "Submit SRP Report"}
          </Button>
          <NavButton to="/fc/srp" variant="secondary">
            Cancel
          </NavButton>
        </div>
      </Box>
    </>
  );
}
