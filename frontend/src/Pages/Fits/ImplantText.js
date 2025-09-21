//import { InfoNote } from "../../Components/NoteBox";
import { Highlight } from "../../Components/Form";
import { Copyable } from "../../Components/Copy";
import { ToastContext } from "../../contexts";
import React from "react";
import {
  CellHead,
  SmallCellHead,
  Table,
  TableHead,
  Row,
  TableBody,
  Cell,
} from "../../Components/Table";

export function ImplantTable({ type }) {
  const toastContext = React.useContext(ToastContext);
  var implants;
  if (type === "Hybrid") {
    implants = [
      "High-grade Amulet Alpha",
      "High-grade Amulet Beta",
      "High-grade Amulet Gamma",
      "High-grade Amulet Delta",
      "High-grade Amulet Epsilon",
    ];
  } else {
    implants = [
      "High-grade Ascendancy Alpha",
      "High-grade Ascendancy Beta",
      "High-grade Ascendancy Gamma",
      "High-grade Ascendancy Delta",
      "High-grade Ascendancy Epsilon",
    ];
  }
  return (
    <>
      {/*<InfoNote>
        {type === "Hybrid"
          ? "Hybrid tagged fits require at least Amulet 1 - 5 to be flown."
: "Required for Elite badge on non implant specific ships."}
      </InfoNote>*/}

      <Table style={{ width: "100%" }}>
        <TableHead>
          <Row>
            <SmallCellHead></SmallCellHead>
            <CellHead>DEFAULT</CellHead>
            <CellHead>ALTERNATIVE</CellHead>
          </Row>
        </TableHead>
        <TableBody>
          {implants.map((currentValue, index) => (
            <ImplantAllRow
              key={index}
              toast={toastContext}
              slot={index + 1}
              implant={currentValue}
            />
          ))}

          <Row>
            <Cell>
              <b>Slot 6</b>
            </Cell>
            <Cell>
              <CopyImplantText toast={toastContext} item={"WS-618"} /> +18% warp speed
            </Cell>
            {type === "Hybrid" ? (
              <Cell></Cell>
            ) : (
              <Cell>
                <CopyImplantText toast={toastContext} item={"High-grade Ascendancy Omega"} /> +21% warp speed
              </Cell>
            )}
          </Row>

          <HardWires toastContext={toastContext} />
        </TableBody>
      </Table>
    </>
  );
}

function ImplantAllRow({ toast, slot, implant }) {
  return (
    <Row>
      <Cell>
        <b>Slot {slot}</b>
      </Cell>
      <Cell>
        <CopyImplantText toast={toast} item={implant} />
      </Cell>
      <Cell></Cell>
    </Row>
  );
}

function CopyImplantText({ toast, item }) {
  return (
    <Highlight
      onClick={(evt) => {
        Copyable(toast, item);
      }}
    >
      {item}
    </Highlight>
  );
}

function HardWires({ toastContext }) {
  return (
    <>
      <Row>
        <Cell>
          <b>Slot 7</b>
        </Cell>
        <Cell>
          <b>Kronos/Paladin/Vindicator:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"Ogdin's Eye"} /> +6% Tracking
          <br />
          <b>Vargur:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"TA-706"} /> +6% Falloff
          <br />
        </Cell>

        <Cell>
          <CopyImplantText toast={toastContext} item={"MR-706"} /> +6% Tracking
          <br />
          <b>Logi:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"RA-706"} /> -6% Cap use for remote reps
        </Cell>
      </Row>
      <Row>
        <Cell>
          <b>Slot 8</b>
        </Cell>
        <Cell>
          <CopyImplantText toast={toastContext} item={"EM-806"} /> +6% Capacitor
        </Cell>

        <Cell>
          <CopyImplantText toast={toastContext} item={"Zor's Custom Navigation Hyper-Link"} /> +5% MWD/AB Speed
          <br />
          <b>Vindicator:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"MR-807"} /> +7% Web range
        </Cell>
      </Row>
      <Row>
        <Cell>
          <b>Slot 9</b>
        </Cell>
        <Cell>
          <CopyImplantText toast={toastContext} item={"RF-906"} /> +6% rate of fire for all turrets
        </Cell>

        <Cell>
          <CopyImplantText toast={toastContext} item={"SS-906"} /> +6% damage for all turrets
          <br />
          <CopyImplantText toast={toastContext} item={"Pashan's Turret Customization Mindlink"} /> +7% rate of fire for all turrets
        </Cell>
      </Row>
      <Row>
        <Cell>
          <b>Slot 10</b>
        </Cell>
        <Cell>
          <b>Kronos/Vindicator:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"LH-1006"} /> +6% hybrid weapon damage
          <br />
          <b>Paladin:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"LE-1006"} /> +6% energy weapon damage
          <br />
          <b>Vargur:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"LP-1006"} /> +6% projectile weapon damage
        </Cell>
        <Cell>
          <b>Paladin:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"Pashan's Turret Handling Mindlink"} /> +7% energy weapon damage
          <br />
          <b>Logi:</b>
          <br />
          <CopyImplantText toast={toastContext} item={"HG-1006"} /> +6% Armor hitpoints
          <br />
          <CopyImplantText toast={toastContext} item={"HG-1008"} /> +8% Armor hitpoints
          <br />
        </Cell>
      </Row>
    </>
  );
}
