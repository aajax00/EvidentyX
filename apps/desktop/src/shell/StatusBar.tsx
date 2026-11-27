import { useEffect, useState } from "react";
import { getStorageInfo, isDemoMode, type StorageInfo } from "../api";
import { useEngine } from "../engine";
import { useInvestigations } from "../investigations/InvestigationsProvider";

const pad = (n: number) => String(n).padStart(2, "0");
const utcClock = (d: Date) => `UTC ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}:${pad(d.getUTCSeconds())}`;

/** Barre d'état : version, base locale, nombre d'enquêtes, horloge UTC. Les anomalies seules sont colorées. */
export function StatusBar() {
  const engine = useEngine();
  const { all, revision } = useInvestigations();
  const [storage, setStorage] = useState<StorageInfo | null>(null);
  const [storageError, setStorageError] = useState<string | null>(null);
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    getStorageInfo()
      .then((info) => {
        setStorage(info);
        setStorageError(info.error);
      })
      .catch((e: { message?: string }) => setStorageError(e.message ?? String(e)));
  }, [revision]);

  useEffect(() => {
    const t = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(t);
  }, []);

  const engineText =
    engine.kind === "ready"
      ? `EvidentyX v${engine.info.appVersion} · moteur ${engine.info.engineVersion}`
      : engine.kind === "loading"
        ? "Moteur : connexion…"
        : "Moteur indisponible";

  const storageText = storageError
    ? "Base indisponible"
    : isDemoMode()
      ? "Données de démonstration (mémoire du navigateur)"
      : storage
        ? `Base SQLite locale · schéma ${storage.schemaVersion ?? "?"} · ${all.length} enquête${all.length > 1 ? "s" : ""}`
        : "Base : ouverture…";

  return (
    <footer className="shell-statusbar">
      <div className="shell-statusbar__group">
        <span
          className={`shell-statusbar__item mono${engine.kind === "unavailable" && !isDemoMode() ? " shell-statusbar__item--danger" : ""}`}
          title={engine.kind === "ready" ? `Plateforme : ${engine.info.platform}` : engine.kind === "unavailable" ? engine.reason : undefined}
        >
          {engineText}
        </span>
        <span className="shell-statusbar__sep" aria-hidden="true" />
        <span
          className={`shell-statusbar__item mono${storageError ? " shell-statusbar__item--danger" : isDemoMode() ? " shell-statusbar__item--warning" : ""}`}
          title={storageError ?? storage?.path}
        >
          {storageText}
        </span>
      </div>
      <div className="shell-statusbar__group">
        <span className="shell-statusbar__item shell-statusbar__item--secondary">Hors ligne · aucune télémétrie</span>
        <span className="shell-statusbar__sep shell-statusbar__item--secondary" aria-hidden="true" />
        <span className="shell-statusbar__item mono">{utcClock(now)}</span>
      </div>
    </footer>
  );
}
