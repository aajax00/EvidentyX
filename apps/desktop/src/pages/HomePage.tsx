import { Archive, ArrowRight, FolderOpen, FolderSearch, Layers, Plus } from "lucide-react";
import { Badge, Button, EmptyState, Panel, Spinner } from "../components/ui";
import { formatRelative } from "../format";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { BrandLogo } from "../shell/BrandLogo";
import { useRouter } from "../shell/router";

const PHASES: ReadonlyArray<{ name: string; content: string; state: "en cours" | "planifiée" }> = [
  { name: "Fondations", content: "Application, design, enquêtes, stockage", state: "en cours" },
  { name: "Ingestion", content: "JSON, CSV, Syslog, Apache/Nginx, PCAP, SHA-256", state: "planifiée" },
  { name: "Moteur d'enquête", content: "Normalisation, chronologie, corrélation, graphe", state: "planifiée" },
  { name: "Détection et rapports", content: "Règles, coffre de preuves, exports", state: "planifiée" },
  { name: "Stabilisation", content: "Tests, sécurité, performances, paquets", state: "planifiée" },
];

export function HomePage() {
  const { all, loading, error, openCreate } = useInvestigations();
  const { navigate } = useRouter();
  const active = all.filter((i) => i.status === "active");
  const recent = active.slice(0, 6);

  const stats = [
    { label: "Enquêtes actives", value: active.length, icon: FolderOpen },
    { label: "Archivées", value: all.length - active.length, icon: Archive },
    { label: "Total", value: all.length, icon: Layers },
  ];

  return (
    <div className="page">
      <section className="hero">
        <BrandLogo className="hero__logo" />
        <div className="hero__body">
          <p className="hero__eyebrow">Plateforme d'investigation numérique</p>
          <h2 className="hero__title">Chaque trace. Chaque connexion. Chaque vérité.</h2>
          <p className="hero__text">
            Transformez des preuves numériques fragmentées en enquêtes structurées, chronologiques et explicables —
            entièrement hors ligne.
          </p>
          <div className="hero__actions">
            <Button variant="primary" icon={Plus} onClick={openCreate}>
              Nouvelle enquête
            </Button>
            <Button icon={FolderSearch} onClick={() => navigate({ page: "investigations" })}>
              Voir les enquêtes
            </Button>
          </div>
        </div>
      </section>

      <div className="stat-grid" role="list" aria-label="Statistiques">
        {stats.map((s) => (
          <div key={s.label} className="stat" role="listitem">
            <s.icon size={18} aria-hidden="true" className="stat__icon" />
            <span className="stat__text">
              <span className="stat__value mono">{loading ? "—" : s.value}</span>
              <span className="stat__label">{s.label}</span>
            </span>
          </div>
        ))}
      </div>

      <div className="home-grid">
        <Panel
          title="Reprendre une enquête"
          actions={
            all.length > 0 && (
              <Button size="sm" variant="ghost" onClick={() => navigate({ page: "investigations" })}>
                Tout afficher
              </Button>
            )
          }
        >
          {error ? (
            <p className="page-error">{error.message}</p>
          ) : loading ? (
            <p className="page-inline">
              <Spinner size="sm" /> Chargement…
            </p>
          ) : recent.length === 0 ? (
            <EmptyState
              icon={FolderSearch}
              title="Aucune enquête active"
              action={
                <Button variant="primary" icon={Plus} onClick={openCreate}>
                  Créer une enquête
                </Button>
              }
            >
              Créez une enquête pour commencer à structurer un incident.
            </EmptyState>
          ) : (
            <ul className="recent-list">
              {recent.map((inv) => (
                <li key={inv.id}>
                  <button
                    type="button"
                    className="recent-list__item"
                    onClick={() => navigate({ page: "investigation", id: inv.id })}
                  >
                    <span className="recent-list__name">{inv.name}</span>
                    <span className="recent-list__date">{formatRelative(inv.updatedAt)}</span>
                    <ArrowRight size={14} aria-hidden="true" className="recent-list__arrow" />
                  </button>
                </li>
              ))}
            </ul>
          )}
        </Panel>

        <Panel title="Feuille de route">
          <ol className="page-phases">
            {PHASES.map((phase, index) => (
              <li key={phase.name}>
                <span className="page-phases__index mono">{String(index + 1).padStart(2, "0")}</span>
                <span>
                  <span className="page-phases__name">{phase.name}</span>
                  <span className="page-phases__content">{phase.content}</span>
                </span>
                <Badge tone={phase.state === "en cours" ? "accent" : "neutral"}>{phase.state}</Badge>
              </li>
            ))}
          </ol>
        </Panel>
      </div>
    </div>
  );
}
