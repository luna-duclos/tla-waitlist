import React from "react";
import { NavLink } from "react-router-dom";
import { AuthContext, ToastContext } from "../../contexts";
import { Confirm } from "../../Components/Modal";
import { Button, Buttons, InputGroup, NavButton, Select } from "../../Components/Form";
import { Content, Title } from "../../Components/Page";
import { apiCall, errorToaster, toaster, useApi } from "../../api";
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
import { usePageTitle } from "../../Util/title";
import fleetcomp from "./fleetcomp.png";
import { PilotTags } from "../../Components/Badge";

const marauders = ["Paladin", "Kronos", "Golem", "Vargur"];
const booster = ["Eos", "Damnation", "Claymore", "Vulture"];

async function setWaitlistOpen(waitlistId, isOpen) {
  return await apiCall("/api/waitlist/set_open", {
    json: { waitlist_id: waitlistId, open: isOpen },
  });
}

async function emptyWaitlist(waitlistId) {
  return await apiCall("/api/waitlist/empty", {
    json: { waitlist_id: waitlistId },
  });
}

async function closeFleet(characterId) {
  return await apiCall("/api/fleet/close", {
    json: { character_id: characterId },
  });
}

export function Fleet() {
  const [fleetCloseModalOpen, setFleetCloseModalOpen] = React.useState(false);
  const [emptyWaitlistModalOpen, setEmptyWaitlistModalOpen] = React.useState(false);
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [fleets] = useApi("/api/fleet/status");

  React.useEffect(() => {
    // FCs will need this, request it now
    if (window.Notification && Notification.permission === "default") {
      Notification.requestPermission();
    }
  }, []);

  usePageTitle("Fleet");
  return (
    <>
      <Buttons>
        <NavButton to="/fc/fleet/register">Configure fleet</NavButton>
        <NavButton to="/auth/start/fc">ESI re-auth as FC</NavButton>
        <InputGroup>
          <Button variant="success" onClick={() => toaster(toastContext, setWaitlistOpen(1, true))}>
            Open waitlist
          </Button>
          <Button onClick={() => toaster(toastContext, setWaitlistOpen(1, false))}>
            Close waitlist
          </Button>
          <Button onClick={() => setEmptyWaitlistModalOpen(true)}>Empty waitlist</Button>
        </InputGroup>
        <Button variant="danger" onClick={() => setFleetCloseModalOpen(true)}>
          Kick everyone from fleet
        </Button>
      </Buttons>
      <Content>
        <p>
          Make sure you re-auth via ESI, then create an in-game fleet with your comp. Click the
          &quot;Configure fleet&quot; button, and select the five squads that the tool will invite
          people into. Then open the waitlist, allowing people to X up.
        </p>
        <p>
          To hand over the fleet, transfer the star (Boss role). Then the new FC should go via
          &quot;Configure fleet&quot; again, as if it was a new fleet.
        </p>
        {!fleets
          ? null
          : fleets.fleets.map((fleet) => (
              <div key={fleet.id}>
                STATUS: Fleet {fleet.id}, boss {fleet.boss.name}
              </div>
            ))}
      </Content>
      {authContext.access["fleet-history-view"] && (
        <Buttons>
          <NavButton to="/fc/fleet/history">Fleet comp history</NavButton>
        </Buttons>
      )}

      <FleetMembers />
      <Confirm
        open={fleetCloseModalOpen}
        setOpen={setFleetCloseModalOpen}
        title="Kick everyone from fleet"
        onConfirm={(evt) =>
          toaster(toastContext, closeFleet(authContext.current.id)).finally(() =>
            setFleetCloseModalOpen(false)
          )
        }
      >
        Are you sure?
      </Confirm>
      <Confirm
        open={emptyWaitlistModalOpen}
        setOpen={setEmptyWaitlistModalOpen}
        title="Empty waitlist"
        onConfirm={(evt) =>
          toaster(toastContext, emptyWaitlist(1)).finally(() => setEmptyWaitlistModalOpen(false))
        }
      >
        Are you sure?
      </Confirm>
    </>
  );
}

async function registerFleet({ fleetInfo, categoryMatches, authContext }) {
  return await apiCall("/api/fleet/register", {
    json: {
      character_id: authContext.current.id,
      assignments: categoryMatches,
      fleet_id: fleetInfo.fleet_id,
    },
  });
}

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
              {/*<NavButton
                style={{
                  marginLeft: "1em",
                  height: "unset",
                  fontSize: "10px",
                  padding: "0 0.2em",
                  lineHeight: "2em",
                }}
                to={"/pilot?character_id=" + member.id}
              >
                Information
	  </NavButton>*/}
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

function FleetMembers() {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [fleetMembers, setFleetMembers] = React.useState(null);
  const [fleetInfo, setFleetInfo] = React.useState(null);

  const characterId = authContext.current.id;
  React.useEffect(() => {
    setFleetMembers(null);
    apiCall("/api/fleet/members?character_id=" + characterId, {})
      .then(setFleetMembers)
      .catch((err) => setFleetMembers(null)); // What's error handling?
  }, [characterId]);

  React.useEffect(() => {
    setFleetInfo(null);
    errorToaster(
      toastContext,
      apiCall("/api/fleet/info?character_id=" + characterId, {}).then(setFleetInfo)
    );
  }, [characterId, toastContext]);

  if (!fleetMembers | !fleetInfo) {
    return null;
  }

  if (fleetInfo) {
    fleetInfo.wings.forEach((wing) => {
      wing.squads.sort((a, b) => a.id - b.id);
    });
  }

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
  return (
    <div style={{ display: "flex", flexWrap: "wrap" }}>
      <div>
        <Title>Members</Title>
        <Table style={{ fontSize: "12px" }}>
          <TableBody>
            {fleetInfo.wings.map((wing, wingIndex) => (
              <React.Fragment key={wing.name}>
                <Row background noAlternating>
                  <Cell style={{ fontWeight: "bold" }}>{wing.name.toUpperCase()}</Cell>
                  <Cell></Cell>
                  <Cell></Cell>
                  <Cell></Cell>
                  <Cell></Cell>
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
    </div>
  );
}

function detectSquads({ matches, categories, wings }) {
  var newMatches = { ...matches };
  var hadChanges = false;
  for (const category of categories) {
    if (!(category.id in matches)) {
      for (const wing of wings) {
        for (const squad of wing.squads) {
          if (
            squad.name.toLowerCase().includes(category.name.toLowerCase()) ||
            squad.name.toLowerCase().includes(category.id.toLowerCase())
          ) {
            newMatches[category.id] = [wing.id, squad.id];
            hadChanges = true;
          }
        }
      }
    }
  }
  if (hadChanges) {
    return newMatches;
  }
  return null;
}

export function FleetRegister() {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [fleetInfo, setFleetInfo] = React.useState(null);
  const [categories, setCategories] = React.useState(null);
  const [categoryMatches, setCategoryMatches] = React.useState({});

  const characterId = authContext.current.id;
  React.useEffect(() => {
    setFleetInfo(null);
    errorToaster(
      toastContext,
      apiCall("/api/fleet/info?character_id=" + characterId, {}).then(setFleetInfo)
    );

    setCategories(null);
    errorToaster(
      toastContext,
      apiCall("/api/categories", {}).then((data) => setCategories(data.categories))
    );
  }, [characterId, toastContext]);

  React.useEffect(() => {
    if (!categories || !fleetInfo) return;

    var newMatches = detectSquads({
      matches: categoryMatches,
      categories,
      wings: fleetInfo.wings,
    });
    if (newMatches) {
      setCategoryMatches(newMatches);
    }
  }, [fleetInfo, categories, categoryMatches, setCategoryMatches]);

  if (!fleetInfo || !categories) {
    return <em>Loading fleet information...</em>;
  }

  return (
    <>
      <div style={{ marginBottom: "1em" }}>
        Will be automatically filled if the squad names in your fleet match.{" "}
      </div>
      <div style={{ display: "flex" }}>
        <CategoryMatcher
          categories={categories}
          wings={fleetInfo.wings}
          value={categoryMatches}
          onChange={setCategoryMatches}
        />
        <img
          style={{ marginLeft: "3em" }}
          src={fleetcomp}
          alt="For auto filling, make sure the squad names match!"
        />
      </div>

      <Button
        variant="primary"
        onClick={(evt) =>
          toaster(toastContext, registerFleet({ authContext, fleetInfo, categoryMatches }))
        }
      >
        Continue
      </Button>
    </>
  );
}

function CategoryMatcher({ categories, wings, onChange, value }) {
  var flatSquads = [];
  wings.forEach((wing) => {
    wing.squads.forEach((squad) => {
      flatSquads.push({
        name: `${wing.name} - ${squad.name}`,
        id: `${wing.id},${squad.id}`,
      });
    });
  });

  var catDom = [];
  for (const category of categories) {
    var squadSelection = flatSquads.map((squad) => (
      <option key={squad.id} value={squad.id}>
        {squad.name}
      </option>
    ));
    catDom.push(
      <p key={category.id}>
        <label className="label">
          {category.name}
          <br />
        </label>
        <Select
          value={value[category.id]}
          onChange={(evt) =>
            onChange({
              ...value,
              [category.id]: evt.target.value.split(",").map((i) => parseInt(i)),
            })
          }
        >
          <option></option>
          {squadSelection}
        </Select>
      </p>
    );
  }
  return <Content>{catDom}</Content>;
}
