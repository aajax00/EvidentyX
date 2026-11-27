import { useEffect, useMemo, useRef, useState } from "react";
import { getEntityLinks, toCommandError, type EntityKind, type EntityLink, type EventFilter } from "../api";
import type { Pivot } from "./GroupsView";
import { Button, Spinner, useToast } from "../components/ui";

/** Liens affichés au plus (les plus fréquents). */
const LIMIT = 60;
const W = 640;
const H = 340;

/** Couleurs et formes reprises de la maquette : réseau en losange, hôte et compte en carré. */
const KIND: Record<EntityKind, { label: string; plural: string; color: string; css: string }> = {
  ip: { label: "Adresse IP", plural: "Adresses IP", color: "#f0696a", css: "var(--color-danger)" },
  host: { label: "Hôte", plural: "Hôtes", color: "#3d8ff0", css: "var(--color-accent)" },
  user: { label: "Utilisateur", plural: "Utilisateurs", color: "#e0a548", css: "var(--color-warning)" },
};

/** Écart entre la disposition calculée et le monde du canevas (nœuds de 30 px). */
const SPREAD = 1.9;

interface Node {
  id: string;
  kind: EntityKind;
  name: string;
  weight: number;
  x: number;
  y: number;
}

/** Disposition par forces, déterministe (positions initiales en cercle). */
export function layout(links: EntityLink[]): { nodes: Node[]; edges: Array<{ a: Node; b: Node; events: number }> } {
  const byId = new Map<string, Node>();
  const node = (kind: EntityKind, name: string, events: number) => {
    const id = `${kind}:${name}`;
    const n = byId.get(id) ?? { id, kind, name, weight: 0, x: 0, y: 0 };
    n.weight += events;
    byId.set(id, n);
    return n;
  };
  const edges = links.map((l) => ({ a: node(l.fromKind, l.from, l.events), b: node(l.toKind, l.to, l.events), events: l.events }));
  const nodes = [...byId.values()];
  nodes.forEach((n, i) => {
    const angle = (i / nodes.length) * Math.PI * 2;
    n.x = W / 2 + Math.cos(angle) * W * 0.3;
    n.y = H / 2 + Math.sin(angle) * H * 0.3;
  });
  const k = Math.sqrt((W * H) / Math.max(nodes.length, 1)) * 0.75;
  for (let iter = 0, temp = W / 8; iter < 300; iter++, temp *= 0.985) {
    const dx = new Map<Node, number>(nodes.map((n) => [n, 0]));
    const dy = new Map<Node, number>(nodes.map((n) => [n, 0]));
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i]!;
        const b = nodes[j]!;
        const ex = a.x - b.x || 0.01;
        const ey = a.y - b.y || 0.01;
        const d2 = ex * ex + ey * ey;
        const f = (k * k) / d2;
        dx.set(a, dx.get(a)! + ex * f);
        dy.set(a, dy.get(a)! + ey * f);
        dx.set(b, dx.get(b)! - ex * f);
        dy.set(b, dy.get(b)! - ey * f);
      }
    }
    for (const { a, b } of edges) {
      const ex = a.x - b.x;
      const ey = a.y - b.y;
      const d = Math.sqrt(ex * ex + ey * ey) || 0.01;
      const f = d / k;
      dx.set(a, dx.get(a)! - ex * f);
      dy.set(a, dy.get(a)! - ey * f);
      dx.set(b, dx.get(b)! + ex * f);
      dy.set(b, dy.get(b)! + ey * f);
    }
    for (const n of nodes) {
      // Légère gravité vers le centre, puis déplacement borné par la « température ».
      const gx = dx.get(n)! + (W / 2 - n.x) * 0.05;
      const gy = dy.get(n)! + (H / 2 - n.y) * 0.05;
      const len = Math.sqrt(gx * gx + gy * gy) || 1;
      n.x += (gx / len) * Math.min(len, temp);
      n.y += (gy / len) * Math.min(len, temp);
    }
  }
  // Mise à l'échelle dans le cadre, avec marge.
  const xs = nodes.map((n) => n.x);
  const ys = nodes.map((n) => n.y);
  const [minX, maxX, minY, maxY] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  for (const n of nodes) {
    n.x = 40 + ((n.x - minX) / Math.max(maxX - minX, 1)) * (W - 80);
    n.y = 24 + ((n.y - minY) / Math.max(maxY - minY, 1)) * (H - 48);
  }
  return { nodes, edges };
}

/**
 * Graphe des échanges observés : adresses, utilisateurs et hôtes reliés par
 * les événements qui les citent ensemble. Un relevé, pas une corrélation.
 * Molette : zoom ; glisser le fond : déplacer ; glisser un nœud : réorganiser ;
 * clic : inspecter.
 */
export function EntityGraph({
  investigationId,
  filter,
  onPivot,
}: {
  investigationId: number;
  filter: EventFilter;
  onPivot: (p: Pivot) => void;
}) {
  const { notify } = useToast();
  const [links, setLinks] = useState<EntityLink[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [kinds, setKinds] = useState<Record<EntityKind, boolean>>({ ip: true, user: true, host: true });
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [isolate, setIsolate] = useState(false);
  const [zoomPct, setZoomPct] = useState(100);
  const stageRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const view = useRef({ zoom: 1, panX: 0, panY: 0 });
  const live = useRef({ kinds, selectedId, isolate });
  live.current = { kinds, selectedId, isolate };
  const controls = useRef<{ zoomBy: (f: number) => void; fit: () => void; reset: () => void } | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLinks(null);
    setSelectedId(null);
    setIsolate(false);
    getEntityLinks(investigationId, filter, LIMIT)
      .then((l) => !cancelled && setLinks(l))
      .catch((e: unknown) => !cancelled && setError(toCommandError(e).message));
    return () => {
      cancelled = true;
    };
  }, [investigationId, filter]);

  const graph = useMemo(() => {
    if (!links) return null;
    const g = layout(links);
    for (const n of g.nodes) {
      n.x *= SPREAD;
      n.y *= SPREAD;
    }
    return g;
  }, [links]);

  useEffect(() => {
    const stage = stageRef.current;
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!graph || !stage || !canvas || !ctx) return;
    const v = view.current;
    let width = 0;
    let height = 0;
    let raf = 0;
    let flow = 0;
    let packet = 0;
    let dragging: (typeof graph.nodes)[number] | null = null;
    let moved = false;
    let panning = false;
    let lastX = 0;
    let lastY = 0;
    const maxEdge = Math.max(...graph.edges.map((e) => e.events), 1);

    const visible = () => graph.nodes.filter((n) => live.current.kinds[n.kind]);
    const readout = () => setZoomPct(Math.round(v.zoom * 100));

    function fit() {
      const nodes = visible();
      if (nodes.length === 0) return;
      const minX = Math.min(...nodes.map((n) => n.x)) - 60;
      const maxX = Math.max(...nodes.map((n) => n.x)) + 60;
      const minY = Math.min(...nodes.map((n) => n.y)) - 50;
      const maxY = Math.max(...nodes.map((n) => n.y)) + 60;
      const bw = Math.max(100, maxX - minX);
      const bh = Math.max(100, maxY - minY);
      v.zoom = Math.min(1.3, Math.max(0.35, Math.min(width / bw, height / bh) * 0.92));
      v.panX = (width - bw * v.zoom) / 2 - minX * v.zoom;
      v.panY = (height - bh * v.zoom) / 2 - minY * v.zoom;
      readout();
    }

    function resize() {
      if (!canvas || !ctx || !stage) return;
      const dpr = window.devicePixelRatio || 1;
      width = stage.clientWidth;
      height = stage.clientHeight;
      canvas.width = Math.round(width * dpr);
      canvas.height = Math.round(height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    function neighbours(id: string) {
      const ids = new Set([id]);
      for (const e of graph!.edges) {
        if (e.a.id === id) ids.add(e.b.id);
        if (e.b.id === id) ids.add(e.a.id);
      }
      return ids;
    }

    function draw() {
      if (!ctx) return;
      const { kinds: k, selectedId: sel, isolate: iso } = live.current;
      ctx.clearRect(0, 0, width, height);
      // Grille du canevas, liée au zoom et au déplacement.
      ctx.strokeStyle = "rgba(34, 43, 58, 0.45)";
      ctx.lineWidth = 1;
      const step = 40 * v.zoom;
      ctx.beginPath();
      for (let x = v.panX % step; x < width; x += step) {
        ctx.moveTo(x, 0);
        ctx.lineTo(x, height);
      }
      for (let y = v.panY % step; y < height; y += step) {
        ctx.moveTo(0, y);
        ctx.lineTo(width, y);
      }
      ctx.stroke();

      ctx.save();
      ctx.translate(v.panX, v.panY);
      ctx.scale(v.zoom, v.zoom);
      const isoSet = iso && sel ? neighbours(sel) : null;

      for (const e of graph!.edges) {
        if (!k[e.a.kind] || !k[e.b.kind]) continue;
        const onPath = isoSet !== null && (e.a.id === sel || e.b.id === sel);
        const dimmed = isoSet !== null && !onPath;
        ctx.save();
        if (onPath) {
          ctx.strokeStyle = "#f0696a";
          ctx.lineWidth = 2.2;
          ctx.setLineDash([7, 5]);
          ctx.lineDashOffset = -flow;
        } else {
          ctx.strokeStyle = dimmed ? "rgba(49, 60, 79, 0.2)" : "#313c4f";
          ctx.lineWidth = 1 + 1.5 * (Math.log1p(e.events) / Math.log1p(maxEdge));
        }
        ctx.beginPath();
        ctx.moveTo(e.a.x, e.a.y);
        ctx.lineTo(e.b.x, e.b.y);
        ctx.stroke();
        ctx.setLineDash([]);
        // Flèche au milieu : sens source → destination.
        const mx = (e.a.x + e.b.x) / 2;
        const my = (e.a.y + e.b.y) / 2;
        const angle = Math.atan2(e.b.y - e.a.y, e.b.x - e.a.x);
        ctx.fillStyle = onPath ? "#f0696a" : dimmed ? "rgba(49, 60, 79, 0.2)" : "#4c5b75";
        ctx.beginPath();
        ctx.moveTo(mx + Math.cos(angle) * 6, my + Math.sin(angle) * 6);
        ctx.lineTo(mx + Math.cos(angle + 2.5) * 5, my + Math.sin(angle + 2.5) * 5);
        ctx.lineTo(mx + Math.cos(angle - 2.5) * 5, my + Math.sin(angle - 2.5) * 5);
        ctx.closePath();
        ctx.fill();
        if (onPath) {
          const px = e.a.x + (e.b.x - e.a.x) * packet;
          const py = e.a.y + (e.b.y - e.a.y) * packet;
          ctx.fillStyle = "#ffffff";
          ctx.fillRect(px - 3, py - 3, 6, 6);
          ctx.strokeStyle = "#f0696a";
          ctx.lineWidth = 1.5;
          ctx.strokeRect(px - 4.5, py - 4.5, 9, 9);
        }
        ctx.font = '9px "IBM Plex Mono", monospace';
        ctx.fillStyle = onPath ? "#f0696a" : dimmed ? "rgba(135, 146, 164, 0.22)" : "#8792a4";
        ctx.textAlign = "center";
        ctx.fillText(`${e.events.toLocaleString("fr-FR")} év.`, mx, my - 7);
        ctx.restore();
      }

      for (const n of graph!.nodes) {
        if (!k[n.kind]) continue;
        const color = KIND[n.kind].color;
        const isSel = n.id === sel;
        const inIso = isoSet?.has(n.id) ?? false;
        ctx.save();
        ctx.globalAlpha = isoSet && !inIso ? 0.18 : 1;
        ctx.fillStyle = "#111620";
        ctx.strokeStyle = isSel ? "#ffffff" : inIso ? "#f0696a" : color;
        ctx.lineWidth = isSel ? 2.2 : inIso ? 2 : 1.4;
        if (n.kind === "ip") {
          ctx.beginPath();
          ctx.moveTo(n.x, n.y - 16);
          ctx.lineTo(n.x + 16, n.y);
          ctx.lineTo(n.x, n.y + 16);
          ctx.lineTo(n.x - 16, n.y);
          ctx.closePath();
          ctx.fill();
          ctx.stroke();
        } else {
          ctx.fillRect(n.x - 15, n.y - 15, 30, 30);
          ctx.strokeRect(n.x - 15, n.y - 15, 30, 30);
        }
        ctx.fillStyle = color;
        ctx.fillRect(n.x - 3, n.y - 3, 6, 6);
        ctx.font = '600 11px "IBM Plex Mono", monospace';
        ctx.fillStyle = "#e4e8ef";
        ctx.textAlign = "center";
        ctx.fillText(n.name.length > 22 ? `${n.name.slice(0, 21)}…` : n.name, n.x, n.y + 29);
        ctx.restore();
      }
      ctx.restore();
    }

    function animate() {
      if (live.current.isolate) {
        flow = (flow + 0.7) % 24;
        packet = (packet + 0.012) % 1;
      }
      draw();
      raf = requestAnimationFrame(animate);
    }

    const toWorld = (cx: number, cy: number) => {
      const r = canvas.getBoundingClientRect();
      return { x: (cx - r.left - v.panX) / v.zoom, y: (cy - r.top - v.panY) / v.zoom };
    };
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const r = canvas.getBoundingClientRect();
      const mx = e.clientX - r.left;
      const my = e.clientY - r.top;
      const next = Math.min(2.2, Math.max(0.35, v.zoom * (e.deltaY < 0 ? 1.12 : 0.89)));
      v.panX = mx - (mx - v.panX) * (next / v.zoom);
      v.panY = my - (my - v.panY) * (next / v.zoom);
      v.zoom = next;
      readout();
    };
    const onDown = (e: MouseEvent) => {
      const p = toWorld(e.clientX, e.clientY);
      const hit = visible().find((n) => Math.hypot(n.x - p.x, n.y - p.y) < 22);
      moved = false;
      if (hit) {
        dragging = hit;
        setSelectedId(hit.id);
        return;
      }
      panning = true;
      stage.classList.add("graph-stage--panning");
      lastX = e.clientX;
      lastY = e.clientY;
    };
    const onMove = (e: MouseEvent) => {
      if (dragging) {
        const p = toWorld(e.clientX, e.clientY);
        dragging.x = p.x;
        dragging.y = p.y;
        moved = true;
      } else if (panning) {
        v.panX += e.clientX - lastX;
        v.panY += e.clientY - lastY;
        lastX = e.clientX;
        lastY = e.clientY;
        moved = true;
      }
    };
    const onUp = () => {
      // Un clic sur le fond (sans déplacement) désélectionne.
      if (panning && !moved) {
        setSelectedId(null);
        setIsolate(false);
      }
      dragging = null;
      panning = false;
      stage.classList.remove("graph-stage--panning");
    };

    controls.current = {
      zoomBy: (f) => {
        v.zoom = Math.min(2.2, Math.max(0.35, v.zoom * f));
        readout();
      },
      fit,
      reset: () => {
        v.zoom = 1;
        v.panX = 16;
        v.panY = 20;
        readout();
      },
    };

    resize();
    fit();
    const observer = new ResizeObserver(resize);
    observer.observe(stage);
    canvas.addEventListener("wheel", onWheel, { passive: false });
    canvas.addEventListener("mousedown", onDown);
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
    raf = requestAnimationFrame(animate);
    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      canvas.removeEventListener("wheel", onWheel);
      canvas.removeEventListener("mousedown", onDown);
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
      controls.current = null;
    };
  }, [graph]);

  if (error) return <p className="text-muted">Graphe indisponible : {error}</p>;
  if (!graph)
    return (
      <p className="page-inline">
        <Spinner size="sm" /> Calcul du graphe…
      </p>
    );
  if (graph.nodes.length === 0)
    return <p className="text-muted">Aucun lien : les événements retenus ne citent pas deux entités à la fois.</p>;

  const shown = graph.nodes.filter((n) => kinds[n.kind]);
  const shownEdges = graph.edges.filter((e) => kinds[e.a.kind] && kinds[e.b.kind]);
  const selected = graph.nodes.find((n) => n.id === selectedId) ?? null;
  const relations = selected ? graph.edges.filter((e) => e.a.id === selected.id || e.b.id === selected.id) : [];

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      notify("Valeur copiée.");
    } catch {
      notify("Copie impossible : sélectionnez la valeur à la main.", "danger");
    }
  }

  return (
    <div className="entity-graph">
      <div className="graph-toolbar">
        <div className="graph-toolbar__group">
          <span className="section-title">Entités :</span>
          {(Object.keys(KIND) as EntityKind[]).map((k) => (
            <Button
              key={k}
              size="sm"
              aria-pressed={kinds[k]}
              onClick={() => setKinds((cur) => ({ ...cur, [k]: !cur[k] }))}
            >
              {KIND[k].plural}
            </Button>
          ))}
        </div>
        <span className="mono graph-toolbar__count">
          {shown.length} entités · {shownEdges.length} relations (les {LIMIT} plus fréquentes au plus)
        </span>
      </div>

      <div className="graph-layout">
        <div className="graph-stage" ref={stageRef}>
          <canvas ref={canvasRef} className="graph-stage__canvas" aria-hidden="true" />
          <div className="graph-hud">
            <button type="button" title="Zoom arrière" aria-label="Zoom arrière" onClick={() => controls.current?.zoomBy(0.85)}>
              −
            </button>
            <span className="graph-hud__readout">{zoomPct} %</span>
            <button type="button" title="Zoom avant" aria-label="Zoom avant" onClick={() => controls.current?.zoomBy(1.15)}>
              +
            </button>
            <button type="button" title="Ajuster à la vue" onClick={() => controls.current?.fit()}>
              Ajuster
            </button>
            <button type="button" title="Taille réelle" onClick={() => controls.current?.reset()}>
              1:1
            </button>
          </div>
          <div className="graph-legend" aria-hidden="true">
            {(Object.keys(KIND) as EntityKind[]).map((k) => (
              <span key={k} className="graph-legend__item">
                <span className={`graph-legend__swatch graph-legend__swatch--${k}`} style={{ background: KIND[k].css }} />
                {KIND[k].label}
              </span>
            ))}
          </div>
          {/* Équivalent accessible du canevas : chaque entité est sélectionnable au clavier. */}
          <ul className="sr-only" aria-label="Entités du graphe">
            {shown.map((n) => (
              <li key={n.id}>
                <button type="button" onClick={() => setSelectedId(n.id)}>
                  Inspecter {KIND[n.kind].label} {n.name} ({n.weight} événements liés)
                </button>
              </li>
            ))}
          </ul>
        </div>

        <section className="ui-panel graph-inspector" aria-label="Inspecteur d'entité">
          <header className="ui-panel__header">
            <h3 className="ui-panel__title">Inspecteur d'entité</h3>
            {selected && <span className={`ui-badge ui-badge--${selected.kind === "ip" ? "danger" : selected.kind === "user" ? "warning" : "accent"}`}>{KIND[selected.kind].label}</span>}
          </header>
          <div className="ui-panel__body graph-inspector__body">
            {!selected ? (
              <p className="text-muted">
                Cliquez sur une entité du graphe pour voir ses relations. Molette : zoom ; glisser le fond : déplacer ;
                glisser une entité : la déplacer.
              </p>
            ) : (
              <>
                <div>
                  <p className="graph-inspector__name mono">{selected.name}</p>
                  <p className="text-muted graph-inspector__sub">
                    {selected.weight.toLocaleString("fr-FR")} événement{selected.weight > 1 ? "s" : ""} liés
                  </p>
                </div>
                <div>
                  <p className="section-title graph-inspector__title">Relations ({relations.length})</p>
                  <ul className="graph-inspector__relations">
                    {relations.map((r) => {
                      const outgoing = r.a.id === selected.id;
                      const other = outgoing ? r.b : r.a;
                      return (
                        <li key={`${r.a.id}-${r.b.id}`}>
                          <button type="button" onClick={() => setSelectedId(other.id)}>
                            <span className="mono">
                              {outgoing ? "→" : "←"} {other.name}
                            </span>
                            <span className="mono graph-inspector__count">{r.events.toLocaleString("fr-FR")} év.</span>
                          </button>
                        </li>
                      );
                    })}
                  </ul>
                </div>
                <div className="graph-inspector__actions">
                  <Button size="sm" variant="primary" onClick={() => onPivot({ kind: selected.kind, value: selected.name })}>
                    Afficher ses événements
                  </Button>
                  <Button size="sm" aria-pressed={isolate} onClick={() => setIsolate((x) => !x)}>
                    {isolate ? "Tout afficher" : "Isoler les voisins"}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => void copy(selected.name)}>
                    Copier
                  </Button>
                </div>
              </>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
