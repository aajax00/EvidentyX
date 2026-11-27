import { useEffect, useState } from "react";
import { FolderSearch, Plus } from "lucide-react";
import { getDashboard, toCommandError, type Dashboard } from "../api";
import { Badge, Button, EmptyState, Panel, Spinner } from "../components/ui";
import { formatBytes, formatRelative } from "../format";
import { StatusBadge } from "../investigations/StatusBadge";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { useRouter } from "../shell/router";

type PhaseState = "terminée" | "en cours" | "planifiée";

const PHASE_TONES = { terminée: "success", "en cours": "accent", planifiée: "neutral" } as const;

const PHASES: ReadonlyArray<{ name: string; content: string; state: PhaseState }> = [
  { name: "Fondations", content: "Application, design, enquêtes, stockage", state: "terminée" },
  { name: "Ingestion", content: "JSON, CSV, Syslog, Apache/Nginx, PCAP, SHA-256", state: "terminée" },
  { name: "Moteur d'enquête", content: "Chronologie, filtres, corrélation, graphe", state: "terminée" },
  { name: "Détection et rapports", content: "Règles, coffre de preuves, exports", state: "en cours" },
  { name: "Stabilisation", content: "Tests, sécurité, performances, paquets", state: "planifiée" },
];

const INTEGRITY = {
  intact: { label: "Intègre", tone: "success" },
  modified: { label: "Modifiée", tone: "danger" },
  missing: { label: "Introuvable", tone: "warning" },
} as const;

const number = new Intl.NumberFormat("fr-FR");

export function HomePage() {
  const { all, loading, error, openCreate, revision } = useInvestigations();
  const { navigate } = useRouter();
  const [dashboard, setDashboard] = useState<Dashboard | null>(null);
  const [dashError, setDashError] = useState<string | null>(null);

  useEffect(() => {
    getDashboard()
      .then(setDashboard)
      .catch((e: unknown) => setDashError(toCommandError(e).message));
  }, [revision]);

  const active = all.filter((i) => i.status === "active");
  const figures = new Map(dashboard?.figures.map((f) => [f.investigationId, f]));
  const sum = (k: "evidence" | "evidenceIntact" | "evidenceAnalyzed" | "events") =>
    dashboard?.figures.reduce((a, f) => a + f[k], 0) ?? null;
  const evidence = sum("evidence");
  const intact = sum("evidenceIntact");
  const analyzed = sum("evidenceAnalyzed");
  const events = sum("events");
  const withEvents = dashboard?.figures.filter((f) => f.events > 0).length ?? 0;
  const latest = all.reduce<number | null>((t, i) => (t === null || i.updatedAt > t ? i.updatedAt : t), null);
  const fmt = (v: number | null) => (v === null ? "—" : number.format(v));
  const rows = [...all].sort((a, b) => b.updatedAt - a.updatedAt).slice(0, 8);

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h2 className="page-header__title">Tableau de bord</h2>
          <p className="page-header__desc">Synthèse des enquêtes en cours, des preuves et de l'activité. Tout reste sur cette machine.</p>
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
        <div className="kpi-card" role="listitem">
          <span className="section-title">Enquêtes actives</span>
          <span className="kpi-card__value mono">
            {loading ? "—" : active.length}
            <span className="kpi-card__total"> / {loading ? "—" : all.length}</span>
          </span>
          <span className="kpi-card__sub">
            {all.length - active.length} archivée{all.length - active.length > 1 ? "s" : ""}
          </span>
        </div>
        <div className="kpi-card" role="listitem">
          <span className="section-title">Preuves enregistrées</span>
          <span className="kpi-card__value mono">{fmt(evidence)}</span>
          <span className="kpi-card__sub">
            {fmt(intact)} intègre{(intact ?? 0) > 1 ? "s" : ""} · {fmt(analyzed)} analysée{(analyzed ?? 0) > 1 ? "s" : ""}
          </span>
        </div>
        <div className="kpi-card" role="listitem">
          <span className="section-title">Événements extraits</span>
          <span className="kpi-card__value kpi-card__value--accent mono">{fmt(events)}</span>
          <span className="kpi-card__sub">
            dans {withEvents} enquête{withEvents > 1 ? "s" : ""}
          </span>
        </div>
        <div className="kpi-card" role="listitem">
          <span className="section-title">Dernière activité</span>
          <span className="kpi-card__value kpi-card__value--text">{latest === null ? "—" : formatRelative(latest)}</span>
          <span className="kpi-card__sub">Modification d'une enquête</span>
        </div>
      </div>

      <div className="home-grid">
        <Panel
          title="Enquêtes"
          className="ui-panel--flush"
          actions={
            all.length > 0 && (
              <Button size="sm" variant="ghost" onClick={() => navigate({ page: "investigations" })}>
                Voir tout
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
          ) : rows.length === 0 ? (
            <EmptyState
              icon={FolderSearch}
              title="Aucune enquête"
              action={
                <Button variant="primary" icon={Plus} onClick={openCreate}>
                  Créer une enquête
                </Button>
              }
            >
              Créez une enquête pour commencer à structurer un incident.
            </EmptyState>
          ) : (
            <table className="data-table dash-table">
              <thead>
                <tr>
                  <th scope="col" className="dash-table__ref">Réf.</th>
                  <th scope="col">Enquête</th>
                  <th scope="col" className="dash-table__status">Statut</th>
                  <th scope="col" className="dash-table__num">Preuves</th>
                  <th scope="col" className="dash-table__num">Événements</th>
                  <th scope="col" className="dash-table__date">Mise à jour</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((inv) => {
                  const f = figures.get(inv.id);
                  return (
                    <tr key={inv.id} className="dash-table__row" onClick={() => navigate({ page: "investigation", id: inv.id })}>
                      <td className="mono dash-table__ref-cell">#{inv.id}</td>
                      <td>
                        <button
                          type="button"
                          className="dash-table__name"
                          onClick={(e) => {
                            e.stopPropagation();
                            navigate({ page: "investigation", id: inv.id });
                          }}
                        >
                          {inv.name}
                        </button>
                      </td>
                      <td>
                        <StatusBadge status={inv.status} />
                      </td>
                      <td className="mono dash-table__num">{f ? number.format(f.evidence) : "—"}</td>
                      <td className="mono dash-table__num">{f ? number.format(f.events) : "—"}</td>
                      <td className="mono dash-table__muted">{formatRelative(inv.updatedAt)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
        </Panel>

        <div className="home-side">
          <Panel title="Preuves récentes" className="ui-panel--flush">
            {dashError ? (
              <p className="page-error dash-pad">{dashError}</p>
            ) : !dashboard ? (
              <p className="page-inline dash-pad">
                <Spinner size="sm" /> Chargement…
              </p>
            ) : dashboard.recentEvidence.length === 0 ? (
              <p className="text-muted dash-pad">Aucune preuve ajoutée pour l'instant.</p>
            ) : (
              <ul className="dash-activity">
                {dashboard.recentEvidence.map((r) => (
                  <li key={r.evidenceId}>
                    <button type="button" onClick={() => navigate({ page: "investigation", id: r.investigationId })}>
                      <span className="dash-activity__meta">
                        <Badge tone={INTEGRITY[r.integrity].tone}>{INTEGRITY[r.integrity].label}</Badge>
                        <span className="mono dash-activity__case">{r.investigationName}</span>
                      </span>
                      <span className="dash-activity__line">
                        <span className="dash-activity__file mono">{r.fileName}</span>
                        <span className="mono dash-activity__time">
                          {formatBytes(r.sizeBytes)} · {formatRelative(r.addedAt)}
                        </span>
                      </span>
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
    </div>
  );
}
