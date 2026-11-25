import { useEffect, useRef, useState } from "react";
import { Activity, ArrowLeft } from "lucide-react";
import {
  eventsWindow,
  getOverview,
  listEvidence,
  toCommandError,
  type Overview,
  type TimelineEvent,
} from "../api";
import { Button, EmptyState, Spinner } from "../components/ui";
import { formatDateTime } from "../format";
import { useRouter } from "../shell/router";
import { EventDrawer } from "./EventDrawer";
import { endpoint, eventClock, eventDate, kindLabel } from "./eventFormat";
import "./timeline.css";

/** Hauteur d'une ligne (px) : la liste est virtualisée sur cette base. */
const ROW = 34;
/** Événements chargés par requête. */
const PAGE = 200;
/** Pages gardées en mémoire au plus (les plus anciennes sont oubliées). */
const MAX_PAGES = 30;
/** Hauteur défilable maximale : au-delà, le défilement est mis à l'échelle (limite des navigateurs). */
const MAX_PX = 8_000_000;

const number = new Intl.NumberFormat("fr-FR");

/**
 * Chronologie d'une enquête : tous ses événements, du plus ancien au plus
 * récent (sans heure à la fin). Seules les lignes visibles sont affichées et
 * chargées, quelle que soit la taille de l'enquête.
 */
export function TimelinePage({ id }: { id: number }) {
  const { navigate } = useRouter();
  const [overview, setOverview] = useState<Overview | null>(null);
  const [names, setNames] = useState<Map<number, string>>(new Map());
  const [error, setError] = useState<string | null>(null);
  const [pages, setPages] = useState<Map<number, TimelineEvent[]>>(new Map());
  const loading = useRef(new Set<number>());
  const scroller = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewport, setViewport] = useState(600);
  const [selected, setSelected] = useState<TimelineEvent | null>(null);

  useEffect(() => {
    Promise.all([getOverview(id), listEvidence(id)])
      .then(([o, evidence]) => {
        setOverview(o);
        setNames(new Map(evidence.map((e) => [e.id, e.fileName])));
      })
      .catch((e: unknown) => setError(toCommandError(e).message));
  }, [id]);

  useEffect(() => {
    const el = scroller.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setViewport(el.clientHeight));
    observer.observe(el);
    return () => observer.disconnect();
  }, [overview]);

  const total = overview?.events ?? 0;
  const height = Math.min(total * ROW, MAX_PX);
  const visible = Math.ceil(viewport / ROW) + 1;
  // Position du haut de l'écran dans la liste, mise à l'échelle si la liste dépasse MAX_PX.
  const maxScroll = Math.max(height - viewport, 1);
  const firstFloat = (Math.min(scrollTop, maxScroll) / maxScroll) * Math.max(total - viewport / ROW, 0);
  const first = Math.floor(firstFloat);
  const shift = (firstFloat - first) * ROW;

  const firstPage = Math.floor(first / PAGE);
  const lastPage = Math.floor(Math.min(first + visible, Math.max(total - 1, 0)) / PAGE);

  useEffect(() => {
    if (total === 0) return;
    for (let p = firstPage; p <= lastPage; p++) {
      if (pages.has(p) || loading.current.has(p)) continue;
      loading.current.add(p);
      eventsWindow(id, p * PAGE, PAGE)
        .then((events) =>
          setPages((cur) => {
            const next = new Map(cur);
            next.set(p, events);
            // shortcut: oubli des pages les plus anciennement chargées, pas les plus éloignées ; suffisant tant que la mémoire reste bornée.
            while (next.size > MAX_PAGES) next.delete(next.keys().next().value!);
            return next;
          }),
        )
        .catch((e: unknown) => setError(toCommandError(e).message))
        .finally(() => loading.current.delete(p));
    }
  }, [id, total, firstPage, lastPage, pages]);

  if (error) {
    return (
      <div className="page">
        <p className="page-error">Chronologie indisponible : {error}</p>
      </div>
    );
  }
  if (!overview) {
    return (
      <div className="page">
        <p className="page-inline">
          <Spinner size="sm" /> Chargement…
        </p>
      </div>
    );
  }

  const back = (
    <Button icon={ArrowLeft} onClick={() => navigate({ page: "investigation", id })}>
      Retour à l'enquête
    </Button>
  );

  if (total === 0) {
    return (
      <div className="page">
        <EmptyState icon={Activity} title="Aucun événement" action={back}>
          Analysez les preuves de l'enquête : leurs événements apparaîtront ici, dans l'ordre chronologique.
        </EmptyState>
      </div>
    );
  }

  const rows: Array<{ index: number; event: TimelineEvent | undefined }> = [];
  for (let i = first; i < Math.min(first + visible, total); i++) {
    rows.push({ index: i, event: pages.get(Math.floor(i / PAGE))?.[i % PAGE] });
  }
  const untimed = overview.events - overview.timed;

  return (
    <div className="timeline-page">
      <div className="timeline-page__bar">
        {back}
        <p className="timeline-page__summary">
          <strong className="mono">{number.format(total)}</strong> événement{total > 1 ? "s" : ""}
          {overview.firstAt !== null &&
            overview.lastAt !== null &&
            ` · du ${formatDateTime(overview.firstAt)} au ${formatDateTime(overview.lastAt)} (heure locale)`}
          {untimed > 0 && ` · ${number.format(untimed)} sans heure, à la fin`}
        </p>
      </div>

      <div className="timeline-table" role="table" aria-label="Chronologie" aria-rowcount={total}>
        <div className="timeline-row timeline-row--head" role="row">
          <span role="columnheader">Date</span>
          <span role="columnheader">Heure</span>
          <span role="columnheader">Catégorie</span>
          <span role="columnheader">Événement</span>
          <span role="columnheader">Source → destination</span>
          <span role="columnheader">Utilisateur</span>
          <span role="columnheader">Preuve</span>
        </div>
        <div
          ref={scroller}
          className="timeline-table__scroller"
          onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
        >
          <div style={{ height }} />
          <div className="timeline-table__window" style={{ top: scrollTop, transform: `translateY(${-shift}px)` }}>
            {rows.map(({ index, event: e }) =>
              e ? (
                <button
                  key={index}
                  type="button"
                  role="row"
                  aria-rowindex={index + 1}
                  className={`timeline-row${selected?.id === e.id ? " timeline-row--selected" : ""}${e.outcome === "failure" ? " timeline-row--failure" : ""}`}
                  onClick={() => setSelected(e)}
                >
                  <span role="cell" className="mono">{eventDate(e)}</span>
                  <span role="cell" className="mono">{eventClock(e)}</span>
                  <span role="cell">
                    <span className={`kind-tag kind-tag--${e.kind}`}>{kindLabel(e.kind)}</span>
                  </span>
                  <span role="cell" className="timeline-row__message" title={e.message}>
                    {e.message}
                  </span>
                  <span role="cell" className="mono">
                    {[endpoint(e.sourceIp, e.sourcePort), endpoint(e.destinationIp, e.destinationPort)]
                      .filter(Boolean)
                      .join(" → ")}
                  </span>
                  <span role="cell">{e.user}</span>
                  <span role="cell" className="timeline-row__muted">
                    {names.get(e.evidenceId) ?? `n° ${e.evidenceId}`}
                  </span>
                </button>
              ) : (
                <div key={index} role="row" aria-rowindex={index + 1} className="timeline-row timeline-row--loading">
                  <span role="cell">…</span>
                </div>
              ),
            )}
          </div>
        </div>
      </div>

      {selected && (
        <EventDrawer
          event={selected}
          evidenceName={names.get(selected.evidenceId) ?? `preuve n° ${selected.evidenceId}`}
          onClose={() => setSelected(null)}
        />
      )}
    </div>
  );
}
