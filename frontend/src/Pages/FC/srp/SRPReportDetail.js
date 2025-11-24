import React from "react";
import { useApi } from "../../../api";
import { usePageTitle } from "../../../Util/title";
import { PageTitle } from "../../../Components/Page";
import { Box } from "../../../Components/Box";
import { Button } from "../../../Components/Form";
import { Modal } from "../../../Components/Modal";
import { formatNumber } from "../../../Util/number";
import { formatDatetime } from "../../../Util/time";
import { useStatusStyle } from "./util";

export function SRPReportDetail() {
  const reportId = new URLSearchParams(window.location.search).get("id");
  const [reportData, reportError] = useApi(reportId ? `/api/admin/srp/reports/${reportId}` : null);
  const getStatusStyle = useStatusStyle();

  // Helper function to parse SRP paid JSON and get boolean status
  const getSrpPaidStatus = (srpPaidData) => {
    if (!srpPaidData) return false;
    try {
      const parsed = JSON.parse(srpPaidData);
      return parsed.had_coverage === true;
    } catch (e) {
      // Fallback for old boolean data
      return srpPaidData === true || srpPaidData === "true";
    }
  };
  const [killmailData] = useApi(
    reportId ? `/api/admin/srp/reports/${reportId}/killmail/enriched` : null
  );
  const [fleetValidation, fleetValidationError] = useApi(
    reportId ? `/api/admin/srp/reports/${reportId}/fleet-validation` : null
  );
  const [srpValidation, srpValidationError] = useApi(
    reportId ? `/api/admin/srp/reports/${reportId}/srp-validation` : null
  );
  const [appraisalResult, setAppraisalResult] = React.useState(null);
  const [appraisalLoading, setAppraisalLoading] = React.useState(false);
  const [appraisalError, setAppraisalError] = React.useState(null);
  const [appraisalRun, setAppraisalRun] = React.useState(false);
  const [showItemBreakdown, setShowItemBreakdown] = React.useState(false);
  const [showDenyModal, setShowDenyModal] = React.useState(false);
  const [showApproveModal, setShowApproveModal] = React.useState(false);
  const [denyReason, setDenyReason] = React.useState("");
  const [payoutAmount, setPayoutAmount] = React.useState("");
  const [actionLoading, setActionLoading] = React.useState(false);

  // Debug logging for fleet validation
  console.log("Fleet Validation Data:", fleetValidation);
  console.log("Fleet Validation Error:", fleetValidationError);
  console.log("Fleet Validation Type:", typeof fleetValidation);
  console.log("Fleet Validation Keys:", fleetValidation ? Object.keys(fleetValidation) : "null");
  console.log("Fleet Validation Stringified:", JSON.stringify(fleetValidation));

  usePageTitle("SRP Report Details");

  const handleBack = () => {
    window.history.back();
  };

  const handleApprove = async () => {
    // First, open the victim's character window in-game
    try {
      const response = await fetch(`/api/admin/srp/reports/${reportId}/open-victim-window`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      if (!response.ok) {
        console.warn("Failed to open victim character window:", response.status);
        // Don't fail the approval process if window opening fails
      } else {
        const result = await response.json();
        console.log("Character window opened:", result.message);
      }
    } catch (error) {
      console.warn("Error opening character window:", error);
      // Don't fail the approval process if window opening fails
    }

    // Then open the approve modal
    setShowApproveModal(true);
  };

  const handleApproveConfirm = async () => {
    if (!reportId || !payoutAmount.trim()) {
      alert("Please enter a payout amount");
      return;
    }

    const amount = parseFloat(payoutAmount);
    if (isNaN(amount) || amount <= 0) {
      alert("Please enter a valid payout amount");
      return;
    }

    setActionLoading(true);
    try {
      const response = await fetch(`/api/admin/srp/reports/${reportId}/approve`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          payout_amount: amount,
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const result = await response.json();
      alert(result.message);
      setShowApproveModal(false);
      setPayoutAmount("");
      window.location.reload(); // Refresh to show updated status
    } catch (error) {
      alert(`Error approving SRP report: ${error.message}`);
    } finally {
      setActionLoading(false);
    }
  };

  const handleDeny = () => {
    console.log("Deny button clicked, opening modal");
    console.log("Current showDenyModal state:", showDenyModal);
    setShowDenyModal(true);
    console.log("Set showDenyModal to true");
  };

  const handleDenyConfirm = async () => {
    console.log("Deny confirm clicked, reason:", denyReason);
    if (!reportId || !denyReason.trim()) {
      alert("Please provide a reason for denial");
      return;
    }

    setActionLoading(true);
    try {
      console.log("Making deny request to:", `/api/admin/srp/reports/${reportId}/deny`);
      const response = await fetch(`/api/admin/srp/reports/${reportId}/deny`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          reason: denyReason.trim(),
        }),
      });

      console.log("Deny response status:", response.status);
      if (!response.ok) {
        const errorText = await response.text();
        console.log("Deny error response:", errorText);
        throw new Error(`HTTP error! status: ${response.status} - ${errorText}`);
      }

      const result = await response.json();
      console.log("Deny success result:", result);
      alert(result.message);
      setShowDenyModal(false);
      setDenyReason("");
      window.location.reload(); // Refresh to show updated status
    } catch (error) {
      console.log("Deny error:", error);
      alert(`Error denying SRP report: ${error.message}`);
    } finally {
      setActionLoading(false);
    }
  };

  // Extract the report from the response structure
  const report = reportData?.report;

  const handleCalculateAppraisal = React.useCallback(
    async (appraisalType = "auto") => {
      if (!killmailData) return;

      setAppraisalLoading(true);
      setAppraisalError(null);

      try {
        let itemsToAppraise = [];

        if (appraisalType === "auto") {
          // Auto logic based on loot returned status
          if (report && report.loot_returned) {
            // Loot returned = true: only destroyed items
            const destroyedItems = killmailData.victim.items
              .filter((item) => item.quantity_destroyed && item.quantity_destroyed > 0)
              .map((item) => `${item.item_name} ${item.quantity_destroyed}`);

            // Add ship hull
            const shipHull = `${killmailData.victim.ship_name} 1`;
            itemsToAppraise = [shipHull, ...destroyedItems];
          } else {
            // Loot returned = false: everything (destroyed + dropped)
            const allItems = killmailData.victim.items
              .filter(
                (item) =>
                  (item.quantity_destroyed && item.quantity_destroyed > 0) ||
                  (item.quantity_dropped && item.quantity_dropped > 0)
              )
              .map((item) => {
                const totalQuantity = (item.quantity_destroyed || 0) + (item.quantity_dropped || 0);
                return `${item.item_name} ${totalQuantity}`;
              });

            // Add ship hull
            const shipHull = `${killmailData.victim.ship_name} 1`;
            itemsToAppraise = [shipHull, ...allItems];
          }
        } else if (appraisalType === "destroyed") {
          // Manual: only destroyed items
          const destroyedItems = killmailData.victim.items
            .filter((item) => item.quantity_destroyed && item.quantity_destroyed > 0)
            .map((item) => `${item.item_name} ${item.quantity_destroyed}`);

          // Add ship hull
          const shipHull = `${killmailData.victim.ship_name} 1`;
          itemsToAppraise = [shipHull, ...destroyedItems];
        } else if (appraisalType === "everything") {
          // Manual: everything (destroyed + dropped)
          const allItems = killmailData.victim.items
            .filter(
              (item) =>
                (item.quantity_destroyed && item.quantity_destroyed > 0) ||
                (item.quantity_dropped && item.quantity_dropped > 0)
            )
            .map((item) => {
              const totalQuantity = (item.quantity_destroyed || 0) + (item.quantity_dropped || 0);
              return `${item.item_name} ${totalQuantity}`;
            });

          // Add ship hull
          const shipHull = `${killmailData.victim.ship_name} 1`;
          itemsToAppraise = [shipHull, ...allItems];
        }

        const response = await fetch("/api/admin/srp/appraisal", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            destroyed_items: itemsToAppraise,
          }),
        });

        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }

        const result = await response.json();
        setAppraisalResult(result);
      } catch (error) {
        setAppraisalError(error.message);
      } finally {
        setAppraisalLoading(false);
      }
    },
    [killmailData, report]
  );

  // Auto-run appraisal when killmail data loads
  React.useEffect(() => {
    if (killmailData && !appraisalRun && !appraisalLoading && !appraisalResult) {
      handleCalculateAppraisal("auto");
      setAppraisalRun(true);
    }
  }, [killmailData, appraisalRun, appraisalLoading, appraisalResult, handleCalculateAppraisal]);

  // Pre-fill payout amount when approve modal opens
  React.useEffect(() => {
    if (showApproveModal && appraisalResult && appraisalResult.total_value) {
      setPayoutAmount(appraisalResult.total_value.toString());
    }
  }, [showApproveModal, appraisalResult]);

  // Debug logging
  console.log("Report ID:", reportId);
  console.log("Report Data:", reportData);
  console.log("Report Data Type:", typeof reportData);
  console.log("Report Data Keys:", reportData ? Object.keys(reportData) : "null");
  console.log("Report Error:", reportError);
  console.log("Report:", report);
  console.log("!reportId:", !reportId);
  console.log("reportError:", reportError);
  console.log("!reportData:", !reportData);
  console.log("!report:", !report);

  if (!reportId) {
    return (
      <>
        <PageTitle>SRP Report Details</PageTitle>
        <Box>
          <p>No report ID provided.</p>
          <Button onClick={handleBack}>Back to Reports</Button>
        </Box>
      </>
    );
  }

  if (reportError && typeof reportError !== "function") {
    return (
      <>
        <PageTitle>SRP Report Details</PageTitle>
        <Box>
          <p>Error loading report: {reportError.message || "Unknown error"}</p>
          <p>Error details: {JSON.stringify(reportError)}</p>
          <p>Report ID: {reportId}</p>
          <Button onClick={handleBack}>Back to Reports</Button>
        </Box>
      </>
    );
  }

  if (!reportData) {
    return (
      <>
        <PageTitle>SRP Report Details</PageTitle>
        <Box>
          <p>Loading...</p>
        </Box>
      </>
    );
  }

  if (!report) {
    return (
      <>
        <PageTitle>SRP Report Details</PageTitle>
        <Box>
          <p>Report not found or error loading data.</p>
          <Button onClick={handleBack}>Back to Reports</Button>
        </Box>
      </>
    );
  }

  return (
    <>
      <PageTitle>SRP Report Details</PageTitle>

      <Box>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            marginBottom: "1em",
          }}
        >
          <h3>Report #{report.killmail_id}</h3>
          <Button onClick={handleBack}>Back to Reports</Button>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "1em" }}>
          <div>
            <h4>Basic Information</h4>
            <table style={{ width: "100%" }}>
              <tbody>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Killmail ID:</td>
                  <td style={{ padding: "0.5em 0" }}>{report.killmail_id}</td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Submitted By:</td>
                  <td style={{ padding: "0.5em 0" }}>{report.submitted_by_name}</td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Submitted At:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    {formatDatetime(new Date(report.submitted_at * 1000))}
                  </td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Status:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    <span
                      style={{
                        padding: "0.25em 0.5em",
                        borderRadius: "3px",
                        ...getStatusStyle(report.status),
                      }}
                    >
                      {report.status.charAt(0).toUpperCase() + report.status.slice(1)}
                    </span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div>
            <h4>Killmail Information</h4>
            <table style={{ width: "100%" }}>
              <tbody>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Killmail Link:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    <a href={report.killmail_link} target="_blank" rel="noopener noreferrer">
                      View Killmail (raw)
                    </a>
                  </td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Loot Returned:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    <span
                      style={{
                        padding: "0.25em 0.5em",
                        borderRadius: "3px",
                        backgroundColor: report.loot_returned ? "#d4edda" : "#f8d7da",
                        color: report.loot_returned ? "#155724" : "#721c24",
                      }}
                    >
                      {report.loot_returned ? "Yes" : "No"}
                    </span>
                  </td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Payout Amount:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    {report.payout_amount ? formatNumber(report.payout_amount) + " ISK" : "N/A"}
                  </td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Payout Date:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    {report.payout_date
                      ? formatDatetime(new Date(report.payout_date * 1000))
                      : "N/A"}
                  </td>
                </tr>
                <tr>
                  <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>SRP Paid:</td>
                  <td style={{ padding: "0.5em 0" }}>
                    <span
                      style={{
                        padding: "0.25em 0.5em",
                        borderRadius: "3px",
                        backgroundColor: getSrpPaidStatus(report.srp_paid) ? "#d4edda" : "#f8d7da",
                        color: getSrpPaidStatus(report.srp_paid) ? "#155724" : "#721c24",
                      }}
                    >
                      {getSrpPaidStatus(report.srp_paid) ? "Yes" : "No"}
                    </span>
                  </td>
                </tr>
                {report.reason && (
                  <tr>
                    <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Reason:</td>
                    <td style={{ padding: "0.5em 0" }}>
                      <div
                        style={{
                          padding: "0.5em",
                          backgroundColor: "#f8f9fa",
                          borderRadius: "4px",
                          border: "1px solid #dee2e6",
                        }}
                      >
                        {report.reason}
                      </div>
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </div>

        <div style={{ marginTop: "2em" }}>
          <h4>Description</h4>
          <div
            style={{
              padding: "1em",
              backgroundColor: "#f8f9fa",
              borderRadius: "4px",
              border: "1px solid #dee2e6",
              minHeight: "100px",
              color: "black",
            }}
          >
            {report.description || "No description provided"}
          </div>
        </div>

        {/* Fleet Validation */}
        <div style={{ marginTop: "2em" }}>
          <h4>SRP Validation</h4>
          <div
            style={{
              padding: "1em",
              backgroundColor: "white",
              borderRadius: "4px",
              border: "1px solid #dee2e6",
            }}
          >
            {fleetValidationError && typeof fleetValidationError !== "function" ? (
              <div
                style={{
                  color: "#721c24",
                  backgroundColor: "#f8d7da",
                  padding: "0.5em",
                  borderRadius: "4px",
                }}
              >
                Error checking fleet membership: {fleetValidationError.message || "Unknown error"}
              </div>
            ) : fleetValidation === null ? (
              <div style={{ color: "#6c757d" }}>Checking fleet membership...</div>
            ) : (
              <div style={{ display: "flex", flexDirection: "column", gap: "1em" }}>
                <div style={{ display: "flex", alignItems: "center", gap: "1em" }}>
                  <span style={{ fontWeight: "bold", color: "black" }}>
                    Fleet Membership at Death:
                  </span>
                  <span
                    style={{
                      padding: "0.25em 0.75em",
                      borderRadius: "20px",
                      backgroundColor: fleetValidation.was_in_fleet ? "#d4edda" : "#f8d7da",
                      color: fleetValidation.was_in_fleet ? "#155724" : "#721c24",
                      fontWeight: "bold",
                    }}
                  >
                    {fleetValidation.was_in_fleet ? "✓ IN FLEET" : "✗ NOT IN FLEET"}
                  </span>
                </div>

                {srpValidationError && typeof srpValidationError !== "function" ? (
                  <div
                    style={{
                      color: "#721c24",
                      backgroundColor: "#f8d7da",
                      padding: "0.5em",
                      borderRadius: "4px",
                    }}
                  >
                    Error checking SRP coverage: {srpValidationError.message || "Unknown error"}
                  </div>
                ) : srpValidation === null ? (
                  <div style={{ color: "#6c757d" }}>Checking SRP coverage...</div>
                ) : (
                  <div style={{ display: "flex", alignItems: "center", gap: "1em" }}>
                    <span style={{ fontWeight: "bold", color: "black" }}>
                      SRP Coverage at Death:
                    </span>
                    <div style={{ display: "flex", flexDirection: "column", gap: "0.5em" }}>
                      <span
                        style={{
                          padding: "0.25em 0.75em",
                          borderRadius: "20px",
                          backgroundColor: srpValidation.alt_needs_linking
                            ? "#fff3cd"
                            : srpValidation.had_srp_coverage
                            ? "#d4edda"
                            : "#f8d7da",
                          color: srpValidation.alt_needs_linking
                            ? "#856404"
                            : srpValidation.had_srp_coverage
                            ? "#155724"
                            : "#721c24",
                          fontWeight: "bold",
                        }}
                      >
                        {srpValidation.alt_needs_linking
                          ? "⚠ ALT NEEDS LINKING"
                          : srpValidation.had_srp_coverage
                          ? "✓ HAD COVERAGE"
                          : "✗ NO COVERAGE"}
                      </span>
                      {srpValidation.had_srp_coverage &&
                        srpValidation.payment_date &&
                        srpValidation.payment_amount && (
                          <div style={{ fontSize: "0.9em", color: "#6c757d" }}>
                            {srpValidation.coverage_character && srpValidation.coverage_type && (
                              <div style={{ fontWeight: "bold", marginBottom: "0.25em" }}>
                                {srpValidation.coverage_type === "direct" && "Direct Coverage"}
                                {srpValidation.coverage_type === "main" &&
                                  `Coverage via main: ${srpValidation.coverage_character}`}
                                {srpValidation.coverage_type === "alt" &&
                                  `Coverage via alt: ${srpValidation.coverage_character}`}
                              </div>
                            )}
                            <div>Payment: {srpValidation.payment_amount.toLocaleString()} ISK</div>
                            <div>Date: {srpValidation.payment_date}</div>
                            {srpValidation.expires_at && (
                              <div>Expires: {srpValidation.expires_at}</div>
                            )}
                          </div>
                        )}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        {killmailData && (
          <div style={{ marginTop: "2em" }}>
            <h4>Killmail Information</h4>

            {/* Debug logging */}
            {console.log("Killmail Data:", killmailData)}
            {console.log("Victim Data:", killmailData.victim)}
            {console.log("Items Data:", killmailData.victim?.items)}

            {/* Ship Fit Display - COMMENTED OUT */}
            {/*
            {killmailData.victim && killmailData.victim.items && (
              <div>
                {(() => {
                  try {
                    return (
                      <ShipFitDisplay 
                        shipTypeId={killmailData.victim.ship_type_id}
                        shipName={killmailData.victim.ship_name}
                        items={killmailData.victim.items}
                      />
                    );
                  } catch (error) {
                    console.error('Error rendering ShipFitDisplay:', error);
                    return (
                      <div style={{ padding: '1em', backgroundColor: '#f8d7da', border: '1px solid #f5c6cb', borderRadius: '4px', color: '#721c24' }}>
                        Error rendering ship fitting: {error.message}
                      </div>
                    );
                  }
                })()}
              </div>
            )}
            */}

            {/* Kill Summary */}
            <div
              style={{
                borderRadius: "8px",
                padding: "1.5em",
                border: "1px solid #dee2e6",
                marginBottom: "1em",
              }}
            >
              <h4 style={{ margin: "0 0 1em 0", color: "white" }}>Kill Summary</h4>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "2em" }}>
                <div>
                  <h5 style={{ margin: "0 0 0.5em 0", color: "white" }}>Victim Information</h5>
                  <table style={{ width: "100%" }}>
                    <tbody>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0", width: "40%" }}>
                          Character:
                        </td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.victim.character_name ||
                            killmailData.victim.character_id ||
                            "N/A"}
                          {killmailData.victim.character_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {killmailData.victim.character_id})
                            </span>
                          )}
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Corporation:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.victim.corporation_name ||
                            killmailData.victim.corporation_id ||
                            "N/A"}
                          {killmailData.victim.corporation_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {killmailData.victim.corporation_id})
                            </span>
                          )}
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Alliance:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.victim.alliance_name ||
                            killmailData.victim.alliance_id ||
                            "N/A"}
                          {killmailData.victim.alliance_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {killmailData.victim.alliance_id})
                            </span>
                          )}
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Ship:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.victim.ship_name}
                          <span style={{ fontSize: "0.9em" }}>
                            {" "}
                            (ID: {killmailData.victim.ship_type_id})
                          </span>
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Damage Taken:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.victim.damage_taken.toLocaleString()}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>

                <div>
                  <h5 style={{ margin: "0 0 0.5em 0", color: "white" }}>Kill Information</h5>
                  <table style={{ width: "100%" }}>
                    <tbody>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0", width: "40%" }}>
                          Killmail ID:
                        </td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.killmail_id}
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Kill Time:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.killmail_time}
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Solar System:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.solar_system_name || "Unknown"}
                          <span style={{ fontSize: "0.9em" }}>
                            {" "}
                            (ID: {killmailData.solar_system_id})
                          </span>
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>Moon ID:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.moon_id || "N/A"}
                        </td>
                      </tr>
                      <tr>
                        <td style={{ fontWeight: "bold", padding: "0.5em 0" }}>War ID:</td>
                        <td style={{ padding: "0.5em 0", color: "white" }}>
                          {killmailData.war_id || "N/A"}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>

            {/* Abyssal Modules Lost */}
            <div
              style={{
                borderRadius: "8px",
                padding: "1.5em",
                border: "1px solid #dee2e6",
                marginBottom: "1em",
              }}
            >
              <h4 style={{ margin: "0 0 1em 0", color: "white" }}>Abyssal Modules Lost</h4>
              {(() => {
                // Count destroyed abyssal modules
                const abyssalModules = killmailData.victim.items
                  .filter(
                    (item) =>
                      item.quantity_destroyed &&
                      item.quantity_destroyed > 0 &&
                      item.item_name.toLowerCase().includes("abyssal")
                  )
                  .reduce((acc, item) => {
                    const existing = acc.find((m) => m.name === item.item_name);
                    if (existing) {
                      existing.quantity += item.quantity_destroyed;
                    } else {
                      acc.push({
                        name: item.item_name,
                        quantity: item.quantity_destroyed,
                        typeId: item.item_type_id,
                      });
                    }
                    return acc;
                  }, []);

                if (abyssalModules.length === 0) {
                  return (
                    <div
                      style={{
                        color: "#6c757d",
                        fontStyle: "italic",
                        padding: "1em",
                        backgroundColor: "#f8f9fa",
                        borderRadius: "4px",
                      }}
                    >
                      No abyssal modules were destroyed.
                    </div>
                  );
                }

                return (
                  <div
                    style={{
                      backgroundColor: "#f8f9fa",
                      border: "1px solid #dee2e6",
                      borderRadius: "4px",
                      padding: "1em",
                    }}
                  >
                    {abyssalModules.map((module, index) => (
                      <div
                        key={index}
                        style={{
                          display: "flex",
                          alignItems: "center",
                          padding: "0.5em 0",
                          borderBottom:
                            index < abyssalModules.length - 1 ? "1px solid #e9ecef" : "none",
                          fontSize: "0.9em",
                        }}
                      >
                        <img
                          src={`https://images.evetech.net/types/${
                            module.typeId || 0
                          }/icon?size=32`}
                          alt={module.name}
                          style={{
                            width: "24px",
                            height: "24px",
                            marginRight: "0.5em",
                            borderRadius: "2px",
                          }}
                          onError={(e) => {
                            e.target.style.display = "none";
                          }}
                        />
                        <span style={{ flex: 1, color: "black" }}>{module.name}</span>
                        <span
                          style={{
                            fontWeight: "bold",
                            color: "black",
                            backgroundColor: "#f8d7da",
                            padding: "0.25em 0.5em",
                            borderRadius: "3px",
                            fontSize: "0.8em",
                          }}
                        >
                          {module.quantity} destroyed
                        </span>
                      </div>
                    ))}
                  </div>
                );
              })()}
            </div>

            {/* Attackers */}
            <div
              style={{
                borderRadius: "8px",
                padding: "1.5em",
                border: "1px solid #dee2e6",
                marginBottom: "1em",
              }}
            >
              <h4 style={{ margin: "0 0 1em 0", color: "white" }}>
                Attackers ({killmailData.attackers.length})
              </h4>
              <div style={{ maxHeight: "300px", overflowY: "auto" }}>
                <table style={{ width: "100%", borderCollapse: "collapse" }}>
                  <thead>
                    <tr style={{ backgroundColor: "#333" }}>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Character
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Corporation
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Alliance
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Ship
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Weapon
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Damage Done
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Final Blow
                      </th>
                      <th
                        style={{ padding: "0.5em", textAlign: "left", border: "1px solid #dee2e6" }}
                      >
                        Security Status
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {killmailData.attackers.map((attacker, index) => (
                      <tr key={index} style={{ border: "1px solid #dee2e6" }}>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.character_name || attacker.character_id || "N/A"}
                          {attacker.character_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {attacker.character_id})
                            </span>
                          )}
                        </td>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.corporation_name || attacker.corporation_id || "N/A"}
                          {attacker.corporation_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {attacker.corporation_id})
                            </span>
                          )}
                        </td>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.alliance_name || attacker.alliance_id || "N/A"}
                          {attacker.alliance_id && (
                            <span style={{ fontSize: "0.9em" }}> (ID: {attacker.alliance_id})</span>
                          )}
                        </td>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.ship_name || attacker.ship_type_id || "N/A"}
                          {attacker.ship_type_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {attacker.ship_type_id})
                            </span>
                          )}
                        </td>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.weapon_name || attacker.weapon_type_id || "N/A"}
                          {attacker.weapon_type_id && (
                            <span style={{ fontSize: "0.9em" }}>
                              {" "}
                              (ID: {attacker.weapon_type_id})
                            </span>
                          )}
                        </td>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.damage_done.toLocaleString()}
                        </td>
                        <td style={{ padding: "0.5em", border: "1px solid #dee2e6" }}>
                          <span
                            style={{
                              padding: "0.25em 0.5em",
                              borderRadius: "3px",
                              backgroundColor: attacker.final_blow ? "#d4edda" : "#f8d7da",
                              color: attacker.final_blow ? "#155724" : "#721c24",
                            }}
                          >
                            {attacker.final_blow ? "Yes" : "No"}
                          </span>
                        </td>
                        <td
                          style={{ padding: "0.5em", border: "1px solid #dee2e6", color: "white" }}
                        >
                          {attacker.security_status.toFixed(2)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Item Appraisal */}
            <div
              style={{
                borderRadius: "8px",
                padding: "1.5em",
                border: "1px solid #dee2e6",
                marginBottom: "1em",
              }}
            >
              <h4 style={{ margin: "0 0 1em 0", color: "white" }}>Droped and Destroyed Items</h4>
              {(() => {
                const droppedItems = killmailData.victim.items
                  .filter((item) => item.quantity_dropped && item.quantity_dropped > 0)
                  .map((item) => ({
                    name: item.item_name,
                    quantity: item.quantity_dropped,
                    typeId: item.item_type_id,
                  }));

                const destroyedItems = killmailData.victim.items
                  .filter((item) => item.quantity_destroyed && item.quantity_destroyed > 0)
                  .map((item) => ({
                    name: item.item_name,
                    quantity: item.quantity_destroyed,
                    typeId: item.item_type_id,
                  }));

                // Add the ship hull to destroyed items (ship was destroyed)
                const shipHull = {
                  name: killmailData.victim.ship_name,
                  quantity: 1,
                  typeId: killmailData.victim.ship_type_id,
                };
                const allDestroyedItems = [shipHull, ...destroyedItems];

                return (
                  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "2em" }}>
                    <div>
                      <h5
                        style={{
                          margin: "0 0 1em 0",
                          color: "white",
                          borderBottom: "2px solid #28a745",
                          paddingBottom: "0.5em",
                        }}
                      >
                        Dropped Items ({droppedItems.length})
                      </h5>
                      {droppedItems.length > 0 ? (
                        <div
                          style={{
                            backgroundColor: "#f8fff8",
                            border: "1px solid #d4edda",
                            borderRadius: "4px",
                            padding: "1em",
                            maxHeight: "300px",
                            overflowY: "auto",
                          }}
                        >
                          {droppedItems.map((item, index) => (
                            <div
                              key={index}
                              style={{
                                display: "flex",
                                alignItems: "center",
                                padding: "0.5em 0",
                                borderBottom:
                                  index < droppedItems.length - 1 ? "1px solid #e8f5e8" : "none",
                                fontSize: "0.9em",
                              }}
                            >
                              <img
                                src={`https://images.evetech.net/types/${
                                  item.typeId || 0
                                }/icon?size=32`}
                                style={{
                                  width: "24px",
                                  height: "24px",
                                  marginRight: "0.5em",
                                  borderRadius: "2px",
                                }}
                                alt={item.name || "Unknown Item"}
                                onError={(e) => {
                                  e.target.onerror = null;
                                  e.target.src = `https://images.evetech.net/types/${
                                    item.typeId || 0
                                  }/bp?size=32`;
                                }}
                              />
                              <span style={{ fontFamily: "monospace", color: "black" }}>
                                {item.name} {item.quantity}
                              </span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div
                          style={{
                            backgroundColor: "#f8f9fa",
                            border: "1px solid #dee2e6",
                            borderRadius: "4px",
                            padding: "1em",
                            textAlign: "center",
                            color: "#6c757d",
                          }}
                        >
                          No items were dropped
                        </div>
                      )}
                    </div>

                    <div>
                      <h5
                        style={{
                          margin: "0 0 1em 0",
                          color: "white",
                          borderBottom: "2px solid #dc3545",
                          paddingBottom: "0.5em",
                        }}
                      >
                        Destroyed Items ({allDestroyedItems.length})
                      </h5>
                      {allDestroyedItems.length > 0 ? (
                        <div
                          style={{
                            backgroundColor: "#fff8f8",
                            border: "1px solid #f8d7da",
                            borderRadius: "4px",
                            padding: "1em",
                            maxHeight: "300px",
                            overflowY: "auto",
                          }}
                        >
                          {allDestroyedItems.map((item, index) => (
                            <div
                              key={index}
                              style={{
                                display: "flex",
                                alignItems: "center",
                                padding: "0.5em 0",
                                borderBottom:
                                  index < allDestroyedItems.length - 1
                                    ? "1px solid #f5e8e8"
                                    : "none",
                                fontSize: "0.9em",
                              }}
                            >
                              <img
                                src={`https://images.evetech.net/types/${
                                  item.typeId || 0
                                }/icon?size=32`}
                                style={{
                                  width: "24px",
                                  height: "24px",
                                  marginRight: "0.5em",
                                  borderRadius: "2px",
                                }}
                                alt={item.name || "Unknown Item"}
                                onError={(e) => {
                                  e.target.onerror = null;
                                  e.target.src = `https://images.evetech.net/types/${
                                    item.typeId || 0
                                  }/bp?size=32`;
                                }}
                              />
                              <span style={{ fontFamily: "monospace", color: "black" }}>
                                {item.name} {item.quantity}
                              </span>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div
                          style={{
                            backgroundColor: "#f8f9fa",
                            border: "1px solid #dee2e6",
                            borderRadius: "4px",
                            padding: "1em",
                            textAlign: "center",
                            color: "#6c757d",
                          }}
                        >
                          No items were destroyed
                        </div>
                      )}
                    </div>
                  </div>
                );
              })()}
            </div>

            {/* Appraisal Calculation */}
            <div
              style={{
                borderRadius: "8px",
                padding: "1.5em",
                border: "1px solid #dee2e6",
                marginBottom: "1em",
              }}
            >
              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  marginBottom: "1em",
                }}
              >
                <h4 style={{ margin: "0", color: "white" }}>
                  Appraisal Calculation
                  {appraisalLoading && !appraisalResult && (
                    <span style={{ fontSize: "0.8em", color: "#6c757d", marginLeft: "0.5em" }}>
                      (Calculating...)
                    </span>
                  )}
                </h4>
                <div style={{ display: "flex", gap: "0.5em" }}>
                  <Button
                    onClick={() => handleCalculateAppraisal("everything")}
                    disabled={appraisalLoading || !killmailData}
                    style={{
                      padding: "0.5em 1em",
                      backgroundColor: "#17a2b8",
                      borderColor: "#17a2b8",
                      color: "white",
                      fontSize: "0.9em",
                    }}
                  >
                    Appraise Everything
                  </Button>
                  <Button
                    onClick={() => handleCalculateAppraisal("destroyed")}
                    disabled={appraisalLoading || !killmailData}
                    style={{
                      padding: "0.5em 1em",
                      backgroundColor: "#ffc107",
                      borderColor: "#ffc107",
                      color: "white",
                      fontSize: "0.9em",
                    }}
                  >
                    Appraise Destroyed Only
                  </Button>
                  <Button
                    onClick={() => handleCalculateAppraisal("auto")}
                    disabled={appraisalLoading || !killmailData}
                    style={{
                      padding: "0.5em 1em",
                      backgroundColor: "#6c757d",
                      borderColor: "#6c757d",
                      color: "white",
                      fontSize: "0.9em",
                    }}
                  >
                    {appraisalLoading ? "Calculating..." : "Recalculate Appraisal"}
                  </Button>
                </div>
              </div>

              {appraisalError && (
                <div
                  style={{
                    color: "#721c24",
                    backgroundColor: "#f8d7da",
                    padding: "0.75em",
                    borderRadius: "4px",
                    marginBottom: "1em",
                  }}
                >
                  Error calculating appraisal: {appraisalError}
                </div>
              )}

              {appraisalResult && (
                <div
                  style={{
                    backgroundColor: "#f8f9fa",
                    border: "1px solid #dee2e6",
                    borderRadius: "4px",
                    padding: "1em",
                  }}
                >
                  <div style={{ marginBottom: "1em" }}>
                    <div>
                      <h5 style={{ margin: "0 0 0.5em 0", color: "#495057" }}>Appraisal Results</h5>
                      <div style={{ fontSize: "1.1em", fontWeight: "bold", color: "#28a745" }}>
                        Total Sell Order Value: {formatNumber(appraisalResult.total_value)} ISK
                      </div>
                      <div style={{ fontSize: "0.9em", color: "#6c757d", marginTop: "0.5em" }}>
                        Items Appraised: {appraisalResult.item_count} | Based on Jita 4-4 market
                        prices
                      </div>
                      <div style={{ marginTop: "1em" }}>
                        <button
                          onClick={() => {
                            navigator.clipboard.writeText(appraisalResult.total_value.toString());
                          }}
                          style={{
                            padding: "0.5em 1em",
                            backgroundColor: "#28a745",
                            border: "none",
                            borderRadius: "4px",
                            color: "white",
                            fontSize: "0.9em",
                            cursor: "pointer",
                          }}
                        >
                          Pay SRP
                        </button>
                      </div>
                    </div>
                  </div>

                  <div style={{ marginTop: "1em" }}>
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        cursor: "pointer",
                        padding: "0.5em 0",
                        borderBottom: "1px solid #dee2e6",
                      }}
                      onClick={() => setShowItemBreakdown(!showItemBreakdown)}
                    >
                      <h5 style={{ margin: "0", color: "#495057", flex: 1 }}>
                        Item Breakdown ({appraisalResult.items?.length || 0} items)
                      </h5>
                      <span
                        style={{
                          fontSize: "1.2em",
                          color: "#495057",
                          transform: showItemBreakdown ? "rotate(90deg)" : "rotate(0deg)",
                          transition: "transform 0.2s ease",
                        }}
                      >
                        ▶
                      </span>
                    </div>

                    {showItemBreakdown && (
                      <div
                        style={{
                          maxHeight: "300px",
                          overflowY: "auto",
                          border: "1px solid #dee2e6",
                          borderRadius: "4px",
                          backgroundColor: "transparent",
                          marginTop: "0.5em",
                        }}
                      >
                        {appraisalResult.items?.map((item, index) => (
                          <div
                            key={index}
                            style={{
                              display: "flex",
                              justifyContent: "space-between",
                              alignItems: "center",
                              padding: "0.75em 1em",
                              borderBottom:
                                index < appraisalResult.items.length - 1
                                  ? "1px solid #eee"
                                  : "none",
                              backgroundColor: index % 2 === 0 ? "#f8f9fa" : "white",
                            }}
                          >
                            <div style={{ display: "flex", alignItems: "center", flex: 1 }}>
                              <img
                                src={`https://images.evetech.net/types/${
                                  item.itemType?.id || 0
                                }/icon?size=32`}
                                style={{
                                  width: "32px",
                                  height: "32px",
                                  marginRight: "0.75em",
                                  borderRadius: "4px",
                                }}
                                alt={item.itemType?.name || "Unknown Item"}
                                onError={(e) => {
                                  e.target.onerror = null;
                                  e.target.src = `https://images.evetech.net/types/${
                                    item.itemType?.id || 0
                                  }/bp?size=32`;
                                }}
                              />
                              <div>
                                <div style={{ fontWeight: "bold", color: "#333" }}>
                                  {item.itemType?.name || "Unknown Item"}
                                </div>
                                <div style={{ fontSize: "0.85em", color: "#495057" }}>
                                  Quantity: {item.amount} | Volume:{" "}
                                  {item.totalVolume?.toLocaleString() || "0"} m³
                                </div>
                              </div>
                            </div>
                            <div style={{ textAlign: "right", minWidth: "120px" }}>
                              <div style={{ fontWeight: "bold", color: "#28a745" }}>
                                {formatNumber(item.effectivePrices?.sellPriceTotal || 0)} ISK
                              </div>
                              <div style={{ fontSize: "0.85em", color: "#495057" }}>
                                {formatNumber(item.effectivePrices?.sellPrice || 0)} each
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        <div style={{ marginTop: "2em" }}>
          <h4>Actions</h4>
          <div style={{ display: "flex", gap: "1em" }}>
            <Button
              onClick={handleApprove}
              disabled={report.status !== "pending" || actionLoading}
              style={{
                backgroundColor: report.status === "pending" ? "#28a745" : "#6c757d",
                borderColor: report.status === "pending" ? "#28a745" : "#6c757d",
              }}
            >
              {actionLoading ? "Processing..." : "Approve"}
            </Button>
            <Button
              onClick={handleDeny}
              disabled={report.status !== "pending" || actionLoading}
              style={{
                backgroundColor: report.status === "pending" ? "#dc3545" : "#6c757d",
                borderColor: report.status === "pending" ? "#dc3545" : "#6c757d",
              }}
            >
              {actionLoading ? "Processing..." : "Reject"}
            </Button>
          </div>
        </div>
      </Box>

      {/* Approve Modal */}
      <Modal open={showApproveModal} setOpen={setShowApproveModal}>
        <Box>
          <h2 style={{ fontWeight: "bolder", marginBottom: "1em" }}>Approve SRP Report</h2>
          <div style={{ padding: "1em" }}>
            <p style={{ marginBottom: "1em" }}>Set the payout amount for this SRP report:</p>
            <div style={{ marginBottom: "1em" }}>
              <label style={{ display: "block", marginBottom: "0.5em", fontWeight: "bold" }}>
                Payout Amount (ISK):
              </label>
              <input
                type="number"
                value={payoutAmount}
                onChange={(e) => setPayoutAmount(e.target.value)}
                placeholder="Enter payout amount..."
                style={{
                  width: "100%",
                  padding: "0.5em",
                  border: "1px solid #ccc",
                  borderRadius: "4px",
                  fontSize: "1em",
                }}
                step="0.01"
                min="0"
              />
              {appraisalResult && appraisalResult.total_value && (
                <div style={{ fontSize: "0.9em", color: "#666", marginTop: "0.25em" }}>
                  Appraised value: {formatNumber(appraisalResult.total_value)} ISK
                </div>
              )}
            </div>
            <div
              style={{ display: "flex", gap: "1em", marginTop: "1em", justifyContent: "flex-end" }}
            >
              <Button
                onClick={() => {
                  setShowApproveModal(false);
                  setPayoutAmount("");
                }}
                disabled={actionLoading}
                style={{
                  backgroundColor: "#6c757d",
                  borderColor: "#6c757d",
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handleApproveConfirm}
                disabled={actionLoading || !payoutAmount.trim()}
                style={{
                  backgroundColor: "#28a745",
                  borderColor: "#28a745",
                }}
              >
                {actionLoading ? "Processing..." : "Approve Report"}
              </Button>
            </div>
          </div>
        </Box>
      </Modal>

      {/* Deny Modal */}
      {console.log("Rendering modal section, showDenyModal:", showDenyModal)}
      <Modal open={showDenyModal} setOpen={setShowDenyModal}>
        <Box>
          <h2 style={{ fontWeight: "bolder", marginBottom: "1em" }}>Deny SRP Report</h2>
          <div style={{ padding: "1em" }}>
            <p style={{ marginBottom: "1em" }}>
              Please provide a reason for denying this SRP report:
            </p>
            <textarea
              value={denyReason}
              onChange={(e) => setDenyReason(e.target.value)}
              placeholder="Enter reason for denial..."
              style={{
                width: "100%",
                minHeight: "100px",
                padding: "0.5em",
                border: "1px solid #ccc",
                borderRadius: "4px",
                resize: "vertical",
              }}
            />
            <div
              style={{ display: "flex", gap: "1em", marginTop: "1em", justifyContent: "flex-end" }}
            >
              <Button
                onClick={() => setShowDenyModal(false)}
                disabled={actionLoading}
                style={{
                  backgroundColor: "#6c757d",
                  borderColor: "#6c757d",
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handleDenyConfirm}
                disabled={actionLoading || !denyReason.trim()}
                style={{
                  backgroundColor: "#dc3545",
                  borderColor: "#dc3545",
                }}
              >
                {actionLoading ? "Processing..." : "Deny Report"}
              </Button>
            </div>
          </div>
        </Box>
      </Modal>
    </>
  );
}
