// Raccourcis clavier : « mod » désigne Ctrl (Windows, Linux) ou ⌘ (macOS).

export interface KeyLike {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

export function isMacPlatform(platform: string = navigator.userAgent): boolean {
  return /Mac|iPhone|iPad/i.test(platform);
}

interface ParsedShortcut {
  mod: boolean;
  alt: boolean;
  shift: boolean;
  key: string;
}

function parse(shortcut: string): ParsedShortcut {
  const parts = shortcut.split("+");
  const key = parts.pop() ?? "";
  for (let i = 0; i < parts.length; i++) parts[i] = (parts[i] ?? "").toLowerCase();
  return {
    mod: parts.includes("mod"),
    alt: parts.includes("alt"),
    shift: parts.includes("shift"),
    key,
  };
}

/** Vrai si l'événement clavier correspond exactement au raccourci. */
export function matchesShortcut(event: KeyLike, shortcut: string, mac: boolean = isMacPlatform()): boolean {
  const s = parse(shortcut);
  const modPressed = mac ? event.metaKey : event.ctrlKey;
  const otherModPressed = mac ? event.ctrlKey : event.metaKey;
  return (
    event.key.toLowerCase() === s.key.toLowerCase() &&
    modPressed === s.mod &&
    !otherModPressed &&
    event.altKey === s.alt &&
    event.shiftKey === s.shift
  );
}

/** Libellés des touches à afficher, ex. ["Ctrl", "K"] ou ["⌘", "K"]. */
export function formatShortcut(shortcut: string, mac: boolean = isMacPlatform()): string[] {
  const s = parse(shortcut);
  const keys: string[] = [];
  if (s.mod) keys.push(mac ? "⌘" : "Ctrl");
  if (s.alt) keys.push(mac ? "⌥" : "Alt");
  if (s.shift) keys.push(mac ? "⇧" : "Maj");
  keys.push(s.key.length === 1 ? s.key.toUpperCase() : s.key);
  return keys;
}
