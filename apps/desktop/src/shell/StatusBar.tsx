import { useEffect, useState } from "react";
import { getStorageInfo, isDemoMode, type StorageInfo } from "../api";
import { useEngine } from "../engine";
import { useInvestigations } from "../investigations/InvestigationsProvider";

/** Barre d'état : moteur, base de données, nombre d'enquêtes. */
export function StatusBar() {
  const engine = useEngine();
  const { all, revision } = useInvestigations();
  const [storage, setStorage] = useState<StorageInfo | null>(null);
  const [storageError, setStorageError] = useState<string | null>(null);

  useEffect(() => {
    getStorageInfo()
      .then((info) => {
        setStorage(info);
        setStorageError(info.error);
      })
      .catch((e: { message?: string }) => setStorageError(e.message ?? String(e)));
  }, [revision]);

  const engineItem =
    engine.kind === "ready"
      ? { tone: "success", text: `Moteur ${engine.info.engineVersion}`, title: `Plateforme : ${engine.info.platform}` }
      : engine.kind === "loading"
        ? { tone: "neutral", text: "Moteur : connexion…", title: undefined }
        : { tone: isDemoMode() ? "warning" : "danger", text: "Moteur indisponible", title: engine.reason };

  const storageItem = storageError
    ? { tone: "danger", text: "Base indisponible", title: storageError }
    : storage
      ? {
          tone: isDemoMode() ? "warning" : "success",
          text: isDemoMode() ? "Données de démonstration" : `Base locale · schéma ${storage.schemaVersion ?? "?"}`,
          title: storage.path,
        }
      : { tone: "neutral", text: "Base : ouverture…", title: undefined };

  return (
    <footer className="shell-statusbar">
      <StatusItem {...engineItem} />
      <StatusItem {...storageItem} />
      <span className="shell-statusbar__spacer" />
      <span className="shell-statusbar__item">
        {all.length} enquête{all.length > 1 ? "s" : ""}
      </span>
      <span className="shell-statusbar__item shell-statusbar__item--secondary">Hors ligne · aucune télémétrie</span>
    </footer>
  );
}

function StatusItem({ tone, text, title }: { tone: string; text: string; title: string | undefined }) {
  return (
    <span className={`shell-statusbar__item shell-statusbar__item--${tone}`} title={title}>
      <span className="shell-statusbar__dot" aria-hidden="true" />
      {text}
    </span>
  );
}
