import { ArrowRight, FolderSearch, Plus } from "lucide-react";
import { Badge, Button, EmptyState, Panel, Spinner } from "../components/ui";
import { formatRelative } from "../format";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { useRouter } from "../shell/router";

type PhaseState = "terminée" | "prochaine" | "planifiée";

const PHASE_TONES = { terminée: "success", prochaine: "accent", planifiée: "neutral" } as const;

const PHASES: ReadonlyArray<{ name: string; content: string; state: PhaseState }> = [
  { name: "Fondations", content: "Application, design, enquêtes, stockage", state: "terminée" },
  { name: "Ingestion", content: "JSON, CSV, Syslog, Apache/Nginx, PCAP, SHA-256", state: "terminée" },
  { name: "Moteur d'enquête", content: "Normalisation, chronologie, corrélation, graphe", state: "prochaine" },
  { name: "Détection et rapports", content: "Règles, coffre de preuves, exports", state: "planifiée" },
  { name: "Stabilisation", content: "Tests, sécurité, performances, paquets", state: "planifiée" },
];

export function HomePage() {
  const { all, loading, error, openCreate } = useInvestigations();
  const { navigate } = useRouter();
  const active = all.filter((i) => i.status === "active");
  const recent = active.slice(0, 6);

  const archived = all.length - active.length;
  const latest = all.reduce<number | null>((t, i) => (t === null || i.updatedAt > t ? i.updatedAt : t), null);
  const dash = loading ? "—" : null;

  const kpis = [
    { label: "Enquêtes actives", value: dash ?? String(active.length), total: dash ? null : all.length, sub: "En cours d'analyse" },
    { label: "Archivées", value: dash ?? String(archived), total: null, sub: "En lecture seule" },
    { label: "Total", value: dash ?? String(all.length), total: null, sub: "Dans la base locale" },
    { label: "Dernière activité", value: dash ?? (latest === null ? "—" : formatRelative(latest)), total: null, sub: "Modification d'une enquête" },
  ];

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h2 className="page-header__title">Tableau de bord</h2>
          <p className="page-header__desc">Synthèse des enquêtes en cours et de l'avancement du projet. Tout reste sur cette machine.</p>
        </div>
        <div className="page-header__actions">
          <Button icon={FolderSearch} onClick={() => navigate({ page: "investigations" })}>
            Voir les enquêtes
          </Button>
          <Button variant="primary" icon={Plus} onClick={openCreate}>
            Nouvelle enquête
          </Button>
        </div>
      </header>

      <div className="kpi-grid" role="list" aria-label="Chiffres clés">
        {kpis.map((k) => (
          <div key={k.label} className="kpi-card" role="listitem">
            <span className="section-title">{k.label}</span>
            <span className="kpi-card__value mono">
              {k.value}
              {k.total !== null && <span className="kpi-card__total"> / {k.total}</span>}
            </span>
            <span className="kpi-card__sub">{k.sub}</span>
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
                <Badge tone={PHASE_TONES[phase.state]}>{phase.state}</Badge>
              </li>
            ))}
          </ol>
        </Panel>
      </div>
    </div>
  );
}
