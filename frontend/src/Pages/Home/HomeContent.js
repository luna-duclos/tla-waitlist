import { NavLink } from "react-router-dom";
import { Content } from "../../Components/Page";
import { CenteredParagraph } from "../../Components/Form";
import { useTranslation, Trans } from "react-i18next";

import styled from "styled-components";

export const HomeContentContainer = styled.div`
  container-type: inline-size;
  container-name: home-content;
`;

export const ThreeColumn = styled.div`
  display: flex;
  justify-content: space-between;
  @media (max-width: 1100px) {
    flex-wrap: wrap;
  }
  @container home-content (max-width: 1100px) {
    flex-wrap: wrap;
  }
`;

const introComponents = {
  guideLink: <NavLink to="/guide" />,
  discordLink: <a href="https://discord.gg/MR3nA9BD9K" />, // eslint-disable-line jsx-a11y/anchor-has-content
  fitsLink: <NavLink to="/fits" />,
  bold: <b />,
};

const faqAnswerComponents = {
  must: <b />,
  dddLink: <NavLink to="/guide/ddd" />,
};

/** Main home page copy (used on / and in admin preview). */
export function HomeContent() {
  const { t } = useTranslation("home");

  return (
    <HomeContentContainer>
    <Content>
      <CenteredParagraph>
        <CenteredParagraph.Head style={{ marginBottom: "1em" }}>
          {t("welcomeTitle")}
        </CenteredParagraph.Head>
        <CenteredParagraph.Paragraph>
          <Trans i18nKey="intro" components={introComponents} />
        </CenteredParagraph.Paragraph>
      </CenteredParagraph>

      <ThreeColumn>
        <CenteredParagraph>
          <CenteredParagraph.Head>{t("whatIsTlaTitle")}</CenteredParagraph.Head>
          <CenteredParagraph.ParagraphALT>{t("whatIsTlaBody")}</CenteredParagraph.ParagraphALT>
        </CenteredParagraph>
        <CenteredParagraph>
          <CenteredParagraph.Head>{t("armorShieldTitle")}</CenteredParagraph.Head>
          <CenteredParagraph.ParagraphALT>{t("armorShieldBody")}</CenteredParagraph.ParagraphALT>
        </CenteredParagraph>
        <CenteredParagraph>
          <CenteredParagraph.Head>{t("faqTitle")}</CenteredParagraph.Head>
          <CenteredParagraph.ParagraphALT>
            <b>{t("faqFitQuestion")}</b> <Trans i18nKey="faqFitAnswer" components={faqAnswerComponents} />
            <b>{t("faqDpsQuestion")}</b> {t("faqDpsAnswer")}
            <b>{t("faqLogiQuestion")}</b> {t("faqLogiAnswer")}
            <b>{t("faqVindiQuestion")}</b>{" "}
            <Trans i18nKey="faqVindiAnswer" components={faqAnswerComponents} />
          </CenteredParagraph.ParagraphALT>
        </CenteredParagraph>
      </ThreeColumn>
    </Content>
    </HomeContentContainer>
  );
}
