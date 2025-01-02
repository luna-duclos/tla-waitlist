import React from "react";
import { Box } from "../../Components/Box";
import { Modal } from "../../Components/Modal";
import { Title, Content } from "../../Components/Page";
import styled from "styled-components";

//import { ImplantOut } from "../Fits/FittingSortDisplay";
import { AButton } from "../../Components/Form";
//import { NavButton, InputGroup } from "../../Components/Form";
//import { InfoNote } from "../../Components/NoteBox";

import { BadgeDOM, BadgeModal } from "../../Components/Badge";
import BadgeIcon from "../../Components/Badge";
import { usePageTitle } from "../../Util/title";

const BadgeDisplay = styled.div`  display: flex;
  flex-wrap: wrap;
  margin-top: 1em;`;

const BadgeImages = {};
function importAll(r) {
r.keys().forEach((key) => (BadgeImages[key] = r(key)));
}
importAll(require.context("./badges", true, /\.(jpg|png)$/));

function BadgeButton({ name, icon, children }) {
const [modalOpen, setModalOpen] = React.useState(false);
return (

<>
{modalOpen ? (
<Modal open={true} setOpen={setModalOpen}>
<Box>
<BadgeModal>
<BadgeModal.Title>
<div style={{ display: "flex", alignItems: "center" }}>
<BadgeIcon type={icon} height={"40px"} />
</div>
<Title>{name} &nbsp;</Title>
</BadgeModal.Title>
{children}
</BadgeModal>
</Box>
</Modal>
) : null}

      <BadgeDOM>
        <a onClick={(evt) => setModalOpen(true)}>
          <BadgeDOM.Content>
            <BadgeDOM.Icon>
              <BadgeIcon type={icon} height={"50px"} />
            </BadgeDOM.Icon>
            {name}
          </BadgeDOM.Content>
        </a>
      </BadgeDOM>
    </>

);
}

export function BadgeData() {
  usePageTitle("Badges");
  const [exampleOpen, setExampleOpen] = React.useState(false);
  return (
    <>
      {exampleOpen ? (
        <Modal open={true} setOpen={setExampleOpen}>
          <Box style={{ width: "100%" }}>
            <img
              src={BadgeImages["./examplediscord.png"]}
              alt={"examplediscord"}
              style={{ width: "100%" }}
            />
          </Box>
        </Modal>
      ) : null}
      <Content style={{ marginBottom: "2em" }}>
        <h1>Badges</h1>
        <p>
          To get the T, A, or ! badge, use the
          <a
            href="https://discord.com/channels/930000021083005028/930014112635826196/1287232984495423570"
            target="_blank"
            rel="noopener noreferrer"
            style={{ marginLeft: "0.5em" }}
          >
            ticket
          </a> system on Discord. Make sure your screenshot is saved on an image
          hosting website & shows the highlighted areas as per the
          <AButton
            onClick={(evt) => setExampleOpen(true)}
            style={{ marginLeft: "0.5em" }}
          >
            example
          </AButton>
        </p>
        <Title>Pilot Badge</Title>
        <BadgeDisplay>
          <BadgeButton name="DPS" icon={"DPS"}>
            The DPS Badge is given to a pilot that has met the main DPS Values
            on the fitting page. You must have implant slots 9 & 10 to get this
            badge.
          </BadgeButton>
          <BadgeButton name="Logi" icon={"LOGI"}>
            The Logi Badge is given out by an FC that has vouched for your logi
            performance. It is not required to fly Logi.
          </BadgeButton>
          <BadgeButton name="Alt Approved" icon={"ALT"}>
            The Alt Approved Badge is given to a pilot that has met the DPS
            Values for alts on the fitting page. Your main or T badge must have
            a <b>full</b> implant set before you can apply for this badge and
            recieve your guaranteed alt spot.
          </BadgeButton>
          <BadgeButton name="VINDICATION!" icon={"VINDI"}>
            The VINDICATION badge is given to a pilot that has met the DPS and Web values on the fitting page.
          </BadgeButton>
        </BadgeDisplay>
        <Title>Commander Badges</Title>
        <BadgeDisplay>
          <BadgeButton name="Training FC" icon={"TRAINEE"}>
            Trainee isk bringers
          </BadgeButton>
          <BadgeButton name="Fleet Commander" icon={"HQ-FC"}>
            Isk bringers
          </BadgeButton>
        </BadgeDisplay>
      </Content>
    </>
  );
}
