import React from "react";
import { NavLink, useParams } from "react-router-dom";
import { Content, PageTitle } from "../../Components/Page";
import styled from "styled-components";
import { ToastContext } from "../../contexts";
import { BadgeData } from "./Badges";
import { errorToaster } from "../../api";
import { Markdown } from "../../Components/Markdown";
import { CardMargin } from "../../Components/Card";
import { replaceTitle, parseMarkdownTitle, usePageTitle } from "../../Util/title";
import BadgeIcon from "../../Components/Badge";

const guideData = {};
function importAll(r) {
  r.keys().forEach((key) => (guideData[key] = r(key)));
}
importAll(require.context("./guides", true, /\.(md|jpg|png)$/));

const GuideContent = styled(Content)`
  max-width: 800px;
`;

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
const DivButton = ({ imageSrc, slug, title, subtitle, children }) => (
  <CardMargin>
    <NavLink style={{ textDecoration: "inherit", color: "inherit" }} exact to={`${slug}`}>
      <ButtonContainer>
        <div style={{ display: "flex", height: "40px" }}>
          {children}
          {imageSrc && <Image src={imageSrc} />}
        </div>
        <Text>{title}</Text>
        <Subtitle>{subtitle}</Subtitle>
      </ButtonContainer>
    </NavLink>
  </CardMargin>
);

export function Guide() {
  const toastContext = React.useContext(ToastContext);
  const { guideName } = useParams();
  const [loadedData, setLoadedData] = React.useState(null);
  const guidePath = `./${guideName}`;
  const filename = `${guidePath}/guide.md`;

  React.useEffect(() => {
    setLoadedData(null);
    if (!(filename in guideData)) return;
    let title = document.title;

    errorToaster(
      toastContext,
      fetch(guideData[filename])
        .then((response) => response.text())
        .then((data) => {
          setLoadedData(data);
          replaceTitle(parseMarkdownTitle(data));
        })
    );
    return () => (document.title = title);
  }, [toastContext, filename]);

  const resolveImage = (name) => {
    const originalName = `${guidePath}/${name}`;
    if (originalName in guideData) {
      return guideData[originalName];
    }
    return name;
  };

  if (!guideData[filename]) {
    return (
      <>
        <strong>Not found!</strong> The guide could not be loaded.
      </>
    );
  }

  if (!loadedData) {
    return (
      <>
        <em>Loading...</em>
      </>
    );
  }

  return (
    <GuideContent style={{ maxWidth: "800px" }}>
      <Markdown transformImageUri={resolveImage} transformLinkUri={null}>
        {loadedData}
      </Markdown>
    </GuideContent>
  );
}
/*
function GuideCard({ icon, slug, name, children }) {
  return (
    <CardMargin>
      <NavLink style={{ textDecoration: "inherit", color: "inherit" }} exact to={`${slug}`}>
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
}*/

export function GuideIndex() {
  usePageTitle("Guides");
  return (
    <>
      <PageTitle style={{ marginLeft: "40%", marginBottom: "1em" }}>Guides</PageTitle>
      <GuideArray style={{ justifyContent: "space-around" }}>
        <DivButton
          imageSrc="https://images.evetech.net/types/14268/icon"
          title="DDD Guide"
          subtitle="get GUD & Read"
          slug="guide/ddd"
        />
        <DivButton title="Badges" subtitle="Showing off u GUD" slug="badges">
          <BadgeIcon type={"DPS"} height={"30px"} />
          <BadgeIcon type={"LOGI"} height={"30px"} />
          <BadgeIcon type={"ALT"} height={"30px"} />
        </DivButton>
        <DivButton
          imageSrc="https://images.evetech.net/types/33400/icon"
          title="Marauder Guide"
          subtitle="Count to 2 and you gucci"
          slug="guide/marauder"
        />
      </GuideArray>
    </>
  );
}

export function BadgeIndex() {
  return <BadgeData />;
}
