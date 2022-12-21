import React, { useContext } from "react";
import { NavLink } from "react-router-dom";
import { AuthContext } from "../contexts";
//import logoImage from "./logo.png";
import styled from "styled-components";
import { InputGroup, Select, NavButton, AButton } from "../Components/Form";
import { EventNotifier } from "../Components/Event";
import { ThemeSelect } from "../Components/ThemeSelect";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faDiscord, faTeamspeak } from "@fortawesome/free-brands-svg-icons";
import { NavLinks, MobileNavButton, MobileNav } from "./Navigation";

const NavBar = styled.div`
  /* existing styles */

  background: linear-gradient(
    to right,
    ${(props) => props.theme.colors.primary},
    ${(props) => props.theme.colors.secondary}
  );
  border-radius: 5px;
  transition: background-color 0.2s ease-in-out;
  &:hover {
    background-color: ${(props) => props.theme.colors.primaryDark};
  }
`;

NavBar.Header = styled.div`
  display: flex;
  @media (max-width: 480px) {
    width: 100%;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    margin-bottom: 1em;
    padding-bottom: 0.2em;
  }
`;

NavBar.LogoLink = styled(NavLink).attrs((props) => ({
  activeClassName: "active",
}))`
  margin-right: 2em;
  flex-grow: 0;
  line-height: 0;
  @media (max-width: 480px) {
    margin-right: unset;
    margin-left: auto;
  }
`;

NavBar.Logo = styled.img`
  width: 150px;
  filter: ${(props) => props.theme.logo.filter};
`;

NavBar.Menu = styled.div`
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  flex-grow: 1;
`;

NavBar.Link = styled(NavLink).attrs((props) => ({
  activeClassName: "active",
}))`
  padding: 1em;
  color: ${(props) => props.theme.colors.accent4};
  text-decoration: none;
  &:hover {
    color: ${(props) => props.theme.colors.text};
  }
  &.active {
    color: ${(props) => props.theme.colors.active};
    font-weight: bold;
  }
`;

NavBar.End = styled.div`
  margin-left: auto;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  > :not(:last-child) {
    @media (max-width: 480px) {
      margin-bottom: 0.4em;
    }
  }
`;

NavBar.Main = styled.div`
  display: flex;
  flex-wrap: wrap;
  @media (max-width: 480px) {
    display: none;
  }
`;

NavBar.Name = styled.div`
  margin-right: 2em;
  @media (max-width: 480px) {
    margin-right: 0em;
    width: 100%;
  }
`;

const Teamspeak = () => {
  const authContext = useContext(AuthContext);

  return (
    <AButton
      title="Join our Teamspeak Server"
      href={`ts3server://ts.candeez.org${
        authContext?.current ? `?nickname=${authContext.current.name}` : ""
      }`}
    >
      <FontAwesomeIcon icon={faTeamspeak} />
    </AButton>
  );
};

export function Menu({ onChangeCharacter, theme, setTheme, sticker, setSticker }) {
  const [isOpenMobileView, setOpenMobileView] = React.useState(false);
  return (
    <AuthContext.Consumer>
      {(whoami) => (
        <NavBar>
          <NavBar.Header>
            <MobileNavButton isOpen={isOpenMobileView} setIsOpen={setOpenMobileView} />
          </NavBar.Header>
          <NavBar.Menu>
            <NavBar.Main>
              <NavLinks whoami={whoami} />
            </NavBar.Main>
            <NavBar.End>
              {whoami && (
                <>
                  <NavBar.Name>
                    <InputGroup fixed>
                      <Select
                        value={whoami.current.id}
                        onChange={(evt) =>
                          onChangeCharacter && onChangeCharacter(parseInt(evt.target.value))
                        }
                        style={{ flexGrow: "1" }}
                      >
                        {whoami.characters.map((character) => (
                          <option key={character.id} value={character.id}>
                            {character.name}
                          </option>
                        ))}
                      </Select>
                      <NavButton exact to="/auth/start/alt">
                        +
                      </NavButton>
                    </InputGroup>
                  </NavBar.Name>
                </>
              )}
              <InputGroup fixed>
                <Teamspeak />
                <AButton title="Discord" href="https://discord.gg/MR3nA9BD9K">
                  <FontAwesomeIcon icon={faDiscord} />
                </AButton>
                <EventNotifier />
                <ThemeSelect
                  theme={theme}
                  setTheme={setTheme}
                  sticker={sticker}
                  setSticker={setSticker}
                />
                {whoami ? (
                  <NavButton exact to="/auth/logout" variant="secondary">
                    Log out
                  </NavButton>
                ) : (
                  <NavButton exact to="/auth/start" variant="primary">
                    Log in
                  </NavButton>
                )}
              </InputGroup>
            </NavBar.End>
            <MobileNav isOpen={isOpenMobileView} whoami={whoami} />
          </NavBar.Menu>
        </NavBar>
      )}
    </AuthContext.Consumer>
  );
}
