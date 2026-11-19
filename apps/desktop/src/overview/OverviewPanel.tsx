import { useMemo, useState } from "react";
import { FlaskConical } from "lucide-react";
import { Badge, Panel } from "../components/ui";
import { demoOverview } from "./demoOverview";
import "./overview.css";

const number = new Intl.NumberFormat("fr-FR");

const SEVERITIES = [
  { key: "critical", label: "Critique" },
  { key: "high", label: "Élevée" },
  { key: "medium", label: "Moyenne" },
  { key: "low", label: "Faible" },
] as const;

/**
 * Vue d'ensemble d'une enquête, illustrée par des données de démonstration
 * tant que l'import de preuves n'existe pas (phase 2). La mention est
 * affichée en titre et dans un bandeau : ces chiffres ne décrivent rien de réel.
 */
export function OverviewPanel({ investigationId }: { investigationId: number }) {
  const data = useMemo(() => demoOverview(investigationId), [investigationId]);
  const [hover, setHover] = useState<number | null>(null);
  const max = Math.max(...data.hourly, 1);
  const maxSource = data.topSources[0]?.events ?? 1;

  return (
    <Panel title="Vue d'ensemble" actions={<Badge tone="warning">Données de démonstration</Badge>}>
      <p className="overview-demo" role="note">
        <FlaskConical size={14} aria-hidden="true" />
        Aperçu illustratif : ces chiffres sont fictifs. Ils seront remplacés par l'analyse réelle des preuves importées
        (phase 2).
      </p>

      <div className="overview-kpis">
        {[
          ["Événements", data.events],
          ["Sources", data.sources],
          ["Entités", data.entities],
          ["Alertes", data.alerts],
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
            Activité sur 24 h
          </h3>
          <div className="overview-chart" onMouseLeave={() => setHover(null)}>
            <span className="overview-chart__tooltip mono" aria-hidden="true">
              {hover === null
                ? `Pic : ${number.format(max)} événements`
                : `${String(hover).padStart(2, "0")} h · ${number.format(data.hourly[hover] ?? 0)} événements`}
            </span>
            <div className="overview-chart__bars" aria-hidden="true">
              {data.hourly.map((count, hour) => (
                <span
                  key={hour}
                  className="overview-chart__slot"
                  onMouseEnter={() => setHover(hour)}
                >
                  <span
                    className={`overview-chart__bar${hover === hour ? " overview-chart__bar--hover" : ""}`}
                    style={{ height: `${Math.max(2, (count / max) * 100)}%` }}
                  />
                </span>
              ))}
            </div>
            <div className="overview-chart__axis mono" aria-hidden="true">
              <span>00 h</span>
              <span>06 h</span>
              <span>12 h</span>
              <span>18 h</span>
              <span>23 h</span>
            </div>
            <table className="sr-only">
              <caption>Événements par heure (données de démonstration)</caption>
              <tbody>
                {data.hourly.map((count, hour) => (
                  <tr key={hour}>
                    <th scope="row">{hour} h</th>
                    <td>{count}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <section className="overview-block" aria-labelledby="overview-sources">
          <h3 id="overview-sources" className="overview-block__title">
            Principales sources
          </h3>
          <ul className="overview-sources">
            {data.topSources.map((s) => (
              <li key={s.name}>
                <span className="overview-sources__name mono">{s.name}</span>
                <span className="overview-sources__track" aria-hidden="true">
                  <span className="overview-sources__fill" style={{ width: `${(s.events / maxSource) * 100}%` }} />
                </span>
                <span className="overview-sources__value mono">{number.format(s.events)}</span>
              </li>
            ))}
          </ul>

          <h3 className="overview-block__title overview-block__title--spaced">Alertes par gravité</h3>
          <ul className="overview-severity">
            {SEVERITIES.map(({ key, label }) => (
              <li key={key} className={`overview-severity__item overview-severity__item--${key}`}>
                <span className="overview-severity__swatch" aria-hidden="true" />
                <span>{label}</span>
                <span className="overview-severity__value mono">{data.severity[key]}</span>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </Panel>
  );
}
