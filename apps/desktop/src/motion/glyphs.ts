// Outils partagés par les animations ASCII.

/** Caractères utilisés pour le brouillage (avant que le vrai caractère apparaisse). */
export const SCRAMBLE_GLYPHS = "01#%&*+=<>/\\|:;$@?!~^";

export function randomGlyph(set: string): string {
  return set[Math.floor(Math.random() * set.length)] ?? " ";
}

/** Police monospace des animations (embarquée), avec repli. */
export const MONO_FONT = '"IBM Plex Mono", ui-monospace, monospace';
