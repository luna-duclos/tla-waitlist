import { useCallback, useContext } from "react";
import { ThemeContext } from "styled-components";

export function useStatusStyle() {
  const theme = useContext(ThemeContext);

  return useCallback(
    (status) => {
      switch (status) {
        case "pending":
          return { backgroundColor: theme.colors.warning.color, color: theme.colors.warning.text };
        case "approved":
          return { backgroundColor: theme.colors.success.color, color: theme.colors.success.text };
        case "rejected":
          return { backgroundColor: theme.colors.danger.color, color: theme.colors.danger.text };
        case "paid":
          return { backgroundColor: theme.colors.primary.color, color: theme.colors.primary.text };
        default:
          return {
            backgroundColor: theme.colors.secondary.color,
            color: theme.colors.secondary.text,
          };
      }
    },
    [theme]
  );
}
