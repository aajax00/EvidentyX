import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { CommandPalette } from "./CommandPalette";
import type { Command } from "./commands";
import { Header } from "./Header";
import { PAGE_TITLES, availablePages, type PageId } from "./navigation";
import { matchesShortcut } from "./shortcuts";
import { Sidebar } from "./Sidebar";
import "./shell.css";

export interface AppShellProps {
  page: PageId;
  onNavigate: (page: PageId) => void;
  children: ReactNode;
}

type Theme = "dark" | "light";

function currentTheme(): Theme {
  return document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
}

/**
 * Coque de l'application : barre latérale, en-tête, zone de contenu et
 * palette de commandes. Raccourcis globaux : mod+K (palette), mod+B (barre latérale).
 */
export function AppShell({ page, onNavigate, children }: AppShellProps) {
  const [collapsed, setCollapsed] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [theme, setTheme] = useState<Theme>(currentTheme);

  const toggleCollapsed = useCallback(() => setCollapsed((c) => !c), []);

  useEffect(() => {
    if (theme === "light") document.documentElement.setAttribute("data-theme", "light");
    else document.documentElement.removeAttribute("data-theme");
  }, [theme]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (matchesShortcut(event, "mod+k")) {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      } else if (matchesShortcut(event, "mod+b")) {
        event.preventDefault();
        toggleCollapsed();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [toggleCollapsed]);

  const commands = useMemo<Command[]>(
    () => [
      ...availablePages().map((item) => ({
        id: `nav-${item.page}`,
        label: `Aller à : ${item.label}`,
        group: "Navigation",
        keywords: ["page", "ouvrir"],
        run: () => onNavigate(item.page),
      })),
      {
        id: "toggle-sidebar",
        label: collapsed ? "Déplier la barre latérale" : "Replier la barre latérale",
        group: "Affichage",
        keywords: ["menu", "navigation"],
        shortcut: "mod+b",
        run: toggleCollapsed,
      },
      {
        id: "toggle-theme",
        label: theme === "dark" ? "Passer au thème clair" : "Passer au thème sombre",
        group: "Affichage",
        keywords: ["apparence", "couleurs", "mode"],
        run: () => setTheme((t) => (t === "dark" ? "light" : "dark")),
      },
    ],
    [collapsed, theme, onNavigate, toggleCollapsed],
  );

  return (
    <div className="shell">
      <Sidebar current={page} collapsed={collapsed} onNavigate={onNavigate} onToggleCollapsed={toggleCollapsed} />
      <div className="shell-main">
        <Header title={PAGE_TITLES[page]} onOpenPalette={() => setPaletteOpen(true)} />
        <main className="shell-content">{children}</main>
      </div>
      {paletteOpen && <CommandPalette commands={commands} onClose={() => setPaletteOpen(false)} />}
    </div>
  );
}
