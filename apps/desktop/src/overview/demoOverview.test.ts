import { describe, expect, it } from "vitest";
import { demoOverview } from "./demoOverview";

describe("demoOverview", () => {
  it("est déterministe : même enquête, mêmes chiffres", () => {
    expect(demoOverview(42)).toEqual(demoOverview(42));
    expect(demoOverview(1)).not.toEqual(demoOverview(2));
  });

  it("reste cohérent : totaux et 24 heures", () => {
    for (const id of [1, 7, 99, 12345]) {
      const o = demoOverview(id);
      expect(o.hourly).toHaveLength(24);
      expect(o.hourly.reduce((a, b) => a + b, 0)).toBe(o.events);
      const s = o.severity;
      expect(s.low + s.medium + s.high + s.critical).toBe(o.alerts);
      expect(o.topSources).toHaveLength(o.sources);
      expect(o.hourly.every((n) => n >= 0)).toBe(true);
    }
  });
});
