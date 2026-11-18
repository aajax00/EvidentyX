import { describe, expect, it } from "vitest";
import { filterCommands, normalize, type Command } from "./commands";

const noop = () => {};
const cmd = (id: string, label: string, group = "Navigation", keywords?: string[]): Command => ({
  id,
  label,
  group,
  run: noop,
  ...(keywords ? { keywords } : {}),
});

const COMMANDS: Command[] = [
  cmd("home", "Aller à : Accueil"),
  cmd("about", "Aller à : À propos"),
  cmd("sidebar", "Replier la barre latérale", "Affichage", ["menu"]),
  cmd("theme", "Passer au thème clair", "Affichage", ["apparence"]),
];

const ids = (list: Command[]) => list.map((c) => c.id);

describe("normalize", () => {
  it("supprime accents, casse et espaces superflus", () => {
    expect(normalize("  Paramètres À PROPOS ")).toBe("parametres a propos");
  });
});

describe("filterCommands", () => {
  it("renvoie toutes les commandes, dans l'ordre, pour une saisie vide", () => {
    expect(ids(filterCommands(COMMANDS, "   "))).toEqual(["home", "about", "sidebar", "theme"]);
  });

  it("ignore les accents et la casse", () => {
    expect(ids(filterCommands(COMMANDS, "THEME"))).toEqual(["theme"]);
    expect(ids(filterCommands(COMMANDS, "latérale"))).toEqual(["sidebar"]);
  });

  it("cherche aussi dans le groupe et les mots-clés", () => {
    expect(ids(filterCommands(COMMANDS, "menu"))).toEqual(["sidebar"]);
    expect(ids(filterCommands(COMMANDS, "affichage"))).toEqual(["sidebar", "theme"]);
  });

  it("exige que tous les mots correspondent", () => {
    expect(ids(filterCommands(COMMANDS, "aller propos"))).toEqual(["about"]);
    expect(ids(filterCommands(COMMANDS, "aller inexistant"))).toEqual([]);
  });

  it("classe les correspondances en début de libellé ou de mot en premier", () => {
    const list = [cmd("a", "Exporter le rapport"), cmd("b", "Rapport complet"), cmd("c", "Ouvrir un rapport")];
    expect(ids(filterCommands(list, "rapport"))).toEqual(["b", "a", "c"]);
  });

  it("est déterministe à pertinence égale (ordre d'origine)", () => {
    const list = [cmd("x", "Aller à : B"), cmd("y", "Aller à : A")];
    expect(ids(filterCommands(list, "aller"))).toEqual(["x", "y"]);
  });
});
