import { describe, expect, it } from "vitest";
import { layout } from "./EntityGraph";

describe("layout", () => {
  it("fusionne les entités et reste dans le cadre", () => {
    const g = layout([
      { fromKind: "ip", from: "10.0.0.1", toKind: "ip", to: "10.0.0.9", events: 5 },
      { fromKind: "user", from: "root", toKind: "ip", to: "10.0.0.1", events: 2 },
      { fromKind: "ip", from: "10.0.0.1", toKind: "host", to: "srv", events: 1 },
    ]);
    expect(g.nodes.map((n) => n.id).sort()).toEqual(["host:srv", "ip:10.0.0.1", "ip:10.0.0.9", "user:root"]);
    expect(g.nodes.find((n) => n.id === "ip:10.0.0.1")?.weight).toBe(8);
    for (const n of g.nodes) {
      expect(n.x).toBeGreaterThanOrEqual(40);
      expect(n.x).toBeLessThanOrEqual(600);
      expect(n.y).toBeGreaterThanOrEqual(24);
      expect(n.y).toBeLessThanOrEqual(316);
      expect(Number.isFinite(n.x) && Number.isFinite(n.y)).toBe(true);
    }
  });

  it("supporte un graphe vide", () => {
    expect(layout([])).toEqual({ nodes: [], edges: [] });
  });
});
