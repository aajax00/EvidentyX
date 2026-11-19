// Réglages de l'application : chargés au démarrage, enregistrés à chaque modification.

import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { getSettings, toCommandError, updateSettings, type CommandError, type Settings } from "../api";

/** Valeurs utilisées tant que les réglages ne sont pas lus (ou si la base est indisponible). */
export const DEFAULT_SETTINGS: Settings = {
  splashScreen: true,
  backgroundAnimation: true,
  sidebarCollapsed: false,
  defaultSort: "updated_desc",
  defaultStatusFilter: "active",
};

interface SettingsApi {
  settings: Settings;
  /** Vrai une fois la lecture terminée (réussie ou non). */
  loaded: boolean;
  /** Modifie un ou plusieurs réglages et les enregistre ; rejette en cas d'échec. */
  update: (changes: Partial<Settings>) => Promise<void>;
  error: CommandError | null;
}

const SettingsContext = createContext<SettingsApi | null>(null);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<CommandError | null>(null);

  useEffect(() => {
    getSettings()
      .then(setSettings)
      .catch((e: unknown) => setError(toCommandError(e)))
      .finally(() => setLoaded(true));
  }, []);

  const update = useCallback(
    async (changes: Partial<Settings>) => {
      const previous = settings;
      const next = { ...settings, ...changes };
      setSettings(next); // affichage immédiat, annulé si l'enregistrement échoue
      try {
        setSettings(await updateSettings(next));
        setError(null);
      } catch (e) {
        setSettings(previous);
        const err = toCommandError(e);
        setError(err);
        throw err;
      }
    },
    [settings],
  );

  const api = useMemo(() => ({ settings, loaded, update, error }), [settings, loaded, update, error]);
  return <SettingsContext.Provider value={api}>{children}</SettingsContext.Provider>;
}

export function useSettings(): SettingsApi {
  const api = useContext(SettingsContext);
  if (!api) throw new Error("useSettings doit être utilisé sous <SettingsProvider>");
  return api;
}
