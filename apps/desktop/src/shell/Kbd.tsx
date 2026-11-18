import { formatShortcut } from "./shortcuts";

/** Affiche un raccourci clavier sous forme de touches. */
export function Kbd({ shortcut }: { shortcut: string }) {
  const keys = formatShortcut(shortcut);
  return (
    <span className="shell-kbd" aria-label={`Raccourci : ${keys.join(" + ")}`}>
      {keys.map((k) => (
        <kbd key={k}>{k}</kbd>
      ))}
    </span>
  );
}
