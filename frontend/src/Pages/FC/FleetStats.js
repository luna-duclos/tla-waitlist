import React from "react";
import { NavLink } from "react-router-dom";
import { AuthContext, ToastContext } from "../../contexts";
import { InputGroup } from "../../Components/Form";
import { Title } from "../../Components/Page";
import { apiCall, useApi } from "../../api";
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

function GetNamesByCategory({ filteredMembers, category, warnActive }) {
  const nonBoosterShips = filteredMembers
    .map((object) => object.ship.name)
    .filter((name) => !booster.includes(name));
  return (
    <>
      {filteredMembers.map((member) => (
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
  const filteredMembers = React.useMemo(() => {
    return members
      .filter((member) => member.category === squadname)
      .sort((a, b) => {
        if (a.role === "squad_commander") return -1;
        if (b.role === "squad_commander") return 1;
        return 0;
      });
  }, [members, squadname]);
  return (
    <React.Fragment key={squadname}>
      <Row key={squadname} style={{ paddingLeft: "20px" }} noAlternating>
        <CellTight></CellTight>
        <CellTight>
          {squadname}
          {filteredMembers.length > 0 && ` (${filteredMembers.length})`}
        </CellTight>
        <CellTight></CellTight>
        <CellTight></CellTight>
        <CellTight></CellTight>
      </Row>
      <GetNamesByCategory
        filteredMembers={filteredMembers}
        category={squadname}
        warnActive={warnActive}
      />
    </React.Fragment>
  );
}

function FleetComposition({ cats, summary }) {
  return (
    <div style={{ marginLeft: "4em" }}>
      <Title>Fleet composition</Title>
      <InputGroup>
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

export default function FleetMembers({ fleetcomp = true }) {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [fleetMembers, setFleetMembers] = React.useState(null);
  const [fleetInfo, setFleetInfo] = React.useState(null);
  const characterId = authContext.current.id;

  React.useEffect(() => {
    setFleetMembers(null);
    apiCall("/api/fleet/members?character_id=" + characterId, {})
      .then(setFleetMembers)
      .catch((err) => {
        setFleetMembers(null);
      });
    const intervalId = setInterval(() => {
      apiCall("/api/fleet/members?character_id=" + characterId, {})
        .then(setFleetMembers)
        .catch((err) => {
          setFleetMembers(null);
        });
    }, 10000);
    return () => clearInterval(intervalId);
  }, [characterId]);

  React.useEffect(() => {
    apiCall("/api/fleet/info?character_id=" + characterId, {})
      .then(setFleetInfo)
      .catch((err) => {
        setFleetInfo(null);
      });
  }, [characterId, toastContext]);

  if (!fleetMembers || !fleetInfo) {
    return null;
  }

  if (fleetInfo) {
    fleetInfo.wings.forEach((wing) => {
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
    if (fleetMembers) {
      fleetMembers.members.forEach((member) => {
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
            {fleetInfo.wings.map((wing, wingIndex) => (
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
                    {wing.name.toUpperCase()}
                  </Cell>
                  <Cell style={{ width: "80px" }}></Cell>
                  <Cell></Cell>
                  <Cell style={{ width: "50px" }}></Cell>
                  <Cell style={{ width: "75px" }}></Cell>
                </Row>

                {wing.squads.map((squad, squadIndex) => (
                  <Squad
                    key={squad.name}
                    squadname={squad.name}
                    members={fleetMembers.members}
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
