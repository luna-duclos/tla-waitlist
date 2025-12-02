import React from "react";
import { NavLink } from "react-router-dom";
import { ThemeContext } from "styled-components";
import { AuthContext, ToastContext } from "../../contexts";
import { InputGroup } from "../../Components/Form";
import { Title } from "../../Components/Page";
import { apiCall, useApi } from "../../api";
import { addToast } from "../../Components/Toast";
import { CategoryHeadingDOM, ColumnWaitlistDOM } from "../Waitlist/displaymodes";
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
import { Confirm } from "../../Components/Modal";
import _ from "lodash";

import { PilotTags } from "../../Components/Badge";
import soundFile from "../../Components/Event/notification-error-427345.mp3";
const marauders = ["Paladin", "Kronos", "Golem", "Vargur"];
const booster = ["Eos", "Damnation", "Claymore", "Vulture", "Sleipnir", "Astarte", "Absolution", "Nighthawk", ];
const roleOrder = ["DDD", "MS", "LR", "MTAC", "PS"];

// Hull validation mapping (same as backend)
const getAllowedHullsForRole = (role) => {
  const roleUpper = role.toUpperCase();
  switch (roleUpper) {
    case "DDD":
      return ["vindicator"];
    case "MS":
      return ["damnation", "eos", "claymore"];
    case "LR":
      return ["kronos"];
    case "MTAC":
      return ["paladin", "vargur", "occator", "mastodon", "bustard", "impel"];
    case "PS":
      return ["paladin", "vargur"];
    default:
      return [];
  }
};

function PilotTagsFromId({ characterId }) {
  const [basicInfo] = useApi(`/api/pilot/info?character_id=${characterId}`);

  return <PilotTags tags={basicInfo && basicInfo.tags} height={"14px"} />;
}

// Context menu component for role selection
function RoleContextMenu({ open, setOpen, position, character, onRoleSelect, authContext }) {
  const theme = React.useContext(ThemeContext);
  
  // Hooks must be called unconditionally - move before early return
  const menuRef = React.useRef(null);
  const [adjustedPosition, setAdjustedPosition] = React.useState(position);

  React.useEffect(() => {
    if (!open) {
      setAdjustedPosition(position);
      return;
    }
    
    const adjustPosition = () => {
      if (!menuRef.current) return;
      
      const menu = menuRef.current;
      const menuHeight = menu.offsetHeight || 200;
      const menuWidth = menu.offsetWidth || 120;
      const windowHeight = window.innerHeight;
      const windowWidth = window.innerWidth;
      
      let adjustedX = position.x;
      let adjustedY = position.y;
      
      if (position.y + menuHeight > windowHeight) {
        adjustedY = Math.max(10, position.y - menuHeight);
      }
      
      if (position.x + menuWidth > windowWidth) {
        adjustedX = Math.max(10, windowWidth - menuWidth - 10);
      }
      
      if (adjustedX < 0) {
        adjustedX = 10;
      }
      
      if (adjustedY < 0) {
        adjustedY = 10;
      }
      
      setAdjustedPosition({ x: adjustedX, y: adjustedY });
    };
    
    adjustPosition();
    
    const timeoutId = setTimeout(adjustPosition, 0);
    
    return () => clearTimeout(timeoutId);
  }, [open, position]);

  const handleRoleClick = (role) => {
    onRoleSelect(role);
    setOpen(false);
  };

  const hoverBgColor = theme.colors.accent1 || "#f0f0f0";
  const borderColor = theme.colors.accent2 || "#ccc";
  const itemBorderColor = theme.colors.accent2 || "#eee";
  
  const isRoleSuitable = (role) => {
    if (!character?.ship?.name) return false;
    const allowedHulls = getAllowedHullsForRole(role);
    const shipNameLower = character.ship.name.toLowerCase();
    return allowedHulls.includes(shipNameLower);
  };

  // Early return after all hooks
  if (!open) return null;

  return (
    <div
      ref={menuRef}
      style={{
        position: "fixed",
        left: `${adjustedPosition.x}px`,
        top: `${adjustedPosition.y}px`,
        backgroundColor: theme.colors.background,
        color: theme.colors.text,
        border: `1px solid ${borderColor}`,
        borderRadius: "4px",
        boxShadow: `0 2px 8px ${theme.colors.shadow}`,
        zIndex: 1000,
        minWidth: "120px",
      }}
      onClick={(e) => e.stopPropagation()}
    >
      {roleOrder.map((role) => {
        const suitable = isRoleSuitable(role);
        
        return (
          <div
            key={role}
            style={{
              padding: "8px 16px",
              cursor: "pointer",
              borderBottom: `1px solid ${itemBorderColor}`,
              color: theme.colors.text,
            }}
            onClick={() => handleRoleClick(role)}
            onMouseEnter={(e) => (e.target.style.backgroundColor = hoverBgColor)}
            onMouseLeave={(e) => (e.target.style.backgroundColor = theme.colors.background)}
          >
            {suitable && <span style={{ color: theme.colors.highlight?.text || theme.colors.primary?.color || "#fc9936", marginRight: "6px" }}>★</span>}
            {role}
          </div>
        );
      })}
    </div>
  );
}

function SquadMembers({ members, warnActive, onCharacterRightClick }) {
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
            <NavLink
              to={"/pilot?character_id=" + member.id}
              onContextMenu={(e) => {
                e.preventDefault();
                if (onCharacterRightClick) {
                  onCharacterRightClick(e, member);
                }
              }}
            >
              {member.name}{" "}
            </NavLink>
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

function Squad({ members, squadname, warnActive, onCharacterRightClick }) {
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
      <SquadMembers members={members} category={squadname} warnActive={warnActive} onCharacterRightClick={onCharacterRightClick} />
    </React.Fragment>
  );
}

function FleetDscan({ characterId, memberlist, showFull = true }) {
  var cats = {
    Marauder: 0,
    Logiarmor: 0,
    Logishield: 0,
    Vindicator: 0,
    Booster: 0,
  };
  var summary = {};
  if (memberlist) {
    memberlist.forEach((member) => {
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
    <div style={showFull ? {} : { display: "flex", justifyContent: "center" }}>
      {showFull && <Title>Fleet Summary</Title>}
      <InputGroup style={{ marginBottom: "1em" }}>
        <BorderedBox>Marauders: {cats["Marauder"]} </BorderedBox>
        <BorderedBox>
          Logi (N/L): {cats["Logiarmor"]}/{cats["Logishield"]}{" "}
        </BorderedBox>
        <BorderedBox>Vindicators: {cats["Vindicator"]} </BorderedBox>
        <BorderedBox>Boosters: {cats["Booster"]} </BorderedBox>
      </InputGroup>
      {showFull && (
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
      )}
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

function getFleetMembers(fleet) {
  let members = [];
  if (fleet.member) members.push(fleet.member);
  fleet.wings.forEach((wing) => {
    if (wing.member) members.push(wing.member);
    wing.squads.forEach((squad) => {
      squad.members.forEach((member) => {
        members.push(member);
      });
    });
  });

  return members;
}

function RoleAssignments({ roleAssignments }) {
  const prevRoleAssignmentsRef = React.useRef(null);
  const audioRef = React.useRef(null);

  React.useEffect(() => {
    if (!audioRef.current) {
      audioRef.current = new Audio(soundFile);
      audioRef.current.volume = 0.5;
      audioRef.current.preload = "auto";
    }
  }, []);

  const roleAssignmentsSerialized = React.useMemo(() => {
    return roleAssignments ? JSON.stringify(roleAssignments) : null;
  }, [roleAssignments]);

  React.useEffect(() => {
    if (!roleAssignments) {
      return;
    }
    
    if (!prevRoleAssignmentsRef.current) {
      prevRoleAssignmentsRef.current = JSON.parse(JSON.stringify(roleAssignments));
      return;
    }

    if (roleAssignmentsSerialized === JSON.stringify(prevRoleAssignmentsRef.current)) {
      return;
    }

    let hasDeparture = false;
    const departedCharacters = [];
    
    for (const role of roleOrder) {
      const prevChars = prevRoleAssignmentsRef.current[role] || [];
      const currentChars = roleAssignments[role] || [];
      
      const currentCharsMap = new Map();
      currentChars.forEach(char => {
        currentCharsMap.set(char.name.toLowerCase(), char);
      });
      
      for (const prevChar of prevChars) {
        const nameLower = prevChar.name.toLowerCase();
        const currentChar = currentCharsMap.get(nameLower);
        
        if (prevChar.in_fleet === true) {
          if (!currentChar) {
            hasDeparture = true;
            departedCharacters.push(`${nameLower} (${role})`);
          } else if (currentChar.in_fleet === false) {
            hasDeparture = true;
            departedCharacters.push(`${nameLower} (${role})`);
          }
        }
      }
    }

    if (hasDeparture && audioRef.current) {
      const storageKey = "EventNotifierSettings";
      let enableSound = false;
      if (window.localStorage && window.localStorage.getItem(storageKey)) {
        try {
          const settings = JSON.parse(window.localStorage.getItem(storageKey));
          enableSound = settings.enableSound === true;
        } catch (err) {
          console.log("Could not parse EventNotifierSettings:", err);
        }
      }
      
      if (enableSound) {
        audioRef.current.currentTime = 0;
        const playPromise = audioRef.current.play();
        if (playPromise !== undefined) {
          playPromise.catch(err => {
            console.log("Could not play sound:", err);
          });
        }
      }
    }

    prevRoleAssignmentsRef.current = JSON.parse(JSON.stringify(roleAssignments));
  }, [roleAssignments, roleAssignmentsSerialized]);

  return (
    <div style={{ marginBottom: "1em" }}>
      <Title>Fleet Roles</Title>
      <InputGroup>
        {roleOrder.map((role) => {
          const characters = roleAssignments && roleAssignments[role];
          if (!characters || characters.length === 0) {
            return (
              <BorderedBox key={role}>
                {role}: <span style={{ color: "red" }}>Missing</span>
              </BorderedBox>
            );
          }
          return (
            <BorderedBox key={role}>
              {role}:{" "}
              {characters.map((char, index) => {
                let color = "inherit";
                if (!char.in_fleet) {
                  color = "red"; // Not in fleet
                } else if (char.correct_hull === false) {
                  color = "yellow"; // In fleet but wrong hull
                }
                return (
                  <span key={index}>
                    {index > 0 && ", "}
                    {char.correct_hull === false && char.hull_id && (
                      <img
                        src={`https://images.evetech.net/types/${char.hull_id}/icon?size=32`}
                        alt=""
                        style={{ height: "16px", verticalAlign: "middle", marginRight: "4px" }}
                      />
                    )}
                    <span style={{ color: color }}>
                      {char.name}
                    </span>
                  </span>
                );
              })}
            </BorderedBox>
          );
        })}
      </InputGroup>
    </div>
  );
}

export function FleetMembers({ fleetpage = true, setStatTempActive = null }) {
  const authContext = React.useContext(AuthContext);
  const toastContext = React.useContext(ToastContext);
  const [fleetCompositionInfo, setFleetCompositionInfo] = React.useState(null);
  const [errorCount, setErrorCount] = React.useState(0);
  const characterId = authContext.current.id;
  
  // Context menu state
  const [contextMenuOpen, setContextMenuOpen] = React.useState(false);
  const [contextMenuPosition, setContextMenuPosition] = React.useState({ x: 0, y: 0 });
  const [selectedCharacter, setSelectedCharacter] = React.useState(null);
  const [selectedRole, setSelectedRole] = React.useState(null);
  
  // Confirmation dialogs state
  const [showDuplicateConfirm, setShowDuplicateConfirm] = React.useState(false);
  const [showHullConfirm, setShowHullConfirm] = React.useState(false);
  const [pendingUpdate, setPendingUpdate] = React.useState(null);
  React.useEffect(() => {
    apiCall("/api/fleet/fleetcomp?character_id=" + characterId, {})
      .then(setFleetCompositionInfo)
      .catch((err) => {
        setFleetCompositionInfo(null);
        if (!fleetpage) {
          setStatTempActive(false);
        }
      });
  }, [characterId, fleetpage, setStatTempActive]);

  React.useEffect(() => {
    if (!fleetpage) {
      const intervalId = setInterval(() => {
        if (errorCount >= 4) {
          addToast(toastContext, {
            title: "Error",
            message: "Consecutive Error Limit Exceeded, shutting down fleetstats",
            variant: "danger",
          });
          setStatTempActive(false);
          return null;
        }
        apiCall("/api/fleet/fleetcomp?character_id=" + characterId, {})
          .then((fleetCompositionInfo) => {
            setFleetCompositionInfo(fleetCompositionInfo);
            setErrorCount(0);
          })
          .catch((err) => {
            setErrorCount(errorCount + 1);
            if (err.toLowerCase().includes("fleet".toLowerCase())) setStatTempActive(false);
          });
      }, 15000);
      return () => clearInterval(intervalId);
    }
  }, [characterId, errorCount, fleetpage, setStatTempActive, toastContext]);

  // Close context menu when clicking outside
  React.useEffect(() => {
    const handleClick = () => {
      setContextMenuOpen(false);
    };
    if (contextMenuOpen) {
      document.addEventListener("click", handleClick);
      return () => document.removeEventListener("click", handleClick);
    }
  }, [contextMenuOpen]);

  // Handle right-click on character
  const handleCharacterRightClick = React.useCallback((e, member) => {
    e.preventDefault();
    setSelectedCharacter(member);
    setContextMenuPosition({ x: e.clientX, y: e.clientY });
    setContextMenuOpen(true);
  }, []);

  // Perform the MOTD update
  const performMotdUpdate = React.useCallback(async (updateData) => {
    try {
      const response = await apiCall("/api/fleet/update-motd", {
        json: updateData,
      });
      
      if (response.success) {
        addToast(toastContext, {
          title: "Success",
          message: `Added ${updateData.character_name} to ${updateData.role}`,
          variant: "success",
        });
        
        // Refresh fleet composition
        const updatedInfo = await apiCall("/api/fleet/fleetcomp?character_id=" + characterId, {});
        setFleetCompositionInfo(updatedInfo);
      }
    } catch (err) {
      addToast(toastContext, {
        title: "Error",
        message: `Failed to update MOTD: ${err}`,
        variant: "danger",
      });
    }
  }, [characterId, toastContext]);

  // Handle role selection from context menu
  const handleRoleSelect = React.useCallback((role) => {
    if (!selectedCharacter) return;
    
    setSelectedRole(role);
    setContextMenuOpen(false);
    
    // Check for duplicate
    const isDuplicate = fleetCompositionInfo?.role_assignments?.[role]?.some(
      (char) => char.name.toLowerCase() === selectedCharacter.name.toLowerCase()
    ) || false;
    
    // Check hull
    const allowedHulls = getAllowedHullsForRole(role);
    const shipNameLower = selectedCharacter.ship.name.toLowerCase();
    const hasIncorrectHull = !allowedHulls.includes(shipNameLower);
    
    const updateData = {
      character_id: characterId,
      role: role,
      character_name: selectedCharacter.name,
    };
    
    // Show confirmations if needed
    if (isDuplicate && hasIncorrectHull) {
      setPendingUpdate(updateData);
      setShowDuplicateConfirm(true);
    } else if (isDuplicate) {
      setPendingUpdate(updateData);
      setShowDuplicateConfirm(true);
    } else if (hasIncorrectHull) {
      setPendingUpdate(updateData);
      setShowHullConfirm(true);
    } else {
      // No confirmations needed, proceed directly
      performMotdUpdate(updateData);
    }
  }, [selectedCharacter, fleetCompositionInfo, characterId, performMotdUpdate]);

  // Handle duplicate confirmation
  const handleDuplicateConfirm = React.useCallback(() => {
    setShowDuplicateConfirm(false);
    if (pendingUpdate) {
      // Check if we also need hull confirmation
      const allowedHulls = getAllowedHullsForRole(pendingUpdate.role);
      const character = selectedCharacter;
      const shipNameLower = character?.ship.name.toLowerCase();
      const hasIncorrectHull = character && !allowedHulls.includes(shipNameLower);
      
      if (hasIncorrectHull) {
        setShowHullConfirm(true);
      } else {
        performMotdUpdate(pendingUpdate);
        setPendingUpdate(null);
      }
    }
  }, [pendingUpdate, selectedCharacter, performMotdUpdate]);

  // Handle hull confirmation
  const handleHullConfirm = React.useCallback(() => {
    setShowHullConfirm(false);
    if (pendingUpdate) {
      performMotdUpdate(pendingUpdate);
      setPendingUpdate(null);
    }
  }, [pendingUpdate, performMotdUpdate]);

  if (!fleetCompositionInfo) {
    return null;
  }
  if (fleetCompositionInfo) {
    fleetCompositionInfo.wings.forEach((wing) => {
      wing.squads.sort((a, b) => a.id - b.id);
    });
  }

  return (
    <ColumnWaitlistDOM.Category>
      {!fleetpage && (
        <CategoryHeadingDOM>
          <h2>Members</h2>
        </CategoryHeadingDOM>
      )}
      <div style={{ display: "flex", flexWrap: "wrap" }}>
        <div>
          {fleetpage && <Title>Members</Title>}
          {!fleetpage && (
            <FleetDscan
              characterId={authContext.current.id}
              showFull={false}
              memberlist={getFleetMembers(fleetCompositionInfo)}
            />
          )}
          <RoleAssignments roleAssignments={fleetCompositionInfo.role_assignments} />
          <Table style={{ fontSize: "12px" }} fullWidth={fleetpage ? undefined : true}>
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
                      onCharacterRightClick={handleCharacterRightClick}
                    />
                  ))}
                </React.Fragment>
              ))}
            </TableBody>
          </Table>
        </div>
        {fleetpage && (
          <div style={{ marginLeft: "4em" }}>
            <FleetDscan
              characterId={authContext.current.id}
              showFull={true}
              memberlist={getFleetMembers(fleetCompositionInfo)}
            />
          </div>
        )}
      </div>
      
      {/* Context Menu */}
      <RoleContextMenu
        open={contextMenuOpen}
        setOpen={setContextMenuOpen}
        position={contextMenuPosition}
        character={selectedCharacter}
        onRoleSelect={handleRoleSelect}
        authContext={authContext}
      />
      
      {/* Duplicate Confirmation */}
      <Confirm
        open={showDuplicateConfirm}
        setOpen={setShowDuplicateConfirm}
        onConfirm={handleDuplicateConfirm}
        title="Duplicate Assignment"
      >
        <p>
          {selectedCharacter?.name} is already assigned to {selectedRole}. Do you want to add them again?
        </p>
      </Confirm>
      
      {/* Hull Mismatch Confirmation */}
      <Confirm
        open={showHullConfirm}
        setOpen={setShowHullConfirm}
        onConfirm={handleHullConfirm}
        title="Incorrect Hull"
      >
        <p>
          {selectedCharacter?.name} is flying a {selectedCharacter?.ship.name}, which is probably not the correct hull for {selectedRole}.
        </p>
        <p>Do you want to proceed anyway?</p>
      </Confirm>
    </ColumnWaitlistDOM.Category>
  );
}
