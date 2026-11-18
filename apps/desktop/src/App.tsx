import { useEffect, useState } from "react";
import { getEngineInfo, isTauri, type EngineInfo } from "./api";

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
        <span className="brand">EVIDENTYX</span>
        <span className="tagline">Chaque trace. Chaque connexion. Chaque vérité.</span>
      </header>

      <main className="app-main">
        <section className="panel" aria-labelledby="engine-title">
          <h2 id="engine-title">Moteur d'analyse</h2>
          <EngineStatus state={engine} />
        </section>

        <section className="panel" aria-labelledby="roadmap-title">
          <h2 id="roadmap-title">État du projet</h2>
          <p className="muted">
            Version de développement : aucune fonctionnalité d'analyse n'est encore disponible.
          </p>
          <ol className="phases">
            {PHASES.map((phase, index) => (
              <li key={phase.name}>
                <span>
                  {index + 1}. {phase.name}
                </span>
                <span className={`badge badge-${phase.state === "en cours" ? "active" : "planned"}`}>
                  {phase.state}
                </span>
              </li>
            ))}
          </ol>
        </section>
      </main>
    </div>
  );
}

function EngineStatus({ state }: { state: EngineState }) {
  switch (state.kind) {
    case "loading":
      return <p className="muted">Connexion au moteur…</p>;
    case "unavailable":
      return <p className="error">{state.reason}</p>;
    case "ready":
      return (
        <dl className="facts">
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
