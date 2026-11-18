import { useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { Search } from "lucide-react";
import { cx } from "../components/ui/cx";
import { filterCommands, type Command } from "./commands";
import { Kbd } from "./Kbd";

export interface CommandPaletteProps {
  commands: readonly Command[];
  onClose: () => void;
}

/**
 * Palette de commandes modale (Ctrl+K / ⌘K).
 * Clavier : ↑ ↓ pour choisir, Entrée pour exécuter, Échap pour fermer.
 */
export function CommandPalette({ commands, onClose }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const listId = useId();

  const results = useMemo(() => filterCommands(commands, query), [commands, query]);
  const active = results[Math.min(activeIndex, results.length - 1)];

  // Focus sur le champ à l'ouverture, retour du focus à l'élément précédent à la fermeture.
  useEffect(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    inputRef.current?.focus();
    return () => previous?.focus();
  }, []);

  useEffect(() => {
    if (!active) return;
    listRef.current
      ?.querySelector(`[data-command-id="${CSS.escape(active.id)}"]`)
      ?.scrollIntoView({ block: "nearest" });
  }, [active]);

  function execute(command: Command | undefined) {
    if (!command) return;
    onClose();
    command.run();
  }

  function onKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        setActiveIndex((i) => (results.length === 0 ? 0 : (i + 1) % results.length));
        break;
      case "ArrowUp":
        event.preventDefault();
        setActiveIndex((i) => (results.length === 0 ? 0 : (i - 1 + results.length) % results.length));
        break;
      case "Enter":
        event.preventDefault();
        execute(active);
        break;
      case "Escape":
        event.preventDefault();
        onClose();
        break;
      case "Tab":
        // La palette est modale : le focus reste dans le champ.
        event.preventDefault();
        break;
    }
  }

  // Regroupe les résultats en conservant l'ordre de pertinence.
  const groups: Array<{ name: string; items: Command[] }> = [];
  for (const command of results) {
    const group = groups.find((g) => g.name === command.group);
    if (group) group.items.push(command);
    else groups.push({ name: command.group, items: [command] });
  }

  return (
    <div className="shell-palette-overlay" onMouseDown={onClose}>
      <div
        className="shell-palette"
        role="dialog"
        aria-modal="true"
        aria-label="Palette de commandes"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="shell-palette__search">
          <Search size={16} aria-hidden="true" />
          <input
            ref={inputRef}
            type="text"
            role="combobox"
            aria-expanded="true"
            aria-controls={listId}
            aria-activedescendant={active ? `${listId}-${active.id}` : undefined}
            aria-autocomplete="list"
            placeholder="Rechercher une commande…"
            spellCheck={false}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setActiveIndex(0);
            }}
            onKeyDown={onKeyDown}
          />
          <Kbd shortcut="Échap" />
        </div>

        <ul ref={listRef} id={listId} role="listbox" className="shell-palette__list" aria-label="Commandes">
          {results.length === 0 && <li className="shell-palette__empty">Aucune commande ne correspond.</li>}
          {groups.map((group) => (
            <li key={group.name} role="presentation">
              <p className="shell-palette__group section-title" aria-hidden="true">
                {group.name}
              </p>
              <ul role="group" aria-label={group.name}>
                {group.items.map((command) => (
                  <li
                    key={command.id}
                    id={`${listId}-${command.id}`}
                    data-command-id={command.id}
                    role="option"
                    aria-selected={command === active}
                    className={cx("shell-palette__item", command === active && "shell-palette__item--active")}
                    onMouseMove={() => setActiveIndex(results.indexOf(command))}
                    onClick={() => execute(command)}
                  >
                    <span>{command.label}</span>
                    {command.shortcut && <Kbd shortcut={command.shortcut} />}
                  </li>
                ))}
              </ul>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
