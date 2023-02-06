import React from "react";
import { ToastContext, AuthContext } from "../../contexts";
import { addToast } from "../../Components/Toast";
import { apiCall, errorToaster, useApi } from "../../api";
import { Button, Buttons, InputGroup, Textarea } from "../../Components/Form";
import { useLocation } from "react-router-dom";
import { Content, PageTitle } from "../../Components/Page";
import { FitDisplay, ImplantDisplay } from "../../Components/FitDisplay";
import _ from "lodash";
import { Box } from "../../Components/Box";
import styled from "styled-components";
import { usePageTitle } from "../../Util/title";

const exampleFit = String.raw`
[Paladin, Paladin]
Mega Pulse Laser II
Mega Pulse Laser II
Mega Pulse Laser II
Mega Pulse Laser II
Small Tractor Beam I
Large Remote Armor Repairer II
Imperial Navy Large Remote Capacitor Transmitter
Bastion Module I

Large Micro Jump Drive
Federation Navy Tracking Computer
Federation Navy Tracking Computer
Gist X-Type 500MN Microwarpdrive

Imperial Navy Heat Sink
Imperial Navy Heat Sink
Imperial Navy Heat Sink
Imperial Navy Heat Sink
Centum A-Type Multispectrum Energized Membrane
Centum A-Type Multispectrum Energized Membrane
Imperial Navy 1600mm Steel Plates

Large Trimark Armor Pump II
Large Energy Burst Aerator II



Conflagration L x40
Scorch L x40
Blood Radio L x20
Optimal Range Script x2
Tracking Speed Script x2
`.trim();

const exampleMessage = String.raw`
bringing 1 alt
`.trim();

const WaitlistWrap = styled.div`
  display: flex;
  @media (max-width: 1100px) {
    display: block;
  }
`;

async function xUp({ character, eft, toastContext, waitlist_id, alt, messagexup }) {
  await apiCall("/api/waitlist/xup", {
    json: {
      eft: eft,
      character_id: character,
      waitlist_id: parseInt(waitlist_id),
      is_alt: alt,
      messagexup: messagexup,
    },
  });

  addToast(toastContext, {
    title: "Added to waitlist.",
    message: "Your X has been added to the waitlist!",
    variant: "success",
  });

  if (window.Notification) {
    Notification.requestPermission();
  }
}

export function Xup({ setXupOpen }) {
  usePageTitle("X-up");
  const toastContext = React.useContext(ToastContext);
  const authContext = React.useContext(AuthContext);
  const queryParams = new URLSearchParams(useLocation().search);
  const [eft, setEft] = React.useState("");
  const [messagexup, setMessagexup] = React.useState("");
  const [isSubmitting, setIsSubmitting] = React.useState(false);
  const [reviewOpen, setReviewOpen] = React.useState(false);
  const [alt] = React.useState(false);
  const [implants] = useApi(`/api/implants?character_id=${authContext.current.id}`);
  /*
  const handleChange = () => {
    setAlt(!alt);
  };
	*/
  const waitlist_id = queryParams.get("wl");
  if (!waitlist_id) {
    return <em>Missing waitlist information</em>;
  }

  return (
    <>
      {reviewOpen ? (
        <XupCheck waitlistId={waitlist_id} setOpen={setReviewOpen} setXupOpen={setXupOpen} />
      ) : (
        <WaitlistWrap>
          <Content style={{ flex: 1 }}>
            <h2>X-up with fit(s)</h2>
            <Textarea
              placeholder={exampleFit}
              rows={15}
              onChange={(evt) => setEft(evt.target.value)}
              value={eft}
              style={{ width: "100%", marginBottom: "1em" }}
            />
            <div style={{ marginBottom: "1em" }}>
              {/*<label>
              <input type="checkbox" checked={alt} onChange={handleChange} />
              This is an ALT (I already have a character in fleet)
		  </label>*/}
              <h2>X-up message (optional)</h2>
              <Textarea
                placeholder={exampleMessage}
                rows={1}
                onChange={(evt) => setMessagexup(evt.target.value)}
                value={messagexup}
                style={{ width: "100%" }}
              />
              Characters left: {messagexup.length < 101 ? 100 - messagexup.length : "Too long"}
            </div>
            <InputGroup>
              <Button static>{authContext.current.name}</Button>
              <Button
                variant="success"
                onClick={(evt) => {
                  setIsSubmitting(true);
                  errorToaster(
                    toastContext,
                    xUp({
                      character: authContext.current.id,
                      eft,
                      toastContext,
                      waitlist_id,
                      alt,
                      messagexup,
                    }).then((evt) => setReviewOpen(true))
                  ).finally((evt) => setIsSubmitting(false));
                }}
                disabled={
                  eft.trim().length < 50 ||
                  !eft.startsWith("[") ||
                  isSubmitting ||
                  messagexup.length > 100
                }
              >
                X-up
              </Button>
            </InputGroup>
          </Content>
          <Box style={{ flex: 1, marginTop: "1em" }}>
            {implants ? (
              <ImplantDisplay
                implants={implants.implants}
                name={`${authContext.current.name}'s capsule`}
              />
            ) : null}
          </Box>
        </WaitlistWrap>
      )}
    </>
  );
}

function XupCheck({ waitlistId, setOpen, setXupOpen }) {
  const authContext = React.useContext(AuthContext);
  const [xupData] = useApi(`/api/waitlist?waitlist_id=${waitlistId}`);

  if (!xupData) {
    return <em>Loading</em>;
  }

  const myEntry = _.find(
    xupData.waitlist,
    (entry) => entry.character && entry.character.id === authContext.account_id
  );

  return (
    <>
      <PageTitle>Fit review</PageTitle>
      <em>
        You are now on the waitlist! These are the fits you x-ed up with, please check to make sure
        you have everything and adjust your fit if needed.
      </em>
      {myEntry.fits.map((fit) => (
        <Box key={fit.id}>
          <FitDisplay fit={fit} />
        </Box>
      ))}
      <Buttons>
        <Button variant="primary" onClick={(evt) => setXupOpen(false)}>
          Yes, looks good
        </Button>
        <Button variant="secondary" onClick={(evt) => setOpen(false)}>
          No, go back to update my fit
        </Button>
      </Buttons>
    </>
  );
}
