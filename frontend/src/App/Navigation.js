import React from "react";
import { useTranslation } from "react-i18next";
import { useLocation } from "react-router-dom";
import styled from "styled-components";
import { MobileButton } from "../Components/Form";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faBars, faTimes } from "@fortawesome/free-solid-svg-icons";
import { NavBarLink } from "./Navigation.styles";

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
  const { t } = useTranslation("nav");

  return (
    <>
      <NavBarLink exact to="/">
        {t("home")}
      </NavBarLink>
      {whoami && (
        <NavBarLink exact to="/waitlist">
          {t("waitlist")}
        </NavBarLink>
      )}
      <NavBarLink exact to="/guide">
        {t("guides")}
      </NavBarLink>
      <NavBarLink exact to="/fits">
        {t("fits")}
      </NavBarLink>
      {whoami && (
        <NavBarLink exact to="/skills">
          {t("skills")}
        </NavBarLink>
      )}
      <NavBarLink exact to="/isk-h/calc">
        {t("iskPerHour")}
      </NavBarLink>
      {whoami && whoami.access["fleet-view"] && (
        <NavBarLink exact to="/fc/fleet">
          {t("fleet")}
        </NavBarLink>
      )}
      {whoami && whoami.access["fleet-view"] && (
        <NavBarLink exact to="/fc">
          {t("fc")}
        </NavBarLink>
      )}
      {whoami && whoami.access["search"] && (
        <NavBarLink exact to="/fc/search">
          {t("search")}
        </NavBarLink>
      )}
    </>
  );
}
