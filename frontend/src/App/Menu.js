import React, { useContext, useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import { AuthContext } from "../contexts";
import banner from "./banner.png";
import styled from "styled-components";
import { InputGroup, SelectAlt, NavButton, AButtonAlt, NavButtonAlt } from "../Components/Form";
import { EventNotifier } from "../Components/Event";
import { ThemeSelect } from "../Components/ThemeSelect";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faDiscord, faTeamspeak } from "@fortawesome/free-brands-svg-icons";
import { NavLinks, MobileNavButton, MobileNav } from "./Navigation";

const Tlaimage = styled.div`
  background-image: url(${banner});
  background-size: 100% auto;
  background-repeat: no-repeat;
  padding 0.5em;
  border-radius: 0 0 20px 20px;
  box-shadow: 2px 2px 10px rgba(0, 0, 0, 0.3);
  transition: border-radius 0.1s ease-in-out;
  @media (max-width: 700px) {
	  background-size: auto 100%;
	  border-radius: ${(props) => props.mobileopen && "unset"};
	  
    
  }
  padding-top: 6.5em;

`;

const MyProfile = styled(NavLink).attrs((props) => ({
  activeClassName: "active",
}))`
  height: 2.5em;
  img {
    &:hover {
      opacity: 0.7;
    }
  }
`;

const NavBar = styled.div`
  margin-bottom: 2em;
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
    <AButtonAlt
      title="Join our Teamspeak Server"
      href={`ts3server://ts.candeez.org${
        authContext?.current ? `?nickname=${authContext.current.name}` : ""
      }`}
    >
      <FontAwesomeIcon icon={faTeamspeak} />
    </AButtonAlt>
  );
};

export function Menu({ onChangeCharacter, theme, setTheme, sticker, setSticker }) {
  const [isOpenMobileView, setOpenMobileView] = React.useState(false);
  const [width, setWidth] = useState(window.innerWidth);
  useEffect(() => {
    const handleResize = () => setWidth(window.innerWidth);
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);
  return (
    <AuthContext.Consumer>
      {(whoami) => (
        <NavBar>
          <Tlaimage mobileopen={isOpenMobileView}>
            <NavBar.Menu>
              <NavBar.Main>
                <NavLinks whoami={whoami} />
              </NavBar.Main>
              <NavBar.End>
                {whoami && (
                  <>
                    <NavBar.Name>
                      <InputGroup fixed>
                        <MyProfile exact to="/pilot" title="My Profile">
                          <img
                            style={{ maxHeight: "100%", borderRadius: "20px 0 0 20px" }}
                            src={`https://images.evetech.net/characters/${
                              whoami.current.id ?? 1
                            }/portrait?size=128`}
                            alt="Character Portrait"
                          />
                        </MyProfile>
                        <SelectAlt
                          value={whoami.current.id}
                          onChange={(evt) =>
                            onChangeCharacter && onChangeCharacter(parseInt(evt.target.value))
                          }
                          style={{ flexGrow: "1", border: "unset" }}
                        >
                          {whoami.characters.map((character) => (
                            <option key={character.id} value={character.id}>
                              {character.name}
                            </option>
                          ))}
                        </SelectAlt>
                        <NavButtonAlt exact to="/auth/start/alt">
                          +
                        </NavButtonAlt>
                      </InputGroup>
                    </NavBar.Name>
                  </>
                )}

                <InputGroup fixed>
                  {width < 481 && (
                    <MobileNavButton isOpen={isOpenMobileView} setIsOpen={setOpenMobileView} />
                  )}
                  <Teamspeak />
                  <AButtonAlt title="Discord" href="https://discord.gg/MR3nA9BD9K">
                    <FontAwesomeIcon icon={faDiscord} />
                  </AButtonAlt>
                  <EventNotifier />
                  <ThemeSelect
                    theme={theme}
                    setTheme={setTheme}
                    sticker={sticker}
                    setSticker={setSticker}
                  />
                  {whoami ? (
                    <NavButtonAlt exact to="/auth/logout">
                      Log out
                    </NavButtonAlt>
                  ) : (
                    <NavButton exact to="/auth/start" variant="primary">
                      Log in
                    </NavButton>
                  )}
                </InputGroup>
              </NavBar.End>
            </NavBar.Menu>
          </Tlaimage>
          <MobileNav isOpen={isOpenMobileView} whoami={whoami} />
        </NavBar>
      )}
    </AuthContext.Consumer>
  );
}
