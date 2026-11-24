import { useEffect, useState } from "react";
import { Activity, BarChart3, FileStack, Globe, Users } from "lucide-react";
import { getOverview, toCommandError, type Overview } from "../api";
import { EmptyState, Panel, Spinner } from "../components/ui";
import { formatDateTime } from "../format";
import "./overview.css";

const number = new Intl.NumberFormat("fr-FR");

const KIND_LABELS: Record<string, string> = {
  authentication: "Authentification",
  network: "Réseau",
  web: "Web",
  process: "Processus",
  file: "Fichier",
  system: "Système",
  other: "Autre",
};

function plural(n: number, word: string) {
  return `${number.format(n)} ${word}${n > 1 ? "s" : ""}`;
}

/** Barres horizontales relatives au plus grand nombre. */
function Bars({ items }: { items: Array<{ key: string; label: string; value: number }> }) {
  const max = items[0]?.value ?? 1;
  return (
    <ul className="overview-sources">
      {items.map((s) => (
        <li key={s.key}>
          <span className="overview-sources__name mono" title={s.label}>
            {s.label}
          </span>
          <span className="overview-sources__track" aria-hidden="true">
            <span className="overview-sources__fill" style={{ width: `${(s.value / max) * 100}%` }} />
          </span>
          <span className="overview-sources__value mono">{number.format(s.value)}</span>
        </li>
      ))}
    </ul>
  );
}

/**
 * Vue d'ensemble d'une enquête : statistiques des événements extraits des
 * preuves analysées. `version` change quand les preuves changent (rechargement).
 */
export function OverviewPanel({ investigationId, version }: { investigationId: number; version: number }) {
  const [data, setData] = useState<Overview | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getOverview(investigationId)
      .then((o) => {
        if (!cancelled) {
          setData(o);
          setError(null);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(toCommandError(e).message);
      });
    return () => {
      cancelled = true;
    };
  }, [investigationId, version]);

  if (error) {
    return (
      <Panel title="Vue d'ensemble">
        <p className="text-muted">Statistiques indisponibles : {error}</p>
      </Panel>
    );
  }
  if (!data) {
    return (
      <Panel title="Vue d'ensemble">
        <p className="page-inline">
          <Spinner size="sm" /> Calcul des statistiques…
        </p>
      </Panel>
    );
  }
  if (data.events === 0) {
    return (
      <Panel title="Vue d'ensemble">
        <EmptyState icon={BarChart3} title="Aucun événement">
          {data.evidenceTotal === 0
            ? "Ajoutez des preuves puis analysez-les : leurs statistiques apparaîtront ici."
            : "Analysez les preuves de l'enquête : leurs statistiques apparaîtront ici."}
        </EmptyState>
      </Panel>
    );
  }

  const analyzed = data.sources.length;
  const first = data.firstAt;
  const last = data.lastAt;
  const span = first !== null && last !== null ? last - first + 1 : 0;
  const bucketStart = (i: number) => (first ?? 0) + Math.floor((span * i) / Math.max(data.histogram.length, 1));
  const untimed = data.events - data.timed;
  const kpis = [
    { label: "Événements", value: data.events, icon: Activity, tone: "blue", spark: data.histogram },
    { label: `Preuves analysées sur ${number.format(data.evidenceTotal)}`, value: analyzed, icon: FileStack, tone: "violet" },
    { label: "Adresses IP distinctes", value: data.ips, icon: Globe, tone: "cyan" },
    { label: "Utilisateurs distincts", value: data.users, icon: Users, tone: "green" },
  ] as const;

  return (
    <Panel title="Vue d'ensemble">
      <div className="kpi-grid overview-kpis">
        {kpis.map((k) => (
          <div key={k.label} className={`kpi kpi--${k.tone}`}>
            <span className="kpi__icon" aria-hidden="true">
              <k.icon size={20} />
            </span>
            <span className="kpi__text">
              <span className="kpi__value mono">{number.format(k.value)}</span>
              <span className="kpi__label">{k.label}</span>
            </span>
            {"spark" in k && k.spark.length > 1 && <Sparkline values={k.spark} />}
          </div>
        ))}
      </div>

      <div className="overview-grid">
        <section className="overview-block" aria-labelledby="overview-activity">
          <h3 id="overview-activity" className="overview-block__title">
            Activité dans le temps
          </h3>
          {first === null || last === null ? (
            <p className="text-muted">Aucun événement daté.</p>
          ) : (
            <AreaChart values={data.histogram} label={(i) => formatDateTime(bucketStart(i))} first={first} last={last} />
          )}
          <p className="overview-note">
            {first !== null && last !== null && `Du ${formatDateTime(first)} au ${formatDateTime(last)} (heure locale)`}
            {untimed > 0 && ` · ${plural(untimed, "événement")} sans heure`}
            {data.hosts > 0 && ` · ${plural(data.hosts, "hôte")}`}
          </p>
        </section>

        <section className="overview-block" aria-labelledby="overview-kinds">
          <h3 id="overview-kinds" className="overview-block__title">
            Catégories
          </h3>
          <Donut
            total={data.events}
            items={data.byKind.map((k) => ({ key: k.kind, label: KIND_LABELS[k.kind] ?? k.kind, value: k.events }))}
          />

          <h3 className="overview-block__title overview-block__title--spaced">Principales sources</h3>
          <Bars
            items={data.sources
              .slice(0, 6)
              .map((s) => ({ key: String(s.evidenceId), label: s.fileName, value: s.events }))}
          />
        </section>
      </div>
    </Panel>
  );
}

/** Mini-courbe (décorative, les valeurs sont dans le graphique d'activité). */
function Sparkline({ values }: { values: number[] }) {
  const max = Math.max(...values, 1);
  const pts = values.map((v, i) => `${(i / (values.length - 1)) * 100},${30 - (v / max) * 28}`).join(" ");
  return (
    <svg className="kpi__spark" viewBox="0 0 100 30" preserveAspectRatio="none" aria-hidden="true">
      <polyline points={`0,30 ${pts} 100,30`} className="kpi__spark-fill" />
      <polyline points={pts} className="kpi__spark-line" />
    </svg>
  );
}

const CHART_W = 600;
const CHART_H = 180;

/** Courbe d'activité en aire, avec repère au survol et équivalent textuel. */
function AreaChart({
  values,
  label,
  first,
  last,
}: {
  values: number[];
  label: (i: number) => string;
  first: number;
  last: number;
}) {
  const [hover, setHover] = useState<number | null>(null);
  const max = Math.max(...values, 1);
  const n = values.length;
  const x = (i: number) => (n === 1 ? CHART_W / 2 : (i / (n - 1)) * CHART_W);
  const y = (v: number) => CHART_H - (v / max) * (CHART_H - 12);
  const line = values.map((v, i) => `${x(i)},${y(v)}`).join(" ");
  return (
    <div className="overview-chart" onMouseLeave={() => setHover(null)}>
      <span className="overview-chart__tooltip mono" aria-hidden="true">
        {hover === null
          ? `Pic : ${plural(max, "événement")} par intervalle`
          : `${label(hover)} · ${plural(values[hover] ?? 0, "événement")}`}
      </span>
      <svg
        className="overview-chart__svg"
        viewBox={`0 0 ${CHART_W} ${CHART_H}`}
        preserveAspectRatio="none"
        aria-hidden="true"
        onMouseMove={(e) => {
          const r = e.currentTarget.getBoundingClientRect();
          const i = Math.round(((e.clientX - r.left) / r.width) * (n - 1));
          setHover(Math.min(n - 1, Math.max(0, i)));
        }}
      >
        <defs>
          <linearGradient id="overview-area" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0" stopColor="var(--chart-1)" stopOpacity="0.45" />
            <stop offset="1" stopColor="var(--chart-2)" stopOpacity="0.02" />
          </linearGradient>
          <linearGradient id="overview-line" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0" stopColor="var(--chart-1)" />
            <stop offset="1" stopColor="var(--chart-2)" />
          </linearGradient>
        </defs>
        {[0.25, 0.5, 0.75].map((f) => (
          <line key={f} x1="0" x2={CHART_W} y1={CHART_H * f} y2={CHART_H * f} className="overview-chart__grid" />
        ))}
        <polygon points={`0,${CHART_H} ${line} ${CHART_W},${CHART_H}`} fill="url(#overview-area)" />
        <polyline points={line} fill="none" stroke="url(#overview-line)" strokeWidth="2" vectorEffect="non-scaling-stroke" />
        {hover !== null && (
          <line x1={x(hover)} x2={x(hover)} y1="0" y2={CHART_H} className="overview-chart__cursor" />
        )}
      </svg>
      <div className="overview-chart__axis mono" aria-hidden="true">
        <span>{formatDateTime(first)}</span>
        <span>{formatDateTime(last)}</span>
      </div>
      <table className="sr-only">
        <caption>Événements datés par intervalle</caption>
        <tbody>
          {values.map((count, i) => (
            <tr key={i}>
              <th scope="row">À partir du {label(i)}</th>
              <td>{count}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Anneau de répartition, avec légende et pourcentages. */
function Donut({ total, items }: { total: number; items: Array<{ key: string; label: string; value: number }> }) {
  const R = 52;
  const C = 2 * Math.PI * R;
  let offset = 0;
  return (
    <div className="overview-donut">
      <svg viewBox="0 0 140 140" className="overview-donut__svg" aria-hidden="true">
        <circle cx="70" cy="70" r={R} className="overview-donut__track" />
        {items.map((it, i) => {
          const len = (it.value / Math.max(total, 1)) * C;
          const seg = (
            <circle
              key={it.key}
              cx="70"
              cy="70"
              r={R}
              fill="none"
              stroke={`var(--chart-${(i % 7) + 1})`}
              strokeWidth="14"
              strokeDasharray={`${Math.max(0, len - 1.5)} ${C}`}
              strokeDashoffset={-offset}
              transform="rotate(-90 70 70)"
            />
          );
          offset += len;
          return seg;
        })}
        <text x="70" y="68" textAnchor="middle" className="overview-donut__value">
          {number.format(total)}
        </text>
        <text x="70" y="86" textAnchor="middle" className="overview-donut__label">
          événements
        </text>
      </svg>
      <ul className="overview-donut__legend">
        {items.map((it, i) => (
          <li key={it.key}>
            <span className="overview-donut__swatch" style={{ background: `var(--chart-${(i % 7) + 1})` }} aria-hidden="true" />
            <span className="overview-donut__name">{it.label}</span>
            <span className="overview-donut__pct mono">
              {(Math.round((it.value / Math.max(total, 1)) * 1000) / 10).toLocaleString("fr-FR")} %
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}
