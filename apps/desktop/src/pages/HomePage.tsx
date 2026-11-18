import { Badge, Panel } from "../components/ui";

const PHASES: ReadonlyArray<{ name: string; content: string; state: "en cours" | "planifiée" }> = [
  { name: "Fondations", content: "Application, design, enquêtes, stockage", state: "en cours" },
  { name: "Ingestion", content: "JSON, CSV, Syslog, Apache/Nginx, PCAP, SHA-256", state: "planifiée" },
  { name: "Moteur d'enquête", content: "Normalisation, chronologie, corrélation, graphe", state: "planifiée" },
  { name: "Détection et rapports", content: "Règles, coffre de preuves, exports", state: "planifiée" },
  { name: "Stabilisation", content: "Tests, sécurité, performances, paquets", state: "planifiée" },
];

export function HomePage() {
  return (
    <div className="page">
      <Panel title="Bienvenue">
        <p className="page-lead">
          EVIDENTYX transforme des preuves numériques fragmentées en enquêtes structurées,
          chronologiques et explicables — entièrement hors ligne.
        </p>
        <p className="page-muted">
          Version de développement : aucune fonctionnalité d'analyse n'est encore disponible. Les
          sections grisées de la barre latérale sont planifiées.
        </p>
      </Panel>

      <Panel title="Feuille de route">
        <ol className="page-phases">
          {PHASES.map((phase, index) => (
            <li key={phase.name}>
              <span className="page-phases__index mono">{String(index + 1).padStart(2, "0")}</span>
              <span className="page-phases__name">{phase.name}</span>
              <span className="page-phases__content">{phase.content}</span>
              <Badge tone={phase.state === "en cours" ? "accent" : "neutral"}>{phase.state}</Badge>
            </li>
          ))}
        </ol>
      </Panel>
    </div>
  );
}
