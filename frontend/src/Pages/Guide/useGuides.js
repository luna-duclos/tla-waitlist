import React from "react";

export function guidePath(guide) {
  return guide.section === "fc" ? `/fc/${guide.slug}` : `/guide/${guide.slug}`;
}

export function useGuides() {
  const [guides, setGuides] = React.useState([]);
  const [loading, setLoading] = React.useState(true);

  React.useEffect(() => {
    let cancelled = false;

    fetch("/api/v2/guides")
      .then((response) => {
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        return response.json();
      })
      .then((data) => {
        if (cancelled) return;
        setGuides(Array.isArray(data.guides) ? data.guides : []);
      })
      .catch(() => {
        if (!cancelled) {
          setGuides([]);
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  const publicGuides = React.useMemo(
    () => guides.filter((g) => g.section !== "fc"),
    [guides]
  );
  const fcGuides = React.useMemo(
    () => guides.filter((g) => g.section === "fc"),
    [guides]
  );

  return { guides, publicGuides, fcGuides, loading };
}
