// Modèle et filtrage des commandes de la palette (logique pure, testée).

export interface Command {
  id: string;
  label: string;
  group: string;
  /** Termes supplémentaires pris en compte par la recherche. */
  keywords?: readonly string[];
  /** Raccourci clavier associé, au format de `shortcuts.ts` (ex. "mod+b"). */
  shortcut?: string;
  run: () => void;
}

/** Minuscules, sans accents ni espaces superflus : « Paramètres » → « parametres ». */
export function normalize(text: string): string {
  return text
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .trim();
}

function score(command: Command, tokens: string[]): number | null {
  const label = normalize(command.label);
  const haystack = [label, normalize(command.group), ...(command.keywords ?? []).map(normalize)].join(" ");
  if (!tokens.every((t) => haystack.includes(t))) return null;
  const first = tokens[0] ?? "";
  if (label.startsWith(first)) return 0;
  if (label.split(/\s+/).some((word) => word.startsWith(first))) return 1;
  if (label.includes(first)) return 2;
  return 3;
}

/**
 * Filtre les commandes selon la saisie : chaque mot doit apparaître (sans tenir
 * compte des accents ni de la casse) dans le libellé, le groupe ou les mots-clés.
 * Tri : correspondance en début de libellé, puis en début de mot, puis ailleurs ;
 * l'ordre d'origine départage (tri stable, résultat déterministe).
 */
export function filterCommands(commands: readonly Command[], query: string): Command[] {
  const tokens = normalize(query).split(/\s+/).filter(Boolean);
  if (tokens.length === 0) return [...commands];
  return commands
    .map((command, index) => ({ command, index, rank: score(command, tokens) }))
    .filter((entry): entry is { command: Command; index: number; rank: number } => entry.rank !== null)
    .sort((a, b) => a.rank - b.rank || a.index - b.index)
    .map((entry) => entry.command);
}
