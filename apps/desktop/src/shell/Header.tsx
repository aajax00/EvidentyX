import { Search } from "lucide-react";
import { useEngine } from "../engine";
import { Kbd } from "./Kbd";

export interface HeaderProps {
  title: string;
  onOpenPalette: () => void;
}

export function Header({ title, onOpenPalette }: HeaderProps) {
  return (
    <header className="shell-header">
      <h1 className="shell-header__title">{title}</h1>

      <button type="button" className="shell-header__palette" onClick={onOpenPalette}>
        <Search size={14} aria-hidden="true" />
        <span>Rechercher une commande…</span>
        <Kbd shortcut="mod+k" />
      </button>

      <EngineIndicator />
    </header>
  );
}

function EngineIndicator() {
  const engine = useEngine();
  const { tone, text } =
    engine.kind === "ready"
      ? { tone: "success", text: `Moteur ${engine.info.engineVersion}` }
      : engine.kind === "loading"
        ? { tone: "neutral", text: "Moteur : connexion…" }
        : { tone: "danger", text: "Moteur indisponible" };
  return (
    <span
      className={`shell-engine shell-engine--${tone}`}
      role="status"
      title={engine.kind === "unavailable" ? engine.reason : undefined}
    >
      <span className="shell-engine__dot" aria-hidden="true" />
      {text}
    </span>
  );
}
