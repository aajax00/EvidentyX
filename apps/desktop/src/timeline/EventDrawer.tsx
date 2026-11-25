import { useEffect, useState } from "react";
import { CircleAlert, CircleCheck, Copy, Filter, Info, Waypoints, X } from "lucide-react";
import type { TimelineEvent } from "../api";
import type { Pivot } from "./GroupsView";
import { IconButton, useToast } from "../components/ui";
import { BASIS_LABELS, PRECISION_LABELS, endpoint, eventTimeFull, isUncertain, kindLabel } from "./eventFormat";

type Tab = "info" | "attributes" | "source";

/**
 * Panneau latéral : détail d'un événement tel qu'extrait de la preuve.
 * Échap ou ✕ le ferme. Rien n'est déduit : seuls les champs présents dans la
 * source sont affichés.
 */
export function EventDrawer({
  event: e,
  evidenceName,
  utc,
  onPivot,
  onClose,
}: {
  event: TimelineEvent;
  evidenceName: string;
  /** Heures affichées en UTC (sinon : heure locale). */
  utc: boolean;
  /** Affiche la chronologie de l'entité choisie. */
  onPivot: (p: Pivot) => void;
  onClose: () => void;
}) {
  const { notify } = useToast();
  const [tab, setTab] = useState<Tab>("info");
  const attributes = Object.entries(e.attributes);

  useEffect(() => {
    const onKey = (k: KeyboardEvent) => {
      if (k.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  useEffect(() => setTab("info"), [e.id]);

  const tone = e.outcome === "failure" ? "danger" : e.outcome === "success" ? "success" : "accent";
  const Icon = e.outcome === "failure" ? CircleAlert : e.outcome === "success" ? CircleCheck : Info;
  const pivotOn = (kind: "ip" | "user" | "host", value: string | null): Pivot | undefined =>
    value ? { kind, value } : undefined;
  const rows: Array<[string, string | null, (Pivot | undefined)?]> = [
    ["Catégorie", kindLabel(e.kind)],
    ["Action", e.action],
    ["Issue", e.outcome === "failure" ? "Échec" : e.outcome === "success" ? "Réussite" : null],
    ["Source", endpoint(e.sourceIp, e.sourcePort), pivotOn("ip", e.sourceIp)],
    ["Destination", endpoint(e.destinationIp, e.destinationPort), pivotOn("ip", e.destinationIp)],
    ["Utilisateur", e.user, pivotOn("user", e.user)],
    ["Hôte", e.host, pivotOn("host", e.host)],
    ["Précision", e.timePrecision ? (PRECISION_LABELS[e.timePrecision] ?? e.timePrecision) : null],
    ["Origine de l'heure", e.timeBasis ? (BASIS_LABELS[e.timeBasis] ?? e.timeBasis) : null],
  ];

  async function copyRaw() {
    try {
      await navigator.clipboard.writeText(e.raw);
      notify("Texte brut copié.");
    } catch {
      notify("Copie impossible : sélectionnez le texte à la main.", "danger");
    }
  }

  return (
    <aside className="event-drawer" aria-label="Détails de l'événement">
      <header className="event-drawer__header">
        <h2 className="event-drawer__title">Détails de l'événement</h2>
        <IconButton autoFocus icon={X} label="Fermer le détail" onClick={onClose} />
      </header>

      <div className={`event-drawer__summary event-drawer__summary--${tone}`}>
        <Icon size={20} aria-hidden="true" />
        <div>
          <p className="event-drawer__message">{e.message}</p>
          <p className="event-drawer__time mono">{eventTimeFull(e, utc)}</p>
          {isUncertain(e) && (
            <p className="event-drawer__uncertain">
              Heure incertaine : {e.timeBasis ? BASIS_LABELS[e.timeBasis] : ""}. À recouper avant de conclure.
            </p>
          )}
        </div>
      </div>

      <div className="event-drawer__tabs" role="tablist" aria-label="Sections du détail">
        {(
          [
            ["info", "Informations"],
            ["attributes", `Attributs (${attributes.length})`],
            ["source", "Source"],
          ] as const
        ).map(([id, label]) => (
          <button
            key={id}
            type="button"
            role="tab"
            aria-selected={tab === id}
            className={`event-drawer__tab${tab === id ? " event-drawer__tab--active" : ""}`}
            onClick={() => setTab(id)}
          >
            {label}
          </button>
        ))}
      </div>

      <div className="event-drawer__body" role="tabpanel">
        {tab === "info" && (
          <dl className="event-drawer__facts">
            {rows
              .filter((r): r is [string, string, (Pivot | undefined)?] => r[1] !== null)
              .map(([k, v, p]) => (
                <div key={k}>
                  <dt>{k}</dt>
                  <dd className={k === "Source" || k === "Destination" ? "mono" : undefined}>
                    {p ? (
                      <button
                        type="button"
                        className="event-drawer__pivot"
                        title="Afficher tous les événements de cette entité"
                        onClick={() => onPivot(p)}
                      >
                        {v} <Filter size={12} aria-hidden="true" />
                      </button>
                    ) : (
                      v
                    )}
                  </dd>
                </div>
              ))}
          </dl>
        )}
        {tab === "info" && e.sourceIp && e.destinationIp && (
          <button
            type="button"
            className="event-drawer__copy event-drawer__flow"
            onClick={() =>
              onPivot({
                kind: "flow",
                sourceIp: e.sourceIp!,
                destinationIp: e.destinationIp!,
                destinationPort: e.destinationPort,
              })
            }
          >
            <Waypoints size={14} aria-hidden="true" /> Événements du même flux
          </button>
        )}

        {tab === "attributes" &&
          (attributes.length === 0 ? (
            <p className="text-muted">Aucun autre champ dans la source.</p>
          ) : (
            <dl className="event-drawer__facts event-drawer__facts--attributes">
              {attributes.map(([k, v]) => (
                <div key={k}>
                  <dt className="mono">{k}</dt>
                  <dd className="mono">{v}</dd>
                </div>
              ))}
            </dl>
          ))}

        {tab === "source" && (
          <div className="event-drawer__source">
            <p className="event-drawer__origin">
              Preuve <strong>« {evidenceName} »</strong>
              {e.line !== null && <> · ligne {e.line.toLocaleString("fr-FR")}</>}
              {e.record !== null && <> · enregistrement {e.record.toLocaleString("fr-FR")}</>}
              {` · octet ${e.byteOffset.toLocaleString("fr-FR")}`}
            </p>
            <pre className="event-drawer__raw mono">{e.raw}</pre>
            {e.rawTruncated && (
              <p className="text-muted">
                Extrait tronqué : l'enregistrement complet se lit dans la preuve à l'octet indiqué.
              </p>
            )}
            <button type="button" className="event-drawer__copy" onClick={() => void copyRaw()}>
              <Copy size={14} aria-hidden="true" /> Copier le texte brut
            </button>
          </div>
        )}
      </div>
    </aside>
  );
}
