import { describe, expect, it } from "vitest";
import { NAVIGATION, SECTION_TITLES, availablePages } from "./navigation";

describe("navigation", () => {
  it("chaque page disponible a un titre", () => {
    for (const item of availablePages()) {
      expect(SECTION_TITLES[item.page]).toBeTruthy();
    }
  });

  it("chaque section planifiée indique son échéance", () => {
    const planned = NAVIGATION.flatMap((g) => g.items).filter((i) => i.kind === "planned");
    expect(planned.length).toBeGreaterThan(0);
    for (const item of planned) {
      expect(item.kind === "planned" && item.plannedFor).toMatch(/^(étape|phase) /);
    }
  });

  it("les libellés sont uniques", () => {
    const labels = NAVIGATION.flatMap((g) => g.items.map((i) => i.label));
    expect(new Set(labels).size).toBe(labels.length);
  });
});
