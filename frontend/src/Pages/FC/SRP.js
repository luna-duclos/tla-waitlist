import React from "react";
import { useApi } from "../../api";
import { AuthContext } from "../../contexts";
import { usePageTitle } from "../../Util/title";
import { PageTitle } from "../../Components/Page";
import { Box } from "../../Components/Box";
import { Button } from "../../Components/Form";
import { Table, TableHead, TableBody, Row, Cell, CellHead } from "../../Components/Table";
import { formatNumber } from "../../Util/number";
import { formatDatetime } from "../../Util/time";

export function SRP() {
  const authContext = React.useContext(AuthContext);
  const [setupData] = useApi("/api/admin/srp/setup");
  const [serviceAccount] = useApi("/api/admin/srp/service-account");
  const [allStatuses] = useApi("/api/admin/srp/all-statuses");
  const [journalResult, setJournalResult] = React.useState(null);
  const [reconfigureResult, setReconfigureResult] = React.useState(null);
  const [focusEndResult, setFocusEndResult] = React.useState(null);

  usePageTitle("SRP Management");



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
        wallet_id: null
      });
    }
  };

  const handleRemoveServiceAccount = async () => {
    if (window.confirm("Are you sure you want to remove the service account? This will clear the current configuration and allow you to set up a new character.")) {
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
          message: "Error removing service account: " + error.message
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
      const response = await fetch("/api/admin/srp/focus-end-timestamp", {
        method: "GET",
      });
      const result = await response.json();
      setFocusEndResult(result);
    } catch (error) {
      setFocusEndResult({
        focus_end_timestamp: null,
        formatted_date: null,
        error: "Error fetching focus end timestamp: " + error.message
      });
    }
  };

  if (!authContext.access["waitlist-manage"]) {
    return <div>Access denied. You need waitlist-manage permissions.</div>;
  }

  return (
    <>
      <PageTitle>SRP Management</PageTitle>

      <Box>
        <h3>Service Account Setup</h3>
        {setupData && (
          <div>
            <p>
              <strong>Status:</strong>{" "}
              {setupData.has_service_account ? "Configured" : "Not configured"}
            </p>
            {!setupData.has_service_account ? (
              <Button onClick={handleSetupServiceAccount}>
                Configure Service Account
              </Button>
            ) : (
              <Button onClick={handleRemoveServiceAccount} style={{ backgroundColor: "#dc3545", borderColor: "#dc3545" }}>
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
            <strong>{reconfigureResult.success ? "Success:" : "Error:"}</strong> {reconfigureResult.message}
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
          <Button onClick={handleFetchJournal}>
            Fetch Wallet Journal
          </Button>
          <Button onClick={handleGetFocusEndTimestamp}>
            Test Focus End Timestamp
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
            <strong>{journalResult.success ? "Success:" : "Error:"}</strong> {journalResult.message}
            {journalResult.entries && journalResult.entries.length > 0 && (
              <div>
                <p><strong>Recent journal entries (showing first 10):</strong></p>
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
                <p><strong>Raw Timestamp:</strong> {focusEndResult.focus_end_timestamp || "No focus end recorded"}</p>
                <p><strong>Formatted Date:</strong> {focusEndResult.formatted_date || "No focus end recorded"}</p>
              </div>
            )}
          </div>
        )}
      </Box>

      <Box>
        <h3>SRP Payment Status</h3>
        {allStatuses && allStatuses.statuses && allStatuses.statuses.length > 0 && (
          <div style={{ marginBottom: "1em" }}>
            <p><strong>Total characters with SRP coverage:</strong> {allStatuses.statuses.length}</p>
          </div>
        )}
        {allStatuses && allStatuses.statuses && allStatuses.statuses.length > 0 ? (
          <Table fullWidth>
            <TableHead>
              <Row>
                <CellHead>Character</CellHead>
                <CellHead>Payment Amount</CellHead>
                <CellHead>Payment Date</CellHead>
                <CellHead>Coverage Type</CellHead>
                <CellHead>Coverage End</CellHead>
                <CellHead>Status</CellHead>
                <CellHead>Last Updated</CellHead>
              </Row>
            </TableHead>
            <TableBody>
              {allStatuses.statuses.map((payment) => (
                <Row key={payment.id}>
                  <Cell>{payment.character_name}</Cell>
                  <Cell>{formatNumber(payment.payment_amount)} ISK</Cell>
                  <Cell>{formatDatetime(new Date(payment.payment_date * 1000))}</Cell>
                  <Cell>
                    <span
                      style={{
                        padding: "0.25em 0.5em",
                        borderRadius: "3px",
                        backgroundColor: payment.coverage_type === "per_focus" ? "#007bff" : "#6c757d",
                        color: "white",
                      }}
                    >
                      {payment.coverage_type === "per_focus" ? "Per Focus" : "Daily"}
                    </span>
                  </Cell>
                  <Cell>
                    {formatDatetime(new Date(payment.expires_at * 1000))}
                  </Cell>
                  <Cell>
                    <span
                      style={{
                        padding: "0.25em 0.5em",
                        borderRadius: "3px",
                        backgroundColor: payment.expires_at * 1000 > Date.now() ? "#d4edda" : "#f8d7da",
                        color: payment.expires_at * 1000 > Date.now() ? "#155724" : "#721c24",
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
        ) : (
          <div style={{ padding: "1em", backgroundColor: "#f8f9fa", borderRadius: "4px", textAlign: "center" }}>
            <p><strong>No SRP payment data found.</strong></p>
            <p>Try clicking &quot;Process Payments&quot; to scan for SRP payments in the wallet journal.</p>
          </div>
        )}
      </Box>
    </>
  );
}
