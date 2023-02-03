import { NavButton, InputGroup, CenteredParagraph } from "../Components/Form";
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
            your fit looks the same. The associated DPS number is expected after roughly 2 weeks of
            flying with us.
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
              <b>What is the DPS requirement?</b>
              The DPS requirement is the amount of DPS each pilot is expected to in their respective
              hulls (guns only, no heat, no drugs, no drones) after roughly 2 weeks of flying. A
              combination of Skills, Abyssals, and Implants are needed to achieve these numbers.
              <b>Can I bring my Vindicator? </b>
              Yes! Our Vindicator spots are limited so it is recommend to train into a Marauder. You
              will be expected to DDD so please read the{" "}
              <NavLink to="/guide/ddd">DDD GUIDE</NavLink>
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
        </ThreeColumn>
      </Content>
    </>
  );
}
