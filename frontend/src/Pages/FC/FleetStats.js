import React from "react";
import { NavLink } from "react-router-dom";
import { AuthContext, ToastContext } from "../../contexts";
import { InputGroup } from "../../Components/Form";
import { Title } from "../../Components/Page";
import { apiCall, useApi } from "../../api";
import { addToast } from "../../Components/Toast";
import {
  CellTight,
  Cell,
  CellHead,
  Row,
  Table,
  TableBody,
  TableHead,
  CellWithLine,
} from "../../Components/Table";
import { BorderedBox } from "../../Components/NoteBox";
import _ from "lodash";

import { PilotTags } from "../../Components/Badge";
const marauders = ["Paladin", "Kronos", "Golem", "Vargur"];
const booster = ["Eos", "Damnation", "Claymore", "Vulture"];

function PilotTagsFromId({ characterId }) {
  const [basicInfo] = useApi(`/api/pilot/info?character_id=${characterId}`);

  return <PilotTags tags={basicInfo && basicInfo.tags} height={"14px"} />;
}

function SquadMembers({ members, warnActive }) {
  const nonBoosterShips = members
    .map((object) => object.ship.name)
    .filter((name) => !booster.includes(name));
  return (
    <>
      {members.map((member) => (
        <Row key={member.id} noAlternating>
          <CellTight></CellTight>
          <CellWithLine warn={nonBoosterShips.length > 2 && warnActive && "yellow"}>
            {member.role === "squad_commander" && "★"}
          </CellWithLine>
          <CellTight>
            <NavLink to={"/pilot?character_id=" + member.id}>{member.name} </NavLink>
          </CellTight>
          <CellTight>{member.ship.name}</CellTight>
          <CellTight>
            <div style={{ display: "flex", justifyContent: "space-between" }}>
              <PilotTagsFromId characterId={member.id} />
            </div>
          </CellTight>
        </Row>
      ))}
    </>
  );
}

function Squad({ members, squadname, warnActive }) {
  members.sort((a, b) => {
    if (a.role === "squad_commander") {
      return -1;
    } else if (b.role === "squad_commander") {
      return 1;
    }
    return 0;
  });
  return (
    <React.Fragment key={squadname}>
      <Row key={squadname} style={{ paddingLeft: "20px" }} noAlternating>
        <CellTight></CellTight>
        <CellTight>
          {squadname}
          {members.length > 0 && ` (${members.length})`}
        </CellTight>
        <CellTight></CellTight>
        <CellTight></CellTight>
        <CellTight></CellTight>
      </Row>
      <SquadMembers members={members} category={squadname} warnActive={warnActive} />
    </React.Fragment>
  );
}

function FleetComposition({ cats, summary }) {
  return (
    <div style={{ marginLeft: "4em" }}>
      <Title>Fleet composition</Title>
      <InputGroup style={{ marginBottom: "1em" }}>
        <BorderedBox>Marauders: {cats["Marauder"]} </BorderedBox>
        <BorderedBox>
          Logistics (N/L): {cats["Logiarmor"]}/{cats["Logishield"]}{" "}
        </BorderedBox>
        <BorderedBox>Vindicators: {cats["Vindicator"]} </BorderedBox>
        <BorderedBox>Boosters: {cats["Booster"]} </BorderedBox>
      </InputGroup>
      <Table>
        <TableHead>
          <Row>
            <CellHead>Ship</CellHead>
            <CellHead>#</CellHead>
          </Row>
        </TableHead>
        <TableBody>
          {_.sortBy(_.entries(summary), [1]).map(([shipName, count]) => (
            <Row key={shipName}>
              <Cell>{shipName}</Cell>
              <Cell>{count}</Cell>
            </Row>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

function getWingMembersCount(wing) {
  let totalMembers = 0;
  if (wing.member) totalMembers += 1;
  for (let squad of wing.squads) {
    totalMembers += squad.members.length;
  }
  return totalMembers;
}

export default function FleetMembers({ fleetcomp = true, handleChangeStat = null }) {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [fleetCompositionInfo, setFleetCompositionInfo] = React.useState(null);
  const [errorCount, setErrorCount] = React.useState(0);
  const characterId = authContext.current.id;

  React.useEffect(() => {
    apiCall("/api/fleet/fleetcomp?character_id=" + characterId, {})
      .then(setFleetCompositionInfo)
      .catch((err) => {
        setFleetCompositionInfo(null);
        if (!fleetcomp) handleChangeStat();
      });
    if (!fleetcomp) {
      const intervalId = setInterval(() => {
        if (errorCount >= 4) {
          addToast(toastContext, {
            title: "Error",
            message: "Consecutive Error Limit Exceeded, shutting down fleetmembers",
            variant: "danger",
          });
          handleChangeStat();
          return;
        }
        apiCall("/api/fleet/fleetcomp?character_id=" + characterId, {})
          .then((fleetCompositionInfo) => {
            setFleetCompositionInfo(fleetCompositionInfo);
            setErrorCount(0);
          })
          .catch((err) => {
            setErrorCount(errorCount + 1);
            if (err.toLowerCase().includes("fleet".toLowerCase())) handleChangeStat();
          });
      }, 15000);
      return () => clearInterval(intervalId);
    }
  }, [characterId, errorCount, fleetcomp, handleChangeStat, toastContext]);

  if (!fleetCompositionInfo) {
    return null;
  }

  if (fleetCompositionInfo) {
    fleetCompositionInfo.wings.forEach((wing) => {
      wing.squads.sort((a, b) => a.id - b.id);
    });
  }

  if (fleetcomp) {
    var cats = {
      Marauder: 0,
      Logiarmor: 0,
      Logishield: 0,
      Vindicator: 0,
      Booster: 0,
    };

    var summary = {};
    if (fleetCompositionInfo) {
      fleetCompositionInfo.members.forEach((member) => {
        if (!summary[member.ship.name]) summary[member.ship.name] = 0;
        summary[member.ship.name]++;
        if (booster.includes(member.ship.name)) cats["Booster"]++;
        if (marauders.includes(member.ship.name)) cats["Marauder"]++;
        if ("Vindicator" === member.ship.name) cats["Vindicator"]++;
        if ("Loki" === member.ship.name) cats["Logishield"]++;
        if ("Nestor" === member.ship.name) cats["Logiarmor"]++;
      });
    }
  }

  return (
    <div style={{ display: "flex", flexWrap: "wrap" }}>
      <div>
        {fleetcomp && <Title>Members</Title>}
        <Table style={{ fontSize: "12px" }} fullWidth={fleetcomp ? undefined : true}>
          <TableBody>
            {fleetCompositionInfo.wings.map((wing, wingIndex) => (
              <React.Fragment key={wing.name}>
                <Row background noAlternating>
                  <Cell
                    style={{
                      fontWeight: "bold",
                      width: "40px",
                      overflow: "visible",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {wing.name.toUpperCase()} ({getWingMembersCount(wing)})
                  </Cell>
                  <Cell style={{ width: "80px" }}></Cell>
                  <Cell></Cell>
                  <Cell style={{ width: "70px" }}></Cell>
                  <Cell style={{ width: "75px" }}></Cell>
                </Row>

                {wing.squads.map((squad, squadIndex) => (
                  <Squad
                    key={squad.name}
                    squadname={squad.name}
                    members={squad.members}
                    warnActive={squadIndex > 2 && wingIndex === 0}
                  />
                ))}
              </React.Fragment>
            ))}
          </TableBody>
        </Table>
      </div>

      {fleetcomp && <FleetComposition cats={cats} summary={summary} />}
    </div>
  );
}
