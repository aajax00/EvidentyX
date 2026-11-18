import { useEffect, useState } from "react";
import { getEngineInfo, isTauri, type EngineInfo } from "./api";
import { Badge, Panel, Spinner } from "./components/ui";
import "./app.css";

type EngineState =
  | { kind: "loading" }
  | { kind: "ready"; info: EngineInfo }
  | { kind: "unavailable"; reason: string };

const PHASES: ReadonlyArray<{ name: string; state: "en cours" | "planifiée" }> = [
  { name: "Fondations", state: "en cours" },
  { name: "Ingestion", state: "planifiée" },
  { name: "Moteur d'enquête", state: "planifiée" },
  { name: "Détection et rapports", state: "planifiée" },
  { name: "Stabilisation", state: "planifiée" },
];

export function App() {
  const [engine, setEngine] = useState<EngineState>({ kind: "loading" });

  useEffect(() => {
    if (!isTauri()) {
      setEngine({
        kind: "unavailable",
        reason: "Interface ouverte hors de l'application de bureau : moteur non joignable.",
      });
      return;
    }
    let cancelled = false;
    getEngineInfo()
      .then((info) => {
        if (!cancelled) setEngine({ kind: "ready", info });
      })
      .catch((error: unknown) => {
        if (!cancelled) setEngine({ kind: "unavailable", reason: String(error) });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="app">
      <header className="app-header">
        <span className="app-brand">EVIDENTYX</span>
        <span className="app-tagline">Chaque trace. Chaque connexion. Chaque vérité.</span>
      </header>

      <main className="app-main">
        <Panel title="Moteur d'analyse" actions={<EngineBadge state={engine} />}>
          <EngineStatus state={engine} />
        </Panel>

        <Panel title="État du projet">
          <p className="app-muted">
            Version de développement : aucune fonctionnalité d'analyse n'est encore disponible.
          </p>
          <ol className="app-phases">
            {PHASES.map((phase, index) => (
              <li key={phase.name}>
                <span>
                  {index + 1}. {phase.name}
                </span>
                <Badge tone={phase.state === "en cours" ? "accent" : "neutral"}>{phase.state}</Badge>
              </li>
            ))}
          </ol>
        </Panel>
      </main>
    </div>
  );
}

function EngineBadge({ state }: { state: EngineState }) {
  switch (state.kind) {
    case "loading":
      return <Badge>connexion…</Badge>;
    case "ready":
      return <Badge tone="success">connecté</Badge>;
    case "unavailable":
      return <Badge tone="danger">indisponible</Badge>;
  }
}

function EngineStatus({ state }: { state: EngineState }) {
  switch (state.kind) {
    case "loading":
      return (
        <p className="app-inline">
          <Spinner size="sm" label="Connexion au moteur" /> Connexion au moteur…
        </p>
      );
    case "unavailable":
      return <p className="app-error">{state.reason}</p>;
    case "ready":
      return (
        <dl className="app-facts">
          <dt>Moteur</dt>
          <dd>evidentyx-core {state.info.engineVersion}</dd>
          <dt>Application</dt>
          <dd>{state.info.appVersion}</dd>
          <dt>Plateforme</dt>
          <dd>{state.info.platform}</dd>
        </dl>
      );
  }
}
