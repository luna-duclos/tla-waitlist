import { useApi } from "../../api";
import { InputGroup, Button, Buttons } from "../../Components/Form";
import { Fitout, ImplantOut } from "./FittingSortDisplay";
import { PageTitle } from "../../Components/Page";
import { useLocation, useHistory } from "react-router-dom";
import { usePageTitle } from "../../Util/title";

export function Fits() {
  const queryParams = new URLSearchParams(useLocation().search);
  const history = useHistory();
  var tier = queryParams.get("Tier") || "Armor";
  const setTier = (newTier) => {
    queryParams.set("Tier", newTier);
    history.push({
      search: queryParams.toString(),
    });
  };

  return <FitsDisplay tier={tier} setTier={setTier} />;
}

function FitsDisplay({ tier, setTier = null }) {
  usePageTitle(`${tier} Fits`);
  const [fitData] = useApi(`/api/fittings`);
  if (fitData === null) {
    return <em>Loading fits...</em>;
  }

  return (
    <>
      <PageTitle>FITS</PageTitle>
      {setTier != null && (
        <Buttons style={{ marginBottom: "0.5em" }}>
          <InputGroup>
            <Button active={tier === "Armor"} onClick={(evt) => setTier("Armor")}>
              Armor
            </Button>

            <Button active={tier === "Shield"} onClick={(evt) => setTier("Shield")}>
              Shield
            </Button>
          </InputGroup>
        </Buttons>
      )}
      <ImplantOut />
      {tier === "Armor" ? (
        <Fitout data={fitData} tier="Armor" />
      ) : tier === "Shield" ? (
        <Fitout data={fitData} tier="Shield" />
      ) : tier === "Advanced" ? (
        <Fitout data={fitData} tier="Advanced" />
      ) : tier === "Elite" ? (
        <Fitout data={fitData} tier="Elite" />
      ) : tier === "Other" ? (
        <Fitout data={fitData} tier="Other" />
      ) : tier === "Antigank" ? (
        <Fitout data={fitData} tier="Antigank" />
      ) : null}
    </>
  );
}
