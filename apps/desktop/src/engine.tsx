// État de connexion au moteur Rust, partagé par l'en-tête et les pages.

import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { getEngineInfo, isTauri, type EngineInfo } from "./api";

export type EngineState =
  | { kind: "loading" }
  | { kind: "ready"; info: EngineInfo }
  | { kind: "unavailable"; reason: string };

const EngineContext = createContext<EngineState>({ kind: "loading" });

export function EngineProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<EngineState>({ kind: "loading" });

  useEffect(() => {
    if (!isTauri()) {
      setState({
        kind: "unavailable",
        reason: "Interface ouverte hors de l'application de bureau : moteur non joignable.",
      });
      return;
    }
    let cancelled = false;
    getEngineInfo()
      .then((info) => {
        if (!cancelled) setState({ kind: "ready", info });
      })
      .catch((error: unknown) => {
        if (!cancelled) setState({ kind: "unavailable", reason: String(error) });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return <EngineContext.Provider value={state}>{children}</EngineContext.Provider>;
}

export function useEngine(): EngineState {
  return useContext(EngineContext);
}
