import { NavLink } from "react-router-dom";
import styled from "styled-components";

export const NavBarLink = styled(NavLink).attrs({
  activeClassName: "active",
})`
  display: inline-block;
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
      background-color: #454545;
      border-radius: 4px;
    }
  }
`;
