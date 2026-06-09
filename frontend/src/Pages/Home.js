import { NavButton, InputGroup } from "../Components/Form";
import { useTranslation } from "react-i18next";
import { HomeContent } from "./Home/HomeContent";

export function Home() {
  const { t } = useTranslation("home");

  return (
    <>
      <InputGroup style={{ marginTop: "5em", alignItems: "center", gap: "0.75em" }}>
        <NavButton to="/legal">{t("legal")}</NavButton>
      </InputGroup>
      <HomeContent />
    </>
  );
}
