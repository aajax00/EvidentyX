import { useEffect, useState } from "react";
import { eventGroups, toCommandError, type EventFilter, type EventGroup, type GroupBy } from "../api";
import { SegmentedControl, Spinner } from "../components/ui";
import { endpoint, formatMs } from "./eventFormat";

/** Groupes affichés au plus (les plus fournis). */
const LIMIT = 200;
const number = new Intl.NumberFormat("fr-FR");

/** Pivot vers la chronologie : l'entité choisie devient le filtre. */
export type Pivot =
  | { kind: "ip" | "user" | "host"; value: string }
  | { kind: "flow"; sourceIp: string; destinationIp: string; destinationPort: number | null };

const BY_LABELS: Record<GroupBy, string> = {
  ip: "Adresse IP",
  user: "Utilisateur",
  host: "Hôte",
  flow: "Flux réseau (source → destination)",
};

/**
 * Événements retenus par les filtres, regroupés par adresse, utilisateur,
 * hôte ou flux réseau. Un clic ouvre la chronologie de ce groupe.
 */
export function GroupsView({
  investigationId,
  filter,
  utc,
  onPivot,
}: {
  investigationId: number;
  filter: EventFilter;
  utc: boolean;
  onPivot: (p: Pivot) => void;
}) {
  const [by, setBy] = useState<GroupBy>("ip");
  const [groups, setGroups] = useState<EventGroup[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setGroups(null);
    eventGroups(investigationId, filter, by, LIMIT)
      .then((g) => !cancelled && setGroups(g))
      .catch((e: unknown) => !cancelled && setError(toCommandError(e).message));
    return () => {
      cancelled = true;
    };
  }, [investigationId, filter, by]);

  const period = (us: number | null) => (us === null ? "—" : formatMs(Math.floor(us / 1000), utc));
  const pivotOf = (g: EventGroup): Pivot =>
    by === "flow"
      ? { kind: "flow", sourceIp: g.key, destinationIp: g.destinationIp ?? "", destinationPort: g.destinationPort }
      : { kind: by, value: g.key };

  return (
    <div className="groups">
      <div className="groups__bar">
        <SegmentedControl
          label="Regrouper par"
          value={by}
          onChange={setBy}
          options={[
            { value: "ip", label: "Adresses IP" },
            { value: "user", label: "Utilisateurs" },
            { value: "host", label: "Hôtes" },
            { value: "flow", label: "Flux réseau" },
          ]}
        />
        <span className="groups__hint">Cliquez sur un groupe pour afficher ses événements.</span>
      </div>
      {error ? (
        <p className="page-error">Regroupements indisponibles : {error}</p>
      ) : groups === null ? (
        <p className="page-inline">
          <Spinner size="sm" /> Calcul des regroupements…
        </p>
      ) : groups.length === 0 ? (
        <p className="text-muted">Aucun événement retenu ne cite d'entité de ce type.</p>
      ) : (
        <div className="timeline-table groups__table" role="table" aria-label={`Regroupement par ${BY_LABELS[by]}`}>
          <div className="groups__row timeline-row--head" role="row">
            <span role="columnheader">{BY_LABELS[by]}</span>
            <span role="columnheader">Événements</span>
            <span role="columnheader">Échecs</span>
            <span role="columnheader">Premier</span>
            <span role="columnheader">Dernier</span>
          </div>
          <div className="groups__scroller">
            {groups.map((g) => (
              <button
                key={`${g.key}|${g.destinationIp}|${g.destinationPort}`}
                type="button"
                role="row"
                className="groups__row"
                onClick={() => onPivot(pivotOf(g))}
              >
                <span role="cell" className="mono">
                  {by === "flow" ? `${g.key} → ${endpoint(g.destinationIp, g.destinationPort)}` : g.key}
                </span>
                <span role="cell" className="mono">{number.format(g.events)}</span>
                <span role="cell" className={`mono${g.failures > 0 ? " groups__failures" : ""}`}>
                  {number.format(g.failures)}
                </span>
                <span role="cell" className="mono">{period(g.firstUs)}</span>
                <span role="cell" className="mono">{period(g.lastUs)}</span>
              </button>
            ))}
          </div>
          {groups.length === LIMIT && (
            <p className="groups__more">Les {LIMIT} groupes les plus fournis sont affichés : affinez les filtres pour les autres.</p>
          )}
        </div>
      )}
    </div>
  );
}
