import React from "react";
import { CenteredParagraph } from "../../Components/Form";
import styled, { keyframes } from "styled-components";
import { Button } from "../../Components/Form";

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
  width: 5rem;
  height: 5rem;
  border-radius: 50%;
  border: 0.2rem solid ${(props) => props.theme.colors.accent1};
  border-top-color: ${(props) => props.theme.colors.text};
  animation: ${spin} 1s linear infinite;
`;

export default function WaitlistClosed() {
  const [showVideo, setShowVideo] = React.useState(false);
  return (
    <CenteredParagraph style={{ width: "100%" }}>
      <CenteredParagraph.Head style={{ marginBottom: "1em" }}>
        We apologize, the waitlist is currently closed.
      </CenteredParagraph.Head>
      <CenteredParagraph.Paragraph style={{ width: "50%" }}>
        Thank you for your patience. A fleet commander will be available shortly. Please stand by
      </CenteredParagraph.Paragraph>
      <CenteredParagraph.Paragraph style={{ width: "50%" }}>
        <LoadingSpinnerContainer>
          <LoadingSpinnerCircle />
        </LoadingSpinnerContainer>
        <Button style={{ marginTop: "1.5em" }} onClick={(evt) => setShowVideo(true)}>
          Click here to Ping for Fleet
        </Button>
      </CenteredParagraph.Paragraph>
      {showVideo && (
        <iframe
          title="Never Gonna Give You Up"
          width="560"
          height="315"
          src="https://www.youtube.com/embed/dQw4w9WgXcQ?autoplay=1"
          frameBorder="0"
          allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
          allowFullScreen
        />
      )}
    </CenteredParagraph>
  );
}
