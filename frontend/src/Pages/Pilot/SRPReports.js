import React from "react";
import { useApi } from "../../api";
import { Table, TableHead, TableBody, Row, Cell, CellHead } from "../../Components/Table";




// Component for clickable status with reason modal
function StatusWithReason({ status, reason }) {
  const [showModal, setShowModal] = React.useState(false);

  const statusStyle = {
    pending: { backgroundColor: "#fff3cd", color: "#856404" },
    approved: { backgroundColor: "#d4edda", color: "#155724" },
    rejected: { backgroundColor: "#f8d7da", color: "#721c24" }
  };

  const statusText = status.charAt(0).toUpperCase() + status.slice(1);

  if (status === "rejected" && reason) {
    return (
      <>
        <span
          style={{
            cursor: "pointer",
            padding: "0.25em 0.5em",
            borderRadius: "3px",
            ...statusStyle[status],
            textDecoration: "underline"
          }}
          onClick={() => setShowModal(true)}
          title="Click to view rejection reason"
        >
          {statusText}
        </span>
        
        {showModal && (
          <div
            style={{
              position: "fixed",
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              backgroundColor: "rgba(0, 0, 0, 0.5)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              zIndex: 1000
            }}
            onClick={() => setShowModal(false)}
          >
            <div
              style={{
                backgroundColor: "white",
                padding: "2em",
                borderRadius: "8px",
                maxWidth: "80%",
                maxHeight: "80%",
                overflow: "auto"
              }}
              onClick={(e) => e.stopPropagation()}
            >
              <h3>Rejection Reason</h3>
              <div style={{ 
                marginTop: "1em", 
                maxHeight: "400px", 
                overflowY: "auto",
                whiteSpace: "pre-wrap",
                lineHeight: "1.5"
              }}>
                {reason}
              </div>
              <div style={{ marginTop: "1em", textAlign: "right" }}>
                <button
                  onClick={() => setShowModal(false)}
                  style={{
                    padding: "0.5em 1em",
                    backgroundColor: "#007bff",
                    color: "white",
                    border: "none",
                    borderRadius: "4px",
                    cursor: "pointer"
                  }}
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        )}
      </>
    );
  }
  
  return (
    <span
      style={{
        padding: "0.25em 0.5em",
        borderRadius: "3px",
        ...statusStyle[status]
      }}
    >
      {statusText}
    </span>
  );
}

export function SRPReports({ characterId }) {
  const [srpReports] = useApi(`/api/pilot/srp-reports/${characterId}`);



  if (!srpReports || !srpReports.reports || srpReports.reports.length === 0) {
    return (
      <div>
        <h3>SRP Reports</h3>
        <p>No SRP reports found for this pilot or their alts.</p>
      </div>
    );
  }

  return (
    <div>
      <h3>SRP Reports ({srpReports.reports.length})</h3>
      <p>Showing SRP reports where this pilot or their alts were the victim.</p>
      
      <Table fullWidth>
        <TableHead>
          <Row>
            <CellHead>Victim</CellHead>
            <CellHead>Status</CellHead>
          </Row>
        </TableHead>
        <TableBody>
          {srpReports.reports.map((report) => (
            <Row key={report.killmail_id}>
              <Cell>{report.victim_character_name || "Unknown"}</Cell>
              <Cell>
                <StatusWithReason status={report.status} reason={report.reason} />
              </Cell>
            </Row>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
