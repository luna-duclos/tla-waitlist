import React from "react";
import { AuthContext, ToastContext } from "../../contexts";
import { Confirm } from "../../Components/Modal";
import { Button, Buttons, InputGroup, NavButton, Select } from "../../Components/Form";
import { Content } from "../../Components/Page";
import { apiCall, errorToaster, toaster, useApi } from "../../api";
import { usePageTitle } from "../../Util/title";
import fleetcomp from "./fleetcomp.png";
import { FleetMembers } from "./FleetStats";

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
        <InputGroup>
          <NavButton to="/fc/fleet/register">Configure fleet</NavButton>
        </InputGroup>
        <InputGroup>
          <NavButton to="/auth/start/fc">ESI re-auth as FC</NavButton>
        </InputGroup>
        <InputGroup>
          <Button variant="success" onClick={() => toaster(toastContext, setWaitlistOpen(1, true))}>
            Open waitlist
          </Button>
          <Button onClick={() => toaster(toastContext, setWaitlistOpen(1, false))}>
            Close waitlist
          </Button>
          <Button onClick={() => setEmptyWaitlistModalOpen(true)}>Empty waitlist</Button>
        </InputGroup>
        <InputGroup>
          <Button variant="danger" onClick={() => setFleetCloseModalOpen(true)}>
            Kick everyone from fleet
          </Button>
        </InputGroup>
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
          <InputGroup>
            <NavButton to="/fc/fleet/history">Fleet comp history</NavButton>
          </InputGroup>
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
  // First, identify which wing is on-grid (has numbered squads) vs off-grid
  const wingHasNumberedSquads = (wing) => {
    return wing.squads.some((squad) => /\d/.test(squad.name));
  };

  // Sort wings: on-grid first (has numbered squads), then off-grid
  const sortedWings = [...wings].sort((a, b) => {
    const aHasNumbers = wingHasNumberedSquads(a);
    const bHasNumbers = wingHasNumberedSquads(b);
    
    if (aHasNumbers && !bHasNumbers) return -1; // on-grid first
    if (!aHasNumbers && bHasNumbers) return 1;  // off-grid second
    return 0; // same type, keep original order
  });

  var flatSquads = [];
  sortedWings.forEach((wing) => {
    // Within each wing, sort squads: non-numbered first, then numbered
    const sortedSquads = [...wing.squads].sort((a, b) => {
      const aHasNumber = /\d/.test(a.name);
      const bHasNumber = /\d/.test(b.name);
      
      // Non-numbered squads come first
      if (aHasNumber && !bHasNumber) return 1;
      if (!aHasNumber && bHasNumber) return -1;
      
      // If both have numbers, extract and compare numerically
      if (aHasNumber && bHasNumber) {
        const aMatch = a.name.match(/\d+/);
        const bMatch = b.name.match(/\d+/);
        if (aMatch && bMatch) {
          const aNum = parseInt(aMatch[0], 10);
          const bNum = parseInt(bMatch[0], 10);
          if (aNum !== bNum) {
            return aNum - bNum;
          }
        }
      }
      
      // Within each group, sort alphabetically
      return a.name.localeCompare(b.name);
    });

    sortedSquads.forEach((squad) => {
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
