import { useEffect, useRef, useState } from "react";
import { Activity, ArrowLeft, RotateCcw } from "lucide-react";
import {
  eventsCount,
  eventsWindow,
  getOverview,
  listEvidence,
  toCommandError,
  type EventFilter,
  type Overview,
  type TimelineEvent,
} from "../api";
import { Button, EmptyState, SearchField, SegmentedControl, Select, Spinner } from "../components/ui";
import { useSettings } from "../settings/SettingsProvider";
import { useRouter } from "../shell/router";
import { EventDrawer } from "./EventDrawer";
import { EntityGraph } from "./EntityGraph";
import { GroupsView, type Pivot } from "./GroupsView";
import { BASIS_LABELS, endpoint, eventClock, eventDate, formatMs, isUncertain, kindLabel, zoneLabel } from "./eventFormat";
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

/** Saisie des filtres (texte des champs ; vide : pas de filtre). */
const EMPTY = { text: "", kind: "", evidence: "", outcome: "", time: "", ip: "", user: "", host: "", from: "", to: "" };
type Draft = typeof EMPTY;

/** « AAAA-MM-JJTHH:MM » d'un champ datetime-local, lu dans le fuseau d'affichage → µs UTC (début ou fin de la minute). */
function fieldToUs(value: string, end: boolean, utc: boolean): number | undefined {
  const ms = new Date(utc ? `${value}Z` : value).getTime();
  if (!value || Number.isNaN(ms)) return undefined;
  return ms * 1000 + (end ? 59_999_999 : 0);
}

function toFilter(d: Draft, utc: boolean, flow: EventFilter["flow"] | null): EventFilter {
  const f: EventFilter = {};
  if (flow) f.flow = flow;
  const from = fieldToUs(d.from, false, utc);
  const to = fieldToUs(d.to, true, utc);
  if (from !== undefined) f.fromUs = from;
  if (to !== undefined) f.toUs = to;
  if (d.kind) f.kind = d.kind;
  if (d.evidence) f.evidenceId = Number(d.evidence);
  if (d.outcome === "success" || d.outcome === "failure") f.outcome = d.outcome;
  if (d.time === "uncertain" || d.time === "missing") f.time = d.time;
  for (const k of ["ip", "user", "host", "text"] as const) if (d[k].trim()) f[k] = d[k].trim();
  return f;
}

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
  const { settings, update } = useSettings();
  const utc = settings.displayUtc;
  const [draft, setDraft] = useState<Draft>(EMPTY);
  /** Flux réseau choisi par pivot (pas de champ de saisie). */
  const [flow, setFlow] = useState<EventFilter["flow"] | null>(null);
  const [view, setView] = useState<"events" | "groups" | "graph">("events");

  /** L'entité choisie remplace les autres critères d'entité ; période et autres filtres restent. */
  const pivot = (p: Pivot) => {
    setDraft((d) => ({ ...d, ip: "", user: "", host: "", ...(p.kind === "flow" ? {} : { [p.kind]: p.value }) }));
    setFlow(p.kind === "flow" ? { sourceIp: p.sourceIp, destinationIp: p.destinationIp, destinationPort: p.destinationPort } : null);
    setSelected(null);
    setView("events");
  };
  const [filter, setFilter] = useState<EventFilter>({});
  const [total, setTotal] = useState<number | null>(null);
  /** Génération du filtre : les réponses d'un filtre précédent sont ignorées. */
  const generation = useRef(0);
  const set = (k: keyof Draft) => (v: string) => setDraft((d) => ({ ...d, [k]: v }));

  // Saisie appliquée après une courte pause (pas une requête par touche).
  useEffect(() => {
    const t = setTimeout(() => {
      const next = toFilter(draft, utc, flow);
      // Même filtre : rien à recharger.
      setFilter((cur) => (JSON.stringify(cur) === JSON.stringify(next) ? cur : next));
    }, 300);
    return () => clearTimeout(t);
  }, [draft, utc, flow]);

  useEffect(() => {
    const gen = ++generation.current;
    setPages(new Map());
    loading.current.clear();
    setTotal(null);
    setSelected(null);
    if (scroller.current) scroller.current.scrollTop = 0;
    setScrollTop(0);
    eventsCount(id, filter)
      .then((n) => gen === generation.current && setTotal(n))
      .catch((e: unknown) => setError(toCommandError(e).message));
  }, [id, filter]);

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
  }, [overview, view]);

  const count = total ?? 0;
  const height = Math.min(count * ROW, MAX_PX);
  const visible = Math.ceil(viewport / ROW) + 1;
  // Position du haut de l'écran dans la liste, mise à l'échelle si la liste dépasse MAX_PX.
  const maxScroll = Math.max(height - viewport, 1);
  const firstFloat = (Math.min(scrollTop, maxScroll) / maxScroll) * Math.max(count - viewport / ROW, 0);
  const first = Math.floor(firstFloat);
  const shift = (firstFloat - first) * ROW;

  const firstPage = Math.floor(first / PAGE);
  const lastPage = Math.floor(Math.min(first + visible, Math.max(count - 1, 0)) / PAGE);

  useEffect(() => {
    if (count === 0) return;
    const gen = generation.current;
    for (let p = firstPage; p <= lastPage; p++) {
      if (pages.has(p) || loading.current.has(p)) continue;
      loading.current.add(p);
      eventsWindow(id, filter, p * PAGE, PAGE)
        .then((events) =>
          gen === generation.current &&
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
  }, [id, filter, count, firstPage, lastPage, pages]);

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

  if (overview.events === 0) {
    return (
      <div className="page">
        <EmptyState icon={Activity} title="Aucun événement" action={back}>
          Analysez les preuves de l'enquête : leurs événements apparaîtront ici, dans l'ordre chronologique.
        </EmptyState>
      </div>
    );
  }

  const rows: Array<{ index: number; event: TimelineEvent | undefined }> = [];
  for (let i = first; i < Math.min(first + visible, count); i++) {
    rows.push({ index: i, event: pages.get(Math.floor(i / PAGE))?.[i % PAGE] });
  }
  const untimed = overview.events - overview.timed;

  return (
    <div className="timeline-page">
      <div className="timeline-page__bar">
        {back}
        <p className="timeline-page__summary">
          {total !== null && total !== overview.events && (
            <>
              <strong className="mono">{number.format(total)}</strong> retenu{total > 1 ? "s" : ""} sur{" "}
            </>
          )}
          <strong className="mono">{number.format(overview.events)}</strong> événement{overview.events > 1 ? "s" : ""}
          {overview.firstAt !== null &&
            overview.lastAt !== null &&
            ` · du ${formatMs(overview.firstAt, utc)} au ${formatMs(overview.lastAt, utc)} (${zoneLabel(utc)})`}
          {untimed > 0 && ` · ${number.format(untimed)} sans heure, à la fin`}
        </p>
        <span className="timeline-page__zone">
          <SegmentedControl
            label="Affichage"
            value={view}
            onChange={setView}
            options={[
              { value: "events", label: "Événements" },
              { value: "groups", label: "Regroupements" },
              { value: "graph", label: "Graphe" },
            ]}
          />
          <SegmentedControl
            label="Fuseau d'affichage"
            value={utc ? "utc" : "local"}
            onChange={(v) => void update({ displayUtc: v === "utc" }).catch((e: unknown) => setError(toCommandError(e).message))}
            options={[
              { value: "local", label: "Heure locale" },
              { value: "utc", label: "UTC" },
            ]}
          />
        </span>
      </div>

      <div className="timeline-filters" role="search" aria-label="Filtrer la chronologie">
        <SearchField
          label="Rechercher dans les messages"
          placeholder="Rechercher dans les messages…"
          value={draft.text}
          onChange={set("text")}
        />
        <Select
          label="Catégorie"
          value={draft.kind}
          onChange={set("kind")}
          options={[{ value: "", label: "Toutes" }, ...overview.byKind.map((k) => ({ value: k.kind, label: kindLabel(k.kind) }))]}
        />
        <Select
          label="Preuve"
          value={draft.evidence}
          onChange={set("evidence")}
          options={[
            { value: "", label: "Toutes" },
            ...overview.sources.map((s) => ({ value: String(s.evidenceId), label: s.fileName })),
          ]}
        />
        <Select
          label="Issue"
          value={draft.outcome}
          onChange={set("outcome")}
          options={[
            { value: "", label: "Toutes" },
            { value: "success", label: "Réussite" },
            { value: "failure", label: "Échec" },
          ]}
        />
        <Select
          label="Heure"
          value={draft.time}
          onChange={set("time")}
          options={[
            { value: "", label: "Toutes" },
            { value: "uncertain", label: "Incertaine" },
            { value: "missing", label: "Absente" },
          ]}
        />
        {(
          [
            ["ip", "Adresse IP (début)", "203.0.113."],
            ["user", "Utilisateur", "root"],
            ["host", "Hôte", "srv-web"],
          ] as const
        ).map(([k, label, example]) => (
          <input
            key={k}
            className="ui-input timeline-filters__text mono"
            aria-label={label}
            placeholder={`${label} : ${example}`}
            value={draft[k]}
            spellCheck={false}
            onChange={(e) => set(k)(e.target.value)}
          />
        ))}
        <label className="timeline-filters__period" title={`Période en ${zoneLabel(utc)}`}>
          Du
          <input type="datetime-local" className="ui-input" value={draft.from} onChange={(e) => set("from")(e.target.value)} />
        </label>
        <label className="timeline-filters__period">
          au
          <input type="datetime-local" className="ui-input" value={draft.to} onChange={(e) => set("to")(e.target.value)} />
        </label>
        {flow && (
          <span className="timeline-filters__chip mono">
            Flux {flow.sourceIp} → {endpoint(flow.destinationIp, flow.destinationPort)}
            <button type="button" aria-label="Retirer le filtre de flux" onClick={() => setFlow(null)}>
              ×
            </button>
          </span>
        )}
        {(flow || JSON.stringify(draft) !== JSON.stringify(EMPTY)) && (
          <Button
            size="sm"
            variant="ghost"
            icon={RotateCcw}
            onClick={() => {
              setDraft(EMPTY);
              setFlow(null);
            }}
          >
            Réinitialiser
          </Button>
        )}
      </div>

      {view === "groups" ? (
        <GroupsView investigationId={id} filter={filter} utc={utc} onPivot={pivot} />
      ) : view === "graph" ? (
        <div className="timeline-graph">
          <EntityGraph investigationId={id} filter={filter} onPivot={pivot} />
        </div>
      ) : (
      <div className="timeline-table" role="table" aria-label="Chronologie" aria-rowcount={count}>
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
          {total === 0 && <p className="timeline-table__empty">Aucun événement ne correspond aux filtres.</p>}
          {total === null && (
            <p className="timeline-table__empty page-inline">
              <Spinner size="sm" /> Recherche…
            </p>
          )}
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
                  <span role="cell" className="mono">{eventDate(e, utc)}</span>
                  <span role="cell" className="mono">
                    {eventClock(e, utc)}
                    {isUncertain(e) && (
                      <abbr className="timeline-row__uncertain" title={`Heure incertaine : ${e.timeBasis ? BASIS_LABELS[e.timeBasis] : ""}`}>
                        {" "}≈
                      </abbr>
                    )}
                  </span>
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
      )}

      {selected && (
        <EventDrawer
          event={selected}
          evidenceName={names.get(selected.evidenceId) ?? `preuve n° ${selected.evidenceId}`}
          utc={utc}
          onPivot={pivot}
          onClose={() => setSelected(null)}
        />
      )}
    </div>
  );
}
