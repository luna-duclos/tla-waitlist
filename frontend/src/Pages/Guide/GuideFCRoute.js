import React from "react";
import { useParams } from "react-router-dom";
import { AuthContext } from "../../contexts";
import { E401, E403, E404 } from "../Errors";
import { GuideViewer } from "./GuideViewer";
import { useGuides } from "./useGuides";

export function GuideFCRoute() {
  const { guideName } = useParams();
  const authContext = React.useContext(AuthContext);
  const { guides, loading } = useGuides();
  const guide = guides.find((g) => g.slug === guideName && g.section === "fc");

  if (loading) {
    return <em>Loading...</em>;
  }

  if (!authContext) {
    return <E401 />;
  }

  if (!guide) {
    return <E404 />;
  }

  const requiredAccess = guide.access ?? "fleet-view";
  if (!authContext.access[requiredAccess]) {
    return <E403 />;
  }

  return <GuideViewer slug={guideName} />;
}
