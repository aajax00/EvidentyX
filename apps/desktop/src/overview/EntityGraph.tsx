import { useEffect, useMemo, useState } from "react";
import { getEntityLinks, toCommandError, type EntityKind, type EntityLink } from "../api";
import { Spinner } from "../components/ui";

/** Liens affichés au plus (les plus fréquents). */
const LIMIT = 60;
const W = 640;
const H = 340;

const KIND: Record<EntityKind, { label: string; color: string }> = {
  ip: { label: "Adresse IP", color: "var(--chart-1)" },
  user: { label: "Utilisateur", color: "var(--chart-2)" },
  host: { label: "Hôte", color: "var(--chart-3)" },
};

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
 * les événements qui les citent ensemble. Pas une corrélation : un relevé.
 */
export function EntityGraph({ investigationId, version }: { investigationId: number; version: number }) {
  const [links, setLinks] = useState<EntityLink[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [hover, setHover] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    getEntityLinks(investigationId, LIMIT)
      .then((l) => !cancelled && setLinks(l))
      .catch((e: unknown) => !cancelled && setError(toCommandError(e).message));
    return () => {
      cancelled = true;
    };
  }, [investigationId, version]);

  const graph = useMemo(() => (links ? layout(links) : null), [links]);

  if (error) return <p className="text-muted">Graphe indisponible : {error}</p>;
  if (!graph)
    return (
      <p className="page-inline">
        <Spinner size="sm" /> Calcul du graphe…
      </p>
    );
  if (graph.nodes.length === 0)
    return <p className="text-muted">Aucun lien : les événements ne citent pas deux entités à la fois.</p>;

  const maxEdge = Math.max(...graph.edges.map((e) => e.events), 1);
  const maxWeight = Math.max(...graph.nodes.map((n) => n.weight), 1);
  const neighbors = new Set<string>();
  if (hover)
    for (const e of graph.edges) {
      if (e.a.id === hover) neighbors.add(e.b.id);
      if (e.b.id === hover) neighbors.add(e.a.id);
    }
  const labelled = new Set([...graph.nodes].sort((a, b) => b.weight - a.weight).slice(0, 8).map((n) => n.id));
  const hovered = graph.nodes.find((n) => n.id === hover);

  return (
    <div className="entity-graph">
      <svg viewBox={`0 0 ${W} ${H}`} className="entity-graph__svg" role="img" aria-label={`Graphe de ${graph.nodes.length} entités`}>
        {graph.edges.map((e) => {
          const active = hover !== null && (e.a.id === hover || e.b.id === hover);
          return (
            <line
              key={`${e.a.id}-${e.b.id}`}
              x1={e.a.x}
              y1={e.a.y}
              x2={e.b.x}
              y2={e.b.y}
              className={`entity-graph__edge${active ? " entity-graph__edge--active" : ""}`}
              strokeWidth={1 + 2.5 * (Math.log1p(e.events) / Math.log1p(maxEdge))}
              opacity={hover && !active ? 0.12 : undefined}
            />
          );
        })}
        {graph.nodes.map((n) => {
          const size = 7 + 9 * Math.sqrt(n.weight / maxWeight);
          const dim = hover !== null && n.id !== hover && !neighbors.has(n.id);
          return (
            <g
              key={n.id}
              className="entity-graph__node"
              opacity={dim ? 0.25 : 1}
              onMouseEnter={() => setHover(n.id)}
              onMouseLeave={() => setHover(null)}
            >
              <rect
                x={n.x - size / 2}
                y={n.y - size / 2}
                width={size}
                height={size}
                fill={`color-mix(in srgb, ${KIND[n.kind].color} 22%, var(--color-surface))`}
                stroke={KIND[n.kind].color}
                strokeWidth={n.id === hover ? 2 : 1.25}
              />
              {(labelled.has(n.id) || n.id === hover) && (
                <text x={n.x + size / 2 + 5} y={n.y + 4} className="entity-graph__label">
                  {n.name}
                </text>
              )}
            </g>
          );
        })}
      </svg>
      <div className="entity-graph__footer">
        <ul className="entity-graph__legend">
          {(Object.keys(KIND) as EntityKind[]).map((k) => (
            <li key={k}>
              <span style={{ borderColor: KIND[k].color }} aria-hidden="true" />
              {KIND[k].label}
            </li>
          ))}
        </ul>
        <span className="entity-graph__hint mono">
          {hovered
            ? `${KIND[hovered.kind].label} ${hovered.name} · ${hovered.weight.toLocaleString("fr-FR")} événements liés`
            : `${graph.nodes.length} entités · ${graph.edges.length} liens les plus fréquents`}
        </span>
      </div>
    </div>
  );
}
