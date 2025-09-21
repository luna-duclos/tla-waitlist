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
`;
*/

const ThreeColumn = styled.div`  display: flex;
  justify-content: space-between;
  @media (max-width: 1100px) {
    flex-wrap: wrap;
  }`;

export function Home() {
  return (
    <>
      <InputGroup style={{ marginTop: "5em" }}>
        <NavButton to={`/legal`}>Legal</NavButton>
      </InputGroup>
      <Content>
        <CenteredParagraph>
          <CenteredParagraph.Head style={{ marginBottom: "1em" }}>
            Welcome to TLA
          </CenteredParagraph.Head>
          <CenteredParagraph.Paragraph>
            Please have a read of the <NavLink to="/guide">GUIDES</NavLink>, join the{" "}
            <a href="https://discord.gg/MR3nA9BD9K">DISCORD</a> & in game channel TLA Incursions
            to ask any questions that aren&apos;t answered here, we&apos;re more than happy to help.
            Look at our <NavLink to="/fits">FITS</NavLink> and make sure your fit matches ours.
            The associated DPS number is expected after roughly 2 weeks of flying with us,
            DPS requirements must be met <b>WITHOUT DRUGS & DRONES, GUN DAMAGE ONLY</b>.
          </CenteredParagraph.Paragraph>
        </CenteredParagraph>

        <ThreeColumn>
          <CenteredParagraph>
            <CenteredParagraph.Head>What Is TLA?</CenteredParagraph.Head>
            <CenteredParagraph.ParagraphALT>
              We are an HQ Incursion Community that specializes in multi-boxing Marauders (Suggested fits can be found in the fits
              section of the waitlist). Marauder pilots are guaranteed an additional paid spot in
              fleet for an alt once both T and A badge requirements are met.
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
          <CenteredParagraph>
            <CenteredParagraph.Head>
              Why are there both Armor and Shield Fits?
            </CenteredParagraph.Head>
            <CenteredParagraph.ParagraphALT>
             We accommodate both tank types to allow anyone to fly their marauder with us and fly mixed logi to complement this.
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
          <CenteredParagraph>
            <CenteredParagraph.Head>FAQ&apos;S</CenteredParagraph.Head>
            <CenteredParagraph.ParagraphALT>
              <b>My fit is different to the one on the website. Can I bring it?</b>
              Amulet Implant sets with the appropriate rigs are accepted (You <b>MUST</b> use T2 Hyperspatial
              rig in replacement of the trimark rig). Fits differing from the ones listed on the website may be welcomed in fleet,
              however it is always upto active FC discretion.
              <b>What is the DPS requirement?</b>
              The DPS requirement is the amount of DPS each pilot is expected to reach in their respective
              hulls (guns only, no heat, no drugs, no drones) after 24 in fleet hours. A
              combination of Skills, Abyssals, and Implants are needed to achieve these numbers.
              <b>I dont have the L badge, can I still fly logi?</b>
              Yes, you can fly any ship at any point, our logi fits require you to be 8 rep stable on loki and 5 rep + MWD stable on Nestor.
              <b>Can I bring my Vindicator?</b>
              Yes! Our Vindicator spots are limited. You
              will be expected to DDD so please read the{" "}
              <NavLink to="/guide/ddd">DDD GUIDE</NavLink>.
            </CenteredParagraph.ParagraphALT>
          </CenteredParagraph>
        </ThreeColumn>
      </Content>
    </>

);
}
