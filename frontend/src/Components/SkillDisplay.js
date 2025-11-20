import { useApi, toaster } from "../api";
import { Badge } from "./Badge";
import { InputGroup, Button, Buttons } from "./Form";
import { Col, Row } from "react-awesome-styled-grid";
import { ToastContext } from "../contexts";
import { useHistory } from "react-router-dom";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faPaste } from "@fortawesome/free-solid-svg-icons";

import styled from "styled-components";
import _ from "lodash";
import React from "react";

const SkillDom = {};

SkillDom.Table = styled.div`
  margin-bottom: 2em;
`;

SkillDom.Table.Name = styled.h3`
  border-bottom: solid 2px ${(props) => props.theme.colors.accent2};
  font-weight: bolder;
  padding: 0.75em;
`;

SkillDom.Table.Row = styled.div`
  display: flex;
  padding: 0.5em 0.75em 0.5em 0.75em;
  border-bottom: solid 1px ${(props) => props.theme.colors.accent2};

  &:last-child {
    border-bottom: none;
  }
  &:nth-child(odd) {
    background-color: ${(props) => props.theme.colors.accent1};
  }
  > :last-child {
    margin-left: auto;
  }
`;

const ContextMenu = styled.div`
  position: fixed;
  background-color: ${(props) => props.theme.colors.background};
  border: 1px solid ${(props) => props.theme.colors.accent2};
  border-radius: 4px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  z-index: 1000;
  min-width: 150px;
  padding: 0.25em 0;
`;

const ContextMenuItem = styled.div`
  padding: 0.5em 1em;
  cursor: pointer;
  color: ${(props) => props.theme.colors.text};
  
  &:hover {
    background-color: ${(props) => props.theme.colors.accent1};
  }
`;

const categoryOrder = [
  "Tank",
  "Engineering",
  "Navigation",
  "Gunnery",
  "Targeting",
  "Neural Enhancement",
  "Spaceship Command",
];
const knownCategories = new Set(categoryOrder);

const LevelIndicator = ({ current, skill }) => {
  if (current === 5) {
    return <Badge variant="primary">{current}</Badge>;
  }

  var nextLevel = null;

  for (const [group, variant] of [
    ["gold", "primary"],
    ["elite", "success"],
    ["min", "warning"],
  ]) {
    if (group in skill) {
      if (current >= skill[group]) {
        return (
          <Badge variant={variant}>
            {current}
            {nextLevel}
          </Badge>
        );
      }
      nextLevel = ` / ${skill[group]}`;
    }
  }

  for (const [group, variant] of [
    ["min", "danger"],
    ["elite", "warning"],
    ["gold", "secondary"],
  ]) {
    if (group in skill) {
      return (
        <Badge variant={variant}>
          {current}
          {nextLevel}
        </Badge>
      );
    }
  }

  return null;
};

function SkillTable({ title, current, requirements, ids, category, filterMin }) {
  var entries = [];
  category.forEach((skillId) => {
    if (!(skillId in requirements)) {
      return;
    }
    const skill = requirements[skillId];
    if (filterMin) {
      if (!skill.min) return;
      if (skill.min <= current[skillId]) {
        return;
      }
    }

    entries.push(
      <SkillDom.Table.Row key={skillId}>
        {ids[skillId]} <LevelIndicator current={current[skillId]} skill={skill} />
      </SkillDom.Table.Row>
    );
  });

  if (!entries.length) {
    return null;
  }

  return (
    <Col xs={4} sm={4} md={2}>
      <SkillDom.Table>
        <SkillDom.Table.Name>{title}</SkillDom.Table.Name>
        {entries}
      </SkillDom.Table>
    </Col>
  );
}

export function SkillList({ mySkills, shipName, filterMin }) {
  const ids = _.invert(mySkills.ids);

  if (!(shipName in mySkills.requirements)) {
    return <em>No skill information found</em>;
  }

  const categories = [...categoryOrder];
  _.forEach(_.keys(mySkills.categories), (categoryName) => {
    if (!knownCategories.has(categoryName)) {
      categories.push(categoryName);
    }
  });

  return (
    <>
      <Row>
        {categories.map((category) => (
          <SkillTable
            key={category}
            title={category}
            current={mySkills.current}
            requirements={mySkills.requirements[shipName]}
            category={mySkills.categories[category]}
            ids={ids}
            filterMin={filterMin}
          />
        ))}
      </Row>
    </>
  );
}

export function Legend() {}

function SkillPlanButtons({ ship, plans, mySkills }) {
  const toastContext = React.useContext(ToastContext);
  const history = useHistory();
  const [contextMenu, setContextMenu] = React.useState(null);

  // Filter skill plans that match the current ship and tank type
  const shipPlans = React.useMemo(() => {
    if (!plans || !plans.plans || !ship) return [];
    
    // For ships with tank type variants (like Kronos), only match plans with the exact tank type
    // For other ships, match by exact name
    return plans.plans.filter((plan) => {
      return plan.ships.some((planShip) => {
        // Exact match for tank type variants
        if (ship === "Armor Kronos" || ship === "Shield Kronos") {
          return planShip.name === ship;
        }
        // For other ships, match by exact name
        return planShip.name === ship;
      });
    });
  }, [plans, ship]);

  // Helper function to convert skill level to Roman numeral
  const romanNumeral = (i) => {
    if (i === 1) return "I";
    if (i === 2) return "II";
    if (i === 3) return "III";
    if (i === 4) return "IV";
    if (i === 5) return "V";
    throw new Error("Unlikely skill numeral");
  };

  // Helper function to format plan for clipboard
  const copyablePlan = (levels, lookup) => {
    return levels
      .map(([skillId, level]) => `${lookup[skillId] || "MISSING SKILL"} ${romanNumeral(level)}`)
      .join("\n");
  };

  // Copy plan to clipboard
  const copyPlan = (plan) => {
    if (!mySkills || !mySkills.ids) {
      toaster(toastContext, Promise.resolve("Loading skills..."));
      return;
    }
    
    const lookup = _.invert(mySkills.ids);
    const planText = copyablePlan(plan.levels, lookup);
    
    toaster(
      toastContext,
      navigator.clipboard
        .writeText(planText)
        .then(() => `Copied "${plan.source.name}" to clipboard`)
    );
  };

  // Handle right-click to show context menu
  const handleContextMenu = (e, plan) => {
    e.preventDefault();
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      plan: plan,
    });
  };

  // Close context menu when clicking outside
  React.useEffect(() => {
    const handleClickOutside = () => {
      setContextMenu(null);
    };

    if (contextMenu) {
      document.addEventListener("click", handleClickOutside);
      return () => {
        document.removeEventListener("click", handleClickOutside);
      };
    }
  }, [contextMenu]);

  // Navigate to plan page
  const viewPlan = (plan) => {
    setContextMenu(null);
    history.push(`/skills/plans?plan=${encodeURIComponent(plan.source.name)}`);
  };

  if (!shipPlans || shipPlans.length === 0) {
    return null;
  }

  return (
    <>
      <Buttons style={{ marginBottom: "1em" }}>
        {shipPlans.map((plan) => (
          <Button
            key={plan.source.name}
            onClick={() => copyPlan(plan)}
            onContextMenu={(e) => handleContextMenu(e, plan)}
            title="Left-click to copy plan. Right-click to view plan page."
          >
            <FontAwesomeIcon icon={faPaste} style={{ marginRight: "0.5em" }} />
            Copy {plan.source.name} Skill Plan
          </Button>
        ))}
      </Buttons>
      {contextMenu && (
        <ContextMenu
          style={{
            left: `${contextMenu.x}px`,
            top: `${contextMenu.y}px`,
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <ContextMenuItem onClick={() => viewPlan(contextMenu.plan)}>
            View Plan
          </ContextMenuItem>
        </ContextMenu>
      )}
    </>
  );
}

export function SkillDisplay({ characterId, ship, setShip = null, filterMin = false, plans = null }) {
  const [skills] = useApi(`/api/skills?character_id=${characterId}`);

  return (
    <>
      {setShip != null && (
        <Buttons style={{ marginBottom: "1em" }}>
          <InputGroup>
            <Button active={ship === "Vindicator"} onClick={(evt) => setShip("Vindicator")}>
              Vindicator
            </Button>
            <Button active={ship === "Armor Kronos"} onClick={(evt) => setShip("Armor Kronos")}>
              Armor Kronos
            </Button>
            <Button active={ship === "Shield Kronos"} onClick={(evt) => setShip("Shield Kronos")}>
              Shield Kronos
            </Button>
            <Button active={ship === "Vargur"} onClick={(evt) => setShip("Vargur")}>
              Vargur
            </Button>
            <Button active={ship === "Paladin"} onClick={(evt) => setShip("Paladin")}>
              Paladin
            </Button>
          </InputGroup>
          <InputGroup>
            <Button active={ship === "Loki"} onClick={(evt) => setShip("Loki")}>
              Loki
            </Button>
            <Button active={ship === "Nestor"} onClick={(evt) => setShip("Nestor")}>
              Nestor
            </Button>
          </InputGroup>
          <InputGroup>
            <Button active={ship === "Eos"} onClick={(evt) => setShip("Eos")}>
              Eos
            </Button>
            <Button active={ship === "Damnation"} onClick={(evt) => setShip("Damnation")}>
              Damnation
            </Button>
            <Button active={ship === "Claymore"} onClick={(evt) => setShip("Claymore")}>
              Claymore
            </Button>
          </InputGroup>
        </Buttons>
      )}

      <div style={{ marginBottom: "1em" }}>
        Legend: <Badge variant="danger">Below Requirements</Badge> <Badge variant="warning">Meets Requirements</Badge>{" "}
        <Badge variant="success">Recommended</Badge> <Badge variant="primary">Maxed</Badge>
      </div>
      {plans && <SkillPlanButtons ship={ship} plans={plans} mySkills={skills} />}
      {skills ? (
        <SkillList mySkills={skills} shipName={ship} filterMin={filterMin} />
      ) : (
        <p>Loading skill information</p>
      )}
    </>
  );
}
