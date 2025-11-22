import React from "react";
import { useApi } from "../../api";
import { AuthContext } from "../../contexts";
import { usePageTitle } from "../../Util/title";
import { PageTitle } from "../../Components/Page";
import { Box } from "../../Components/Box";
import { Button, NavButton } from "../../Components/Form";
import { Table, TableHead, TableBody, Row, Cell, CellHead } from "../../Components/Table";
import { formatNumber } from "../../Util/number";
import { formatDatetime } from "../../Util/time";
import { useLocation } from "react-router-dom";
import { Modal } from "../../Components/Modal";
import { getCharacterCountText } from "../../Util/srpCharacterCount";
import styled from "styled-components";

const Container = styled.div`
  background-color: ${(props) => props.theme.colors.accent1};
  padding: 1em;
  border-radius: 4px;
  text-align: center;
`;

// Component for truncated description with modal functionality
function TruncatedDescription({ text, maxLength = 50 }) {
  const [showModal, setShowModal] = React.useState(false);

  if (!text || text.length <= maxLength) {
    return <span>{text || "No description"}</span>;
  }

  const displayText = text.substring(0, maxLength) + "...";

  return (
    <>
      <span
        style={{ cursor: "pointer", color: "#007bff", textDecoration: "underline" }}
        onClick={() => setShowModal(true)}
        title="Click to view full description"
      >
        {displayText}
      </span>

      <Modal open={showModal} setOpen={setShowModal}>
        <div style={{ padding: "1em" }}>
          <h3>SRP Report Description</h3>
          <div
            style={{
              marginTop: "1em",
              maxHeight: "400px",
              overflowY: "auto",
              whiteSpace: "pre-wrap",
              lineHeight: "1.5",
            }}
          >
            {text}
          </div>
          <div style={{ marginTop: "1em", textAlign: "right" }}>
            <Button onClick={() => setShowModal(false)}>Close</Button>
          </div>
        </div>
      </Modal>
    </>
  );
}

// Component for clickable status with reason modal
function StatusWithReason({ status, reason }) {
  const [showModal, setShowModal] = React.useState(false);

  const getStatusStyle = (status) => {
    switch (status) {
      case "pending":
        return { backgroundColor: "#fff3cd", color: "#856404" };
      case "approved":
        return { backgroundColor: "#d4edda", color: "#155724" };
      case "rejected":
        return { backgroundColor: "#f8d7da", color: "#721c24" };
      case "paid":
        return { backgroundColor: "#cce5ff", color: "#004085" };
      default:
        return { backgroundColor: "#6c757d", color: "#ffffff" };
    }
  };

  const statusText = status.charAt(0).toUpperCase() + status.slice(1);
  const statusStyle = getStatusStyle(status);

  if (status === "rejected" && reason) {
    return (
      <>
        <span
          style={{
            cursor: "pointer",
            padding: "0.25em 0.5em",
            borderRadius: "3px",
            ...statusStyle,
            textDecoration: "underline",
          }}
          onClick={() => setShowModal(true)}
          title="Click to view rejection reason"
        >
          {statusText}
        </span>

        <Modal open={showModal} setOpen={setShowModal}>
          <div style={{ padding: "1em" }}>
            <h3>Rejection Reason</h3>
            <div
              style={{
                marginTop: "1em",
                maxHeight: "400px",
                overflowY: "auto",
                whiteSpace: "pre-wrap",
                lineHeight: "1.5",
              }}
            >
              {reason}
            </div>
            <div style={{ marginTop: "1em", textAlign: "right" }}>
              <Button onClick={() => setShowModal(false)}>Close</Button>
            </div>
          </div>
        </Modal>
      </>
    );
  }

  return (
    <span
      style={{
        padding: "0.25em 0.5em",
        borderRadius: "3px",
        ...statusStyle,
      }}
    >
      {statusText}
    </span>
  );
}

export function SRP() {
  const authContext = React.useContext(AuthContext);
  const location = useLocation();
  const isAdminPage = location.pathname === "/srp-admin";

  const [setupData] = useApi(isAdminPage ? "/api/admin/srp/setup" : null);
  const [serviceAccount] = useApi(isAdminPage ? "/api/admin/srp/service-account" : null);
  const [allStatuses] = useApi(isAdminPage ? "/api/admin/srp/all-statuses" : null);
  const [srpReports] = useApi(isAdminPage ? "/api/admin/srp/reports" : "/api/fc/srp/reports");
  const [journalResult, setJournalResult] = React.useState(null);
  const [reconfigureResult, setReconfigureResult] = React.useState(null);
  const [focusEndResult, setFocusEndResult] = React.useState(null);
  const [testWindowResult, setTestWindowResult] = React.useState(null);
  const [isExpanded, setIsExpanded] = React.useState(false);

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

  usePageTitle(isAdminPage ? "SRP Management" : "SRP Reports");

  const handleFetchJournal = async () => {
    try {
      const response = await fetch("/api/admin/srp/journal", {
        method: "POST",
      });
      const result = await response.json();
      setJournalResult(result);
    } catch (error) {
      setJournalResult({
        success: false,
        message: "Error fetching journal: " + error.message,
        entries: null,
        wallet_id: null,
      });
    }
  };

  const handleRemoveServiceAccount = async () => {
    if (
      window.confirm(
        "Are you sure you want to remove the service account? This will clear the current configuration and allow you to set up a new character."
      )
    ) {
      try {
        const response = await fetch("/api/admin/srp/remove", {
          method: "POST",
        });
        const result = await response.json();
        setReconfigureResult(result);
        // Refresh the setup data to show the new status
        window.location.reload();
      } catch (error) {
        setReconfigureResult({
          success: false,
          message: "Error removing service account: " + error.message,
        });
      }
    }
  };

  const handleSetupServiceAccount = () => {
    if (setupData && setupData.login_url) {
      window.open(setupData.login_url, "_blank");
    }
  };

  const handleGetFocusEndTimestamp = async () => {
    try {
      console.log("Fetching focus end timestamp...");
      const response = await fetch("/api/admin/srp/focus-end-timestamp", {
        method: "GET",
      });
      console.log("Response status:", response.status);
      console.log("Response ok:", response.ok);

      if (!response.ok) {
        const errorText = await response.text();
        console.log("Error response:", errorText);
        throw new Error(`HTTP ${response.status}: ${errorText}`);
      }

      const result = await response.json();
      console.log("Focus end result:", result);
      setFocusEndResult(result);
    } catch (error) {
      console.error("Focus end timestamp error:", error);
      setFocusEndResult({
        focus_end_timestamp: null,
        formatted_date: null,
        error: "Error fetching focus end timestamp: " + error.message,
      });
    }
  };

  const handleTestCharacterWindow = async () => {
    try {
      const response = await fetch("/api/admin/srp/test-character-window", {
        method: "POST",
      });
      const result = await response.json();
      setTestWindowResult(result);
    } catch (error) {
      setTestWindowResult({
        success: false,
        message: "Error testing character window: " + error.message,
      });
    }
  };

  // Check permissions based on route
  if (isAdminPage && !authContext.access["waitlist-manage"]) {
    return <div>Access denied. You need waitlist-manage permissions.</div>;
  }

  if (!isAdminPage && !authContext.access["fleet-view"]) {
    return <div>Access denied. You need fleet-view permissions.</div>;
  }

  return (
    <>
      <PageTitle>{isAdminPage ? "SRP Management" : "SRP Reports"}</PageTitle>

      {/* Admin-only sections */}
      {isAdminPage && (
        <>
          <Box>
            <h3>Service Account Setup</h3>
            {setupData && (
              <div>
                <p>
                  <strong>Status:</strong>{" "}
                  {setupData.has_service_account ? "Configured" : "Not configured"}
                </p>
                {!setupData.has_service_account ? (
                  <Button onClick={handleSetupServiceAccount}>Configure Service Account</Button>
                ) : (
                  <Button
                    onClick={handleRemoveServiceAccount}
                    style={{ backgroundColor: "#dc3545", borderColor: "#dc3545" }}
                  >
                    Remove Service Account
                  </Button>
                )}
              </div>
            )}

            {reconfigureResult && (
              <div
                style={{
                  padding: "1em",
                  backgroundColor: reconfigureResult.success ? "#d4edda" : "#f8d7da",
                  border: `1px solid ${reconfigureResult.success ? "#c3e6cb" : "#f5c6cb"}`,
                  borderRadius: "4px",
                  marginTop: "1em",
                }}
              >
                <strong>{reconfigureResult.success ? "Success:" : "Error:"}</strong>{" "}
                {reconfigureResult.message}
              </div>
            )}

            {serviceAccount && serviceAccount.service_account && (
              <div style={{ marginTop: "1em" }}>
                <h4>Current Service Account</h4>
                <p>
                  <strong>Character:</strong> {serviceAccount.service_account.character_name}
                </p>
                <p>
                  <strong>Corporation ID:</strong> {serviceAccount.service_account.corporation_id}
                </p>
                <p>
                  <strong>Wallet ID:</strong> {serviceAccount.service_account.wallet_id}
                </p>
                <p>
                  <strong>Last Used:</strong>{" "}
                  {serviceAccount.service_account.last_used
                    ? formatDatetime(new Date(serviceAccount.service_account.last_used * 1000))
                    : "Never"}
                </p>
              </div>
            )}
          </Box>

          <Box>
            <h3>Actions</h3>
            <div style={{ display: "flex", gap: "1em", marginBottom: "1em" }}>
              <Button onClick={handleFetchJournal}>Fetch Wallet Journal</Button>
              <Button onClick={handleGetFocusEndTimestamp}>Test Focus End Timestamp</Button>
              <Button
                onClick={() => (window.location.href = "/auth/start/srp-admin")}
                variant="primary"
              >
                ESI re-auth for SRP Admin
              </Button>
              <Button onClick={handleTestCharacterWindow} variant="success">
                Test Character Window
              </Button>
            </div>

            {journalResult && (
              <div
                style={{
                  padding: "1em",
                  backgroundColor: journalResult.success ? "#d4edda" : "#f8d7da",
                  border: `1px solid ${journalResult.success ? "#c3e6cb" : "#f5c6cb"}`,
                  borderRadius: "4px",
                  marginTop: "1em",
                }}
              >
                <strong>{journalResult.success ? "Success:" : "Error:"}</strong>{" "}
                {journalResult.message}
                {journalResult.entries && journalResult.entries.length > 0 && (
                  <div>
                    <p>
                      <strong>Recent journal entries (showing first 10):</strong>
                    </p>
                    <Table fullWidth>
                      <TableHead>
                        <Row>
                          <CellHead>Date</CellHead>
                          <CellHead>Amount</CellHead>
                          <CellHead>Description</CellHead>
                          <CellHead>Type</CellHead>
                        </Row>
                      </TableHead>
                      <TableBody>
                        {journalResult.entries.slice(0, 10).map((entry) => (
                          <Row key={entry.id}>
                            <Cell>{formatDatetime(new Date(entry.date))}</Cell>
                            <Cell>{formatNumber(entry.amount)} ISK</Cell>
                            <Cell>{entry.description}</Cell>
                            <Cell>{entry.ref_type}</Cell>
                          </Row>
                        ))}
                      </TableBody>
                    </Table>
                  </div>
                )}
              </div>
            )}

            {focusEndResult && (
              <div
                style={{
                  padding: "1em",
                  backgroundColor: "#e2e3e5",
                  border: "1px solid #d6d8db",
                  borderRadius: "4px",
                  marginTop: "1em",
                }}
              >
                <strong>Focus End Timestamp Test Result:</strong>
                {focusEndResult.error ? (
                  <p style={{ color: "#721c24" }}>{focusEndResult.error}</p>
                ) : (
                  <div>
                    <p>
                      <strong>Raw Timestamp:</strong>{" "}
                      {focusEndResult.focus_end_timestamp || "No focus end recorded"}
                    </p>
                    <p>
                      <strong>Formatted Date:</strong>{" "}
                      {focusEndResult.formatted_date || "No focus end recorded"}
                    </p>
                  </div>
                )}
              </div>
            )}

            {testWindowResult && (
              <div
                style={{
                  padding: "1em",
                  backgroundColor: testWindowResult.success ? "#d4edda" : "#f8d7da",
                  border: `1px solid ${testWindowResult.success ? "#c3e6cb" : "#f5c6cb"}`,
                  borderRadius: "4px",
                  marginTop: "1em",
                }}
              >
                <strong>{testWindowResult.success ? "Success:" : "Error:"}</strong>{" "}
                {testWindowResult.message}
              </div>
            )}
          </Box>

          <Box>
            <h3>SRP Payment Status</h3>
            {allStatuses && allStatuses.statuses && allStatuses.statuses.length > 0 && (
              <div style={{ marginBottom: "1em" }}>
                <p>
                  <strong>Total characters with active SRP coverage:</strong>{" "}
                  {
                    allStatuses.statuses.filter((payment) => payment.expires_at * 1000 > Date.now())
                      .length
                  }
                </p>
              </div>
            )}
            {allStatuses && allStatuses.statuses && allStatuses.statuses.length > 0 ? (
              <div>
                <Table fullWidth>
                  <TableHead>
                    <Row>
                      <CellHead>Character</CellHead>
                      <CellHead>Payment Amount</CellHead>
                      <CellHead>Characters</CellHead>
                      <CellHead>Payment Date</CellHead>
                      <CellHead>Coverage Type</CellHead>
                      <CellHead>Coverage End</CellHead>
                      <CellHead>Status</CellHead>
                      <CellHead>Last Updated</CellHead>
                    </Row>
                  </TableHead>
                  <TableBody>
                    {allStatuses.statuses
                      .slice(0, isExpanded ? allStatuses.statuses.length : 5)
                      .map((payment) => (
                        <Row key={payment.id}>
                          <Cell>{payment.character_name}</Cell>
                          <Cell>{formatNumber(payment.payment_amount)} ISK</Cell>
                          <Cell>
                            {getCharacterCountText(payment.payment_amount, payment.coverage_type)}
                          </Cell>
                          <Cell>{formatDatetime(new Date(payment.payment_date * 1000))}</Cell>
                          <Cell>
                            <span
                              style={{
                                padding: "0.25em 0.5em",
                                borderRadius: "3px",
                                backgroundColor:
                                  payment.coverage_type === "per_focus" ? "#007bff" : "#6c757d",
                                color: "white",
                              }}
                            >
                              {payment.coverage_type === "per_focus" ? "Per Focus" : "Daily"}
                            </span>
                          </Cell>
                          <Cell>{formatDatetime(new Date(payment.expires_at * 1000))}</Cell>
                          <Cell>
                            <span
                              style={{
                                padding: "0.25em 0.5em",
                                borderRadius: "3px",
                                backgroundColor:
                                  payment.expires_at * 1000 > Date.now() ? "#d4edda" : "#f8d7da",
                                color:
                                  payment.expires_at * 1000 > Date.now() ? "#155724" : "#721c24",
                              }}
                            >
                              {payment.expires_at * 1000 > Date.now() ? "Active" : "Expired"}
                            </span>
                          </Cell>
                          <Cell>{formatDatetime(new Date(payment.created_at * 1000))}</Cell>
                        </Row>
                      ))}
                  </TableBody>
                </Table>

                {allStatuses.statuses.length > 5 && (
                  <div style={{ marginTop: "1em", textAlign: "center" }}>
                    <Button
                      onClick={() => setIsExpanded(!isExpanded)}
                      style={{
                        backgroundColor: "#007bff",
                        borderColor: "#007bff",
                        color: "white",
                      }}
                    >
                      {isExpanded
                        ? `Show Less (5 of ${allStatuses.statuses.length})`
                        : `Show More (${allStatuses.statuses.length - 5} more)`}
                    </Button>
                  </div>
                )}
              </div>
            ) : (
              <div
                style={{
                  padding: "1em",
                  backgroundColor: "#f8f9fa",
                  borderRadius: "4px",
                  textAlign: "center",
                }}
              >
                <p>
                  <strong>No SRP payment data found.</strong>
                </p>
                <p>
                  Try clicking &quot;Process Payments&quot; to scan for SRP payments in the wallet
                  journal.
                </p>
              </div>
            )}
          </Box>
        </>
      )}

      {/* SRP Reports - shown on both admin and FC pages */}
      <Box>
        <div
          style={{
            display: "flex",
            justifyContent: "flex-end",
            marginBottom: "1em",
          }}
        >
          {!isAdminPage && (
            <NavButton to="/fc/srp/submit" variant="success">
              Submit SRP Report
            </NavButton>
          )}
        </div>
        {srpReports && srpReports.reports && srpReports.reports.length > 0 ? (
          <Table fullWidth>
            <TableHead>
              <Row>
                <CellHead>Victim</CellHead>
                <CellHead>Ship Type</CellHead>
                <CellHead>Submitted By</CellHead>
                <CellHead>Submitted At</CellHead>
                <CellHead>Killmail Link</CellHead>
                <CellHead>Loot Returned</CellHead>
                <CellHead>Payout Date</CellHead>
                <CellHead>SRP Paid</CellHead>
                <CellHead>Description</CellHead>
                <CellHead>Status</CellHead>
                {isAdminPage && (
                  <>
                    <CellHead>Payout Amount</CellHead>
                    <CellHead>SRP Paid</CellHead>
                  </>
                )}
                {isAdminPage && <CellHead>Actions</CellHead>}
                <CellHead></CellHead>
              </Row>
            </TableHead>
            <TableBody>
              {srpReports.reports.map((report) => (
                <Row key={report.killmail_id}>
                  <Cell>{report.victim_character_name || "Unknown"}</Cell>
                  <Cell>{report.victim_ship_type || "Unknown"}</Cell>
                  <Cell>{report.submitted_by_name}</Cell>
                  <Cell>{formatDatetime(new Date(report.submitted_at * 1000))}</Cell>
                  <Cell>
                    <span
                      style={{
                        cursor: "pointer",
                        color: "#007bff",
                        textDecoration: "underline",
                      }}
                      onClick={() => {
                        const victimName = report.victim_character_name || "Unknown";
                        const shipType = report.victim_ship_type || "Unknown";
                        const killmailId = report.killmail_id;
                        // Extract hash from URL - it's the second-to-last part (before the trailing slash)
                        const urlParts = report.killmail_link
                          .split("/")
                          .filter((part) => part.length > 0);
                        const hash = urlParts[urlParts.length - 2] || urlParts[urlParts.length - 1];
                        const formattedText = `<url=killReport:${killmailId}:${hash}>Kill: ${victimName}'s ${shipType}</url>`;
                        navigator.clipboard.writeText(formattedText).then(() => {
                          // Show notification
                          if (window.showNotification) {
                            window.showNotification(
                              "Killmail link copied to clipboard!",
                              "success"
                            );
                          } else {
                            // Fallback to standard notification
                            alert("Killmail link copied to clipboard!");
                          }
                        });
                      }}
                      title="Click to copy killmail link to clipboard"
                    >
                      Copy Killmail Link
                    </span>
                  </Cell>
                  <Cell>
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
                  </Cell>
                  <Cell>
                    {report.payout_date
                      ? formatDatetime(new Date(report.payout_date * 1000))
                      : "N/A"}
                  </Cell>
                  <Cell>
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
                  </Cell>
                  <Cell>
                    <TruncatedDescription text={report.description} />
                  </Cell>
                  <Cell>
                    <StatusWithReason status={report.status} reason={report.reason} />
                  </Cell>
                  {isAdminPage && (
                    <>
                      <Cell>
                        {report.payout_amount ? formatNumber(report.payout_amount) + " ISK" : "N/A"}
                      </Cell>
                      <Cell>
                        <span
                          style={{
                            padding: "0.25em 0.5em",
                            borderRadius: "3px",
                            backgroundColor: getSrpPaidStatus(report.srp_paid)
                              ? "#d4edda"
                              : "#f8d7da",
                            color: getSrpPaidStatus(report.srp_paid) ? "#155724" : "#721c24",
                          }}
                        >
                          {getSrpPaidStatus(report.srp_paid) ? "Yes" : "No"}
                        </span>
                      </Cell>
                    </>
                  )}
                  {isAdminPage && (
                    <Cell style={{ verticalAlign: "middle" }}>
                      <Button
                        onClick={() =>
                          (window.location.href = `/srp-report-detail?id=${report.killmail_id}`)
                        }
                        style={{
                          padding: "0.25em 0.5em",
                          fontSize: "0.875em",
                          backgroundColor: "#007bff",
                          borderColor: "#007bff",
                          color: "white",
                          margin: "0",
                          lineHeight: "1.2",
                        }}
                      >
                        Process
                      </Button>
                    </Cell>
                  )}
                  {!isAdminPage && report.status === "pending" ? (
                    <Cell style={{ verticalAlign: "middle" }}>
                      <Button
                        onClick={() =>
                          (window.location.href = `/fc/srp/submit?edit=${report.killmail_id}`)
                        }
                        style={{
                          padding: "0.25em 0.5em",
                          fontSize: "0.875em",
                          backgroundColor: "#6c757d",
                          borderColor: "#6c757d",
                          color: "white",
                          margin: "0",
                          fontWeight: "bold",
                        }}
                      >
                        Update
                      </Button>
                    </Cell>
                  ) : !isAdminPage ? (
                    <Cell style={{ verticalAlign: "middle" }}></Cell>
                  ) : null}
                  {isAdminPage && (
                    <Cell style={{ verticalAlign: "middle" }}>
                      <Button
                        onClick={() =>
                          (window.location.href = `/fc/srp/submit?edit=${report.killmail_id}`)
                        }
                        style={{
                          padding: "0.25em 0.5em",
                          fontSize: "0.875em",
                          backgroundColor: "#6c757d",
                          borderColor: "#6c757d",
                          color: "white",
                          margin: "0",
                          fontWeight: "bold",
                        }}
                      >
                        Update
                      </Button>
                    </Cell>
                  )}
                </Row>
              ))}
            </TableBody>
          </Table>
        ) : (
          <Container>
            <p>
              <strong>No SRP reports found.</strong>
            </p>
            <p>SRP reports submitted by FCs will appear here.</p>
          </Container>
        )}
      </Box>
    </>
  );
}
