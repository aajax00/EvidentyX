import { describe, expect, it } from "vitest";
import { layout, readingHints, relationLabel, relationSentence } from "./EntityGraph";

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

describe("relations", () => {
  const ip = (n: string) => ({ id: `ip:${n}`, kind: "ip" as const, name: n });
  const user = (n: string) => ({ id: `user:${n}`, kind: "user" as const, name: n });
  const host = (n: string) => ({ id: `host:${n}`, kind: "host" as const, name: n });

  it("nomme chaque type de lien et le décrit des deux côtés", () => {
    const conn = { a: ip("10.0.0.1"), b: ip("10.0.0.9"), events: 3 };
    const from = { a: user("root"), b: ip("10.0.0.1"), events: 2 };
    const seen = { a: ip("10.0.0.1"), b: host("srv"), events: 1 };
    expect([relationLabel(conn), relationLabel(from), relationLabel(seen)]).toEqual(["connexion", "depuis", "vue par"]);
    expect(relationSentence(conn, "ip:10.0.0.1")).toBe("se connecte à 10.0.0.9");
    expect(relationSentence(conn, "ip:10.0.0.9")).toBe("connexion reçue de 10.0.0.1");
    expect(relationSentence(from, "ip:10.0.0.1")).toBe("compte root utilisé depuis cette adresse");
    expect(relationSentence(seen, "host:srv")).toBe("a journalisé l'adresse 10.0.0.1");
  });

  it("signale plusieurs comptes depuis une même adresse", () => {
    const edges = ["root", "admin"].map((u) => ({ a: user(u), b: ip("10.0.0.1"), events: 1 }));
    expect(readingHints("ip", edges, "ip:10.0.0.1")[0]).toMatch(/^2 comptes utilisés depuis cette adresse/);
    expect(readingHints("ip", edges.slice(0, 1), "ip:10.0.0.1")).toEqual([]);
  });
});
