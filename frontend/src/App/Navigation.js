import React from "react";
import { useLocation } from "react-router-dom";
import { NavLink } from "react-router-dom";
import styled from "styled-components";
import { MobileButton } from "../Components/Form";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faBars, faTimes } from "@fortawesome/free-solid-svg-icons";

const Links = styled(NavLink).attrs((props) => ({
  activeClassName: "active",
}))`
  padding: 0 1em;
  color: #bbb;
  text-decoration: none;
  text-shadow: 2px 2px 5px rgba(0, 0, 0, 0.5);
  &:hover {
    color: #cccccc;
    border-radius: 2px;
  }
  &.active {
    color: #eeeeee;
    text-shadow: 0px 0px 5px rgba(255, 255, 255, 0.5);
  }
  @media (max-width: 480px) {
    padding: 1em;
    &.active {
      background-color: ${(props) => props.theme.colors.accent2};
      border-radius: 4px;
    }
  }
`;

const MobileButtonDOM = styled.div`
  * {
    display: ${(props) => (props.open ? "flex" : "none")};
    flex-wrap: wrap;
    flex-direction: column;
  }
`;

const Content = styled.div`
  padding: 0.3em;
  width: 100%;
  transition-delay: 2s;
  opacity: ${(props) => (props.open ? "1" : "0")};
  height: ${(props) => (props.open ? "100%" : "0")};
  transition: all 0.3s;
  background-color: #2e2e2e;
  border-radius: 0 0 5px 5px;
`;

export function MobileNav({ isOpen, whoami }) {
  return (
    <>
      <Content open={isOpen}>
        <MobileButtonDOM open={isOpen}>
          <NavLinks whoami={whoami} />
        </MobileButtonDOM>
      </Content>
    </>
  );
}

export function MobileNavButton({ isOpen, setIsOpen }) {
  const location = useLocation();
  React.useEffect(() => {
    setIsOpen(false);
  }, [location, setIsOpen]);
  return (
    <>
      {isOpen ? (
        <MobileButton onClick={(evt) => setIsOpen(false)}>
          <FontAwesomeIcon icon={faTimes} />
        </MobileButton>
      ) : (
        <MobileButton onClick={(evt) => setIsOpen(true)}>
          <FontAwesomeIcon icon={faBars} />
        </MobileButton>
      )}
    </>
  );
}

export function NavLinks({ whoami }) {
  return (
    <>
      <Links exact to="/">
        Home
      </Links>
      {whoami && (
        <>
          <Links exact to="/waitlist">
            Waitlist
          </Links>
        </>
      )}
      <Links exact to="/guide">
        Guides
      </Links>
      <Links exact to="/fits">
        Fits
      </Links>
      <Links exact to="/isk-h/calc">
        ISK/h
      </Links>
      {whoami && whoami.access["fleet-view"] && (
        <Links exact to="/fc/fleet">
          Fleet
        </Links>
      )}
      {whoami && whoami.access["fleet-view"] && (
        <Links exact to="/fc">
          FC
        </Links>
      )}
      {whoami && whoami.access["search"] && (
        <Links exact to="/fc/search">
          Search
        </Links>
      )}
    </>
  );
}
