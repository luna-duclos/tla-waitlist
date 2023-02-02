import { NavButton, InputGroup } from "../Components/Form";
import { NavLink } from "react-router-dom";
import { Content } from "../Components/Page";

import styled from "styled-components";

/*
const BannerImage = styled.div`
  background-image: url("https://i.imgur.com/8NXTXqj.png");
  width: 100%;
  height: 400px;
  background-size: 100% auto;
  background-repeat: no-repeat;
  border-radius: 50px;
  box-shadow: 2px 2px 10px rgba(0, 0, 0, 0.3);
  margin-bottom: 2em;
`;*/

const CenteredParagraph = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  width: 100%;
`;
CenteredParagraph.Head = styled.h3`
  font-weight: bold;
  font-size: 1.5em;
  text-decoration: underline;
`;
CenteredParagraph.Paragraph = styled.div`
  margin: 0;
  border-top: 2px solid;
  padding: 2em 5em;
  @media (max-width: 720px) {
    padding: 1em;
  }
  border-color: ${(props) => props.theme.colors.accent2};
`;
CenteredParagraph.ParagraphALT = styled.div`
  margin: 0;
  padding: 2em 5em;
  @media (max-width: 720px) {
    padding: 1em;
  }
  b {
    display: block;
  }
  border-color: ${(props) => props.theme.colors.accent2};
`;

const ThreeColumn = styled.div`
  display: flex;
  justify-conent: space-between;
  @media (max-width: 1100px) {
    flex-wrap: wrap;
  }
`;

export function Home() {
  return (
    <>
      <InputGroup style={{ marginTop: "5em" }}>
        <NavButton to={`/legal`}>Legal</NavButton>
      </InputGroup>
      <Content>
        <CenteredParagraph>
          <CenteredParagraph.Head style={{ marginBottom: "1em" }}>
            Are you new to TLA or incursions in general?
          </CenteredParagraph.Head>
          <CenteredParagraph.Paragraph>
            Please have a read of some of the <NavLink to="/guide">GUIDES</NavLink> and join the{" "}
            <a href="https://discord.com/invite/D8pkZhE8DD">DISCORD</a> to ask any questioned that
            aren&apos;t answered here. Look at our <NavLink to="/fits">FITS</NavLink> and make sure
            your fit looks the same and meets the DPS numbers above the fit you choose to fly.
          </CenteredParagraph.Paragraph>
          <CenteredParagraph.Paragraph>
            Please join the in-game channel <b>TLA Incursions</b> to X Up for our fleets For any
            questions that aren&apos;t answered below, feel free to drop by and ask! We are more
            than happy to help!
          </CenteredParagraph.Paragraph>
        </CenteredParagraph>

        <ThreeColumn>
          <CenteredParagraph>
            <CenteredParagraph.Head>What Is TLA?</CenteredParagraph.Head>
            <CenteredParagraph.ParagraphALT>
              We are a HQ Incursion Community that specialises in multi-boxing Marauders (Suggested
              fits can be found above). Marauder pilots are guaranteed an additional paid spot in
              fleet for an alt. Your alt is preferred to be in a marauder but can come in anything
              to get payout until its a marauder.
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
          <CenteredParagraph>
            <CenteredParagraph.Head>
              Why are there both Armor and Shield Fits?
            </CenteredParagraph.Head>
            <CenteredParagraph.ParagraphALT>
              We accommodate both tank types to allow anyone to fly their marauder with us.
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
          <CenteredParagraph>
            <CenteredParagraph.Head>FAQ&apos;S</CenteredParagraph.Head>
            <CenteredParagraph.ParagraphALT>
              <b>My fit is different to the one on the website. Can I bring it? </b>
              Fits differing from the ones listed on the website may be welcome in fleet. Ask
              in-game, in Discord or the active FC for clarification.
              <b>Do I need abyssals? </b>
              Abyssal mods are required for mains, the DPS numbers on the fitting page should be met
              to fly with TLA. First alts must now hit at least 93% of their respective hull&apos;s
              required DPS. Failure to do will be considered a sponge. (GUNS ONLY, NO HEAT, NO
              DRUGS, NO DRONES)
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
        </ThreeColumn>
      </Content>
    </>
  );
}
