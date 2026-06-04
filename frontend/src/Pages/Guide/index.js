import { NavLink, useParams } from "react-router-dom";
import { PageTitle } from "../../Components/Page";
import styled from "styled-components";
import { CardMargin } from "../../Components/Card";
import { usePageTitle } from "../../Util/title";
import BadgeIcon from "../../Components/Badge";
import { BadgeData } from "./Badges";
import { GuideViewer } from "./GuideViewer";
import { guidePath, useGuides } from "./useGuides";

const ButtonContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 150px;
  background-color: ${(props) => props.theme.colors.accent1};
  border-radius: 5px;
  cursor: pointer;
  &:hover {
    border: solid 1px ${(props) => props.theme.colors.accent2};
  }
  @media (max-width: 600px) {
    width: 100%;
  }
  box-shadow: 3px 3px 1px ${(props) => props.theme.colors.shadow};
`;

const Image = styled.img`
  height: 40px;
  margin-bottom: 5px;
`;

const Text = styled.span`
  font-size: 20px;
`;

const Subtitle = styled.span`
  font-size: 14px;
  color: gray;
`;

const GuideArray = styled.div`
  display: flex;
  align-items: stretch;
  flex-wrap: wrap;
  border-top: 2px solid;
  padding-top: 5em;
  border-color: ${(props) => props.theme.colors.accent2};
`;

const DivButton = ({ imageSrc, to, title, subtitle, children }) => (
  <CardMargin>
    <NavLink style={{ textDecoration: "inherit", color: "inherit" }} exact to={to}>
      <ButtonContainer>
        <div style={{ display: "flex", height: "40px" }}>
          {children}
          {imageSrc && <Image src={imageSrc} alt="" />}
        </div>
        <Text>{title}</Text>
        {subtitle && <Subtitle>{subtitle}</Subtitle>}
      </ButtonContainer>
    </NavLink>
  </CardMargin>
);

export function Guide() {
  const { guideName } = useParams();
  return <GuideViewer slug={guideName} />;
}

export function GuideIndex() {
  const { publicGuides, loading } = useGuides();
  usePageTitle("Guides");

  return (
    <>
      <PageTitle style={{ marginLeft: "40%", marginBottom: "1em" }}>Guides</PageTitle>
      <GuideArray style={{ justifyContent: "space-around" }}>
        {loading && <em>Loading guides…</em>}
        {!loading &&
          publicGuides.map((guide) => (
            <DivButton
              key={guide.slug}
              imageSrc={guide.icon}
              title={guide.title}
              subtitle={guide.subtitle}
              to={guidePath(guide)}
            />
          ))}
        <DivButton title="Badges" subtitle="Showing off u GUD" to="/badges">
          <BadgeIcon type={"DPS"} height={"30px"} />
          <BadgeIcon type={"LOGI"} height={"30px"} />
          <BadgeIcon type={"ALT"} height={"30px"} />
        </DivButton>
      </GuideArray>
    </>
  );
}

export function BadgeIndex() {
  return <BadgeData />;
}
