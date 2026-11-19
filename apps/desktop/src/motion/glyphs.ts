// Outils partagés par les animations ASCII.

/** Caractères utilisés pour le brouillage (avant que le vrai caractère apparaisse). */
export const SCRAMBLE_GLYPHS = "01#%&*+=<>/\\|:;$@?!~^";

/** Caractères du fond animé : hexadécimal et ponctuation, évoquant des données brutes. */
export const FIELD_GLYPHS = "0123456789abcdef:.-/=+";

export function randomGlyph(set: string): string {
  return set[Math.floor(Math.random() * set.length)] ?? " ";
}

/** Police monospace des animations (embarquée), avec repli. */
export const MONO_FONT = '"IBM Plex Mono", ui-monospace, monospace';

/** Lit une variable CSS de couleur du thème. */
export function cssColor(name: string, fallback: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}
