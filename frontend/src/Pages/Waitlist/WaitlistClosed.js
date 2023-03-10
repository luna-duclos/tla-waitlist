import React from "react";
import { CenteredParagraph } from "../../Components/Form";
import styled, { keyframes } from "styled-components";
import { Button, InputGroup } from "../../Components/Form";

const spin = keyframes`
  to {
    transform: rotate(360deg);
  }
`;

const LoadingSpinnerContainer = styled.div`
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
`;

const LoadingSpinnerCircle = styled.div`
  width: 4rem;
  height: 4rem;
  border-radius: 50%;
  border: 0.2rem solid ${(props) => props.theme.colors.accent1};
  border-top-color: ${(props) => props.theme.colors.text};
  animation: ${spin} 1.5s linear infinite;
  box-shadow: 0px -12px 8px -5px ${(props) => props.theme.colors.text}40;
`;

export default function WaitlistClosed() {
  const [showVideo, setShowVideo] = React.useState(false);
  return (
    <CenteredParagraph style={{ width: "100%" }}>
      <CenteredParagraph.Head style={{ marginBottom: "1em" }}>
        We apologize, the waitlist is currently closed.
      </CenteredParagraph.Head>
      <CenteredParagraph.Paragraph style={{ width: "50%" }}>
        Thank you for your patience. A fleet commander will be available shortly. Please stand by...
      </CenteredParagraph.Paragraph>
      <CenteredParagraph.Paragraph style={{ width: "50%" }}>
        <LoadingSpinnerContainer>
          <LoadingSpinnerCircle />
        </LoadingSpinnerContainer>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginTop: "1.5em",
          }}
        >
          <InputGroup>
            <Button onClick={(evt) => setShowVideo(true)}>
              Click here to Ping FC&apos;s for Fleet
            </Button>
          </InputGroup>
        </div>
      </CenteredParagraph.Paragraph>
      {showVideo && (
        <iframe
          title="Never Gonna Give You Up"
          width="560"
          height="315"
          src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?autoplay=1&controls=0"
          frameBorder="0"
          allow="autoplay; encrypted-media; picture-in-picture"
          allowFullScreen
        />
      )}
    </CenteredParagraph>
  );
}
