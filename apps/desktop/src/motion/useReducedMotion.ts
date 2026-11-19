import { useEffect, useState } from "react";

const QUERY = "(prefers-reduced-motion: reduce)";

/** Vrai si le système demande de limiter les animations : elles sont alors désactivées. */
export function useReducedMotion(): boolean {
  const [reduced, setReduced] = useState(() => window.matchMedia(QUERY).matches);
  useEffect(() => {
    const media = window.matchMedia(QUERY);
    const onChange = () => setReduced(media.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);
  return reduced;
}
