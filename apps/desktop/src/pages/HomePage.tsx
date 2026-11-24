import { useEffect, useState } from "react";
import { Activity, ArrowRight, FileStack, FolderOpen, FolderSearch, Plus, ShieldCheck, type LucideIcon } from "lucide-react";
import { getTotals, type Totals } from "../api";
import { Badge, Button, EmptyState, Panel, Spinner } from "../components/ui";
import { formatRelative } from "../format";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { TerrainBanner } from "../motion/TerrainBanner";
import { useReducedMotion } from "../motion/useReducedMotion";
import { useSettings } from "../settings/SettingsProvider";
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

const number = new Intl.NumberFormat("fr-FR");

interface Kpi {
  label: string;
  value: string;
  hint?: string | undefined;
  icon: LucideIcon;
  tone: "blue" | "violet" | "cyan" | "green";
}

export function HomePage() {
  const { all, loading, error, openCreate, revision } = useInvestigations();
  const { navigate } = useRouter();
  const { settings } = useSettings();
  const reducedMotion = useReducedMotion();
  const [totals, setTotals] = useState<Totals | null>(null);
  const active = all.filter((i) => i.status === "active");
  const recent = active.slice(0, 6);

  useEffect(() => {
    getTotals()
      .then(setTotals)
      .catch(() => setTotals(null));
  }, [revision]);

  const dash = "—";
  const kpis: Kpi[] = [
    {
      label: "Enquêtes actives",
      value: loading ? dash : number.format(active.length),
      hint: `${number.format(all.length - active.length)} archivée${all.length - active.length > 1 ? "s" : ""}`,
      icon: FolderOpen,
      tone: "blue",
    },
    { label: "Événements extraits", value: totals ? number.format(totals.events) : dash, icon: Activity, tone: "violet" },
    { label: "Preuves enregistrées", value: totals ? number.format(totals.evidence) : dash, icon: FileStack, tone: "cyan" },
    {
      label: "Preuves intègres",
      value: totals && totals.evidence > 0 ? `${Math.floor((totals.evidenceIntact / totals.evidence) * 100)} %` : dash,
      hint: totals && totals.evidence > 0 ? `${totals.evidenceIntact} sur ${totals.evidence} · dernière vérification` : undefined,
      icon: ShieldCheck,
      tone: "green",
    },
  ];

  return (
    <div className="page page--wide">
      <section className="dash-hero">
        <TerrainBanner className="dash-hero__terrain" animated={settings.backgroundAnimation && !reducedMotion} />
        <div className="dash-hero__body">
          <p className="dash-hero__eyebrow">Vue d'ensemble</p>
          <h2 className="dash-hero__title">Tableau de bord</h2>
          <p className="dash-hero__text">
            Vos enquêtes et leurs preuves, analysées localement : rien ne quitte cette machine.
          </p>
          <div className="dash-hero__actions">
            <Button variant="primary" icon={Plus} onClick={openCreate}>
              Nouvelle enquête
            </Button>
            <Button icon={FolderSearch} onClick={() => navigate({ page: "investigations" })}>
              Voir les enquêtes
            </Button>
          </div>
        </div>
      </section>

      <div className="kpi-grid" role="list" aria-label="Chiffres clés">
        {kpis.map((k) => (
          <div key={k.label} className={`kpi kpi--${k.tone}`} role="listitem">
            <span className="kpi__icon" aria-hidden="true">
              <k.icon size={20} />
            </span>
            <span className="kpi__text">
              <span className="kpi__value mono">{k.value}</span>
              <span className="kpi__label">{k.label}</span>
              {k.hint && <span className="kpi__hint">{k.hint}</span>}
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
                    <span className="recent-list__icon" aria-hidden="true">
                      <FolderOpen size={16} />
                    </span>
                    <span className="recent-list__main">
                      <span className="recent-list__name">{inv.name}</span>
                      {inv.description && <span className="recent-list__desc">{inv.description}</span>}
                    </span>
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
