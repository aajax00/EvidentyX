import { useCallback, useEffect, useState } from "react";
import { listEvents, toCommandError, type TimelineEvent } from "../api";
import { Button, Spinner } from "../components/ui";
import { eventClock, eventDate, eventSubline, kindLabel } from "./eventFormat";

const PAGE = 50;

/**
 * Chronologie des événements d'une enquête, par pages. Un clic ouvre le
 * détail de l'événement. `version` change quand les preuves changent.
 */
export function Timeline({
  investigationId,
  version,
  selectedId,
  onSelect,
}: {
  investigationId: number;
  version: number;
  selectedId: number | null;
  onSelect: (e: TimelineEvent) => void;
}) {
  const [events, setEvents] = useState<TimelineEvent[] | null>(null);
  const [next, setNext] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(
    async (after: number | null) => {
      setLoading(true);
      try {
        const page = await listEvents(investigationId, after, PAGE);
        setEvents((cur) => (after === null ? page.events : [...(cur ?? []), ...page.events]));
        setNext(page.next);
        setError(null);
      } catch (e) {
        setError(toCommandError(e).message);
      } finally {
        setLoading(false);
      }
    },
    [investigationId],
  );

  useEffect(() => {
    void load(null);
  }, [load, version]);

  if (error) return <p className="text-muted">Chronologie indisponible : {error}</p>;
  if (events === null)
    return (
      <p className="page-inline">
        <Spinner size="sm" /> Chargement…
      </p>
    );

  return (
    <div className="timeline">
      <ol className="timeline__list" aria-label="Événements, du plus ancien au plus récent">
        {events.map((e, i) => {
          const newDay = i === 0 || eventDate(events[i - 1]!) !== eventDate(e);
          return (
            <li key={e.id}>
              {newDay && <p className="timeline__day mono">{eventDate(e)}</p>}
              <button
                type="button"
                className={`timeline__item${selectedId === e.id ? " timeline__item--selected" : ""}${e.outcome === "failure" ? " timeline__item--failure" : ""}`}
                aria-current={selectedId === e.id ? "true" : undefined}
                onClick={() => onSelect(e)}
              >
                <span className="timeline__time mono">{eventClock(e)}</span>
                <span className="timeline__node" aria-hidden="true" />
                <span className="timeline__body">
                  <span className="timeline__message">{e.message}</span>
                  {eventSubline(e) && <span className="timeline__sub mono">{eventSubline(e)}</span>}
                </span>
                <span className={`kind-tag kind-tag--${e.kind}`}>{kindLabel(e.kind)}</span>
              </button>
            </li>
          );
        })}
      </ol>
      {next !== null && (
        <Button size="sm" variant="ghost" loading={loading} onClick={() => void load(next)}>
          Afficher les {PAGE} suivants
        </Button>
      )}
    </div>
  );
}
