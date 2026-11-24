import { useEffect, useState } from "react";
import { BarChart3 } from "lucide-react";
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
  const [hover, setHover] = useState<number | null>(null);

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
  const max = Math.max(...data.histogram, 1);
  const first = data.firstAt;
  const last = data.lastAt;
  const span = first !== null && last !== null ? last - first + 1 : 0;
  const bucketStart = (i: number) => (first ?? 0) + Math.floor((span * i) / Math.max(data.histogram.length, 1));
  const untimed = data.events - data.timed;

  return (
    <Panel title="Vue d'ensemble">
      <div className="overview-kpis">
        {[
          ["Événements", data.events],
          [`Preuves analysées sur ${number.format(data.evidenceTotal)}`, analyzed],
          ["Adresses IP", data.ips],
          ["Utilisateurs", data.users],
        ].map(([label, value]) => (
          <div key={label} className="overview-kpi">
            <span className="overview-kpi__value mono">{number.format(value as number)}</span>
            <span className="overview-kpi__label">{label}</span>
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
            <div className="overview-chart" onMouseLeave={() => setHover(null)}>
              <span className="overview-chart__tooltip mono" aria-hidden="true">
                {hover === null
                  ? `Pic : ${plural(max, "événement")} par intervalle`
                  : `${formatDateTime(bucketStart(hover))} · ${plural(data.histogram[hover] ?? 0, "événement")}`}
              </span>
              <div className="overview-chart__bars" aria-hidden="true">
                {data.histogram.map((count, i) => (
                  <span key={i} className="overview-chart__slot" onMouseEnter={() => setHover(i)}>
                    <span
                      className={`overview-chart__bar${hover === i ? " overview-chart__bar--hover" : ""}`}
                      style={{ height: count === 0 ? 0 : `${Math.max(2, (count / max) * 100)}%` }}
                    />
                  </span>
                ))}
              </div>
              <div className="overview-chart__axis mono" aria-hidden="true">
                <span>{formatDateTime(first)}</span>
                <span>{formatDateTime(last)}</span>
              </div>
              <table className="sr-only">
                <caption>Événements datés par intervalle</caption>
                <tbody>
                  {data.histogram.map((count, i) => (
                    <tr key={i}>
                      <th scope="row">À partir du {formatDateTime(bucketStart(i))}</th>
                      <td>{count}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <p className="overview-note">
            {first !== null && last !== null && `Du ${formatDateTime(first)} au ${formatDateTime(last)} (heure locale)`}
            {untimed > 0 && ` · ${plural(untimed, "événement")} sans heure`}
            {data.hosts > 0 && ` · ${plural(data.hosts, "hôte")}`}
          </p>
        </section>

        <section className="overview-block" aria-labelledby="overview-sources">
          <h3 id="overview-sources" className="overview-block__title">
            Principales sources
          </h3>
          <Bars
            items={data.sources
              .slice(0, 6)
              .map((s) => ({ key: String(s.evidenceId), label: s.fileName, value: s.events }))}
          />

          <h3 className="overview-block__title overview-block__title--spaced">Catégories</h3>
          <Bars items={data.byKind.map((k) => ({ key: k.kind, label: KIND_LABELS[k.kind] ?? k.kind, value: k.events }))} />
        </section>
      </div>
    </Panel>
  );
}
