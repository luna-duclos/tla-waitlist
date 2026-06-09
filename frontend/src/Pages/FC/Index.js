import React from "react";
import { NavLink } from "react-router-dom";
import { PageTitle } from "../../Components/Page";
import { AuthContext } from "../../contexts";
import { Card, CardArray, CardMargin } from "../../Components/Card";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  faChartLine,
  faShieldAlt,
  faUserShield,
  faBullhorn,
  faBan,
  faCreditCard,
  faDatabase,
  faBiohazard,
  faGraduationCap,
} from "@fortawesome/free-solid-svg-icons";
import { usePageTitle } from "../../Util/title";
import { guidePath, useGuides } from "../Guide/useGuides";

const FC_GUIDE_ICONS = {
  documentation: faBiohazard,
  trainee: faGraduationCap,
};

function GuideCard({ icon, slug, name, children }) {
  return (
    <CardMargin>
      <NavLink
        style={{ textDecoration: "inherit", color: "inherit" }}
        exact
        to={guidePath({ slug, section: "fc" })}
      >
        <Card
          title={
            <>
              <FontAwesomeIcon fixedWidth icon={icon} /> {name}
            </>
          }
        >
          <p>{children}</p>
        </Card>
      </NavLink>
    </CardMargin>
  );
}

export function FCMenu() {
  const authContext = React.useContext(AuthContext);
  const { fcGuides } = useGuides();
  usePageTitle("FC Menu");
  return (
    <>
      <PageTitle>FC Dashboard</PageTitle>
      <CardArray>
        {authContext && authContext.access["waitlist-tag:HQ-FC"] && (
          <GuideCard slug="announcements" name="Announcements" icon={faBullhorn} />
        )}
        {authContext && authContext.access["bans-manage"] && (
          <GuideCard slug="bans" name="Bans" icon={faBan} />
        )}
        {authContext && authContext.access["badges-manage"] && (
          <GuideCard slug="badges" name="Badges" icon={faShieldAlt} />
        )}
        {authContext && authContext.access["commanders-view"] && (
          <GuideCard slug="commanders" name="Commanders" icon={faUserShield} />
        )}
        {authContext && authContext.access["fleet-view"] && (
          <GuideCard slug="srp" name="SRP" icon={faCreditCard} />
        )}
        {authContext && authContext.access["commanders-manage:admin"] && (
          <>
            <CardMargin>
              <NavLink style={{ textDecoration: "inherit", color: "inherit" }} exact to="/srp-admin">
                <Card
                  title={
                    <>
                      <FontAwesomeIcon fixedWidth icon={faCreditCard} /> SRP Admin
                    </>
                  }
                >
                </Card>
              </NavLink>
            </CardMargin>
            <CardMargin>
              <NavLink style={{ textDecoration: "inherit", color: "inherit" }} exact to="/admin/data-files">
                <Card
                  title={
                    <>
                      <FontAwesomeIcon fixedWidth icon={faDatabase} /> Data Files Admin
                    </>
                  }
                >
                </Card>
              </NavLink>
            </CardMargin>
          </>
        )}
        {fcGuides.map((guide) => {
          if (guide.access && !authContext?.access[guide.access]) {
            return null;
          }
          return (
            <GuideCard
              key={guide.slug}
              slug={guide.slug}
              name={guide.title}
              icon={FC_GUIDE_ICONS[guide.slug] ?? faBiohazard}
            >
              {guide.subtitle}
            </GuideCard>
          );
        })}
        {authContext && authContext.access["stats-view"] && (
          <GuideCard slug="stats" name="Statistics" icon={faChartLine} />
        )}
      </CardArray>
    </>
  );
}
