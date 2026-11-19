import { useEffect, useRef, useState } from "react";
import logo from "../assets/logo.png";
import { LOGO_ASCII, LOGO_ASCII_LEVELS } from "../assets/logoAscii";
import { MONO_FONT, SCRAMBLE_GLYPHS, randomGlyph } from "./glyphs";
import "./splash.css";

/** Couleurs par niveau du logo ASCII (bleu nuit éclairci, bleu, bleu vif). */
const LEVEL_COLORS = ["transparent", "#2c5aa8", "#3d8ff0", "#8cc8ff"];

/** Chronologie de l'animation, en millisecondes. */
const DECODE_END = 1100;
const LOGO_IN = 1050;
const FADE_OUT = 2100;
const DONE = 2450;

const WORD = "EVIDENTYX";

/**
 * Écran d'ouverture : le logo se « décode » en ASCII (caractères brouillés qui
 * se fixent de gauche à droite), puis le logo réel apparaît et l'écran s'efface.
 * Un clic ou une touche le passe. Affiché une fois au lancement.
 */
export function SplashScreen({ onDone }: { onDone: () => void }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [phase, setPhase] = useState<"decode" | "logo" | "out">("decode");
  const [word, setWord] = useState(() => scramble(WORD, 0));
  const onDoneRef = useRef(onDone);
  onDoneRef.current = onDone;

  // Passer l'animation au clic ou au clavier.
  useEffect(() => {
    const skip = () => {
      setPhase("out");
      window.setTimeout(() => onDoneRef.current(), 250);
    };
    window.addEventListener("keydown", skip, { once: true });
    window.addEventListener("mousedown", skip, { once: true });
    return () => {
      window.removeEventListener("keydown", skip);
      window.removeEventListener("mousedown", skip);
    };
  }, []);

  useEffect(() => {
    const timers = [
      window.setTimeout(() => setPhase("logo"), LOGO_IN),
      window.setTimeout(() => setPhase("out"), FADE_OUT),
      window.setTimeout(() => onDoneRef.current(), DONE),
    ];
    return () => timers.forEach((t) => window.clearTimeout(t));
  }, []);

  // Décodage du mot « EVIDENTYX ».
  useEffect(() => {
    const start = performance.now();
    let raf = 0;
    const step = (now: number) => {
      const progress = Math.min(1, (now - start - 500) / 900);
      setWord(scramble(WORD, progress));
      if (progress < 1) raf = requestAnimationFrame(step);
    };
    raf = requestAnimationFrame(step);
    return () => cancelAnimationFrame(raf);
  }, []);

  // Décodage du logo ASCII sur canvas.
  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    const rows = LOGO_ASCII.length;
    const cols = Math.max(...LOGO_ASCII.map((l) => l.length));
    // Instant où chaque cellule se fixe : de gauche à droite, avec un peu d'aléa.
    const revealAt = LOGO_ASCII.map((line) =>
      Array.from(line, (_, x) => 120 + (x / cols) * 700 + Math.random() * 240),
    );

    const fontSize = Math.max(8, Math.min(16, Math.floor(window.innerWidth / (cols * 0.68))));
    const dpr = window.devicePixelRatio || 1;
    ctx.font = `${fontSize}px ${MONO_FONT}`;
    const cellW = ctx.measureText("0").width || fontSize * 0.6;
    const cellH = fontSize * 1.18;
    const width = cols * cellW;
    const height = rows * cellH;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(height * dpr);
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.font = `${fontSize}px ${MONO_FONT}`;
    ctx.textBaseline = "top";

    const start = performance.now();
    let raf = 0;
    const draw = (now: number) => {
      const t = now - start;
      ctx.clearRect(0, 0, width, height);
      for (let y = 0; y < rows; y++) {
        const line = LOGO_ASCII[y] ?? "";
        const levels = LOGO_ASCII_LEVELS[y] ?? "";
        for (let x = 0; x < line.length; x++) {
          const ch = line[x];
          if (!ch || ch === " ") continue;
          const at = revealAt[y]?.[x] ?? 0;
          if (t < at - 260) continue;
          const settled = t >= at;
          ctx.fillStyle = settled ? (LEVEL_COLORS[Number(levels[x])] ?? LEVEL_COLORS[2]!) : "rgba(140, 200, 255, 0.55)";
          ctx.fillText(settled ? ch : randomGlyph(SCRAMBLE_GLYPHS), x * cellW, y * cellH);
        }
      }
      if (t < DECODE_END + 100) raf = requestAnimationFrame(draw);
    };
    void document.fonts.ready.then(() => {
      raf = requestAnimationFrame(draw);
    });
    return () => cancelAnimationFrame(raf);
  }, []);

  return (
    <div className={`splash splash--${phase}`} role="presentation" aria-hidden="true">
      <div className="splash__stage">
        <canvas ref={canvasRef} className="splash__ascii" />
        <img src={logo} alt="" className="splash__logo" draggable={false} />
      </div>
      <p className="splash__word">{word}</p>
      <p className="splash__tag">Chaque trace. Chaque connexion. Chaque vérité.</p>
    </div>
  );
}

/** Mot partiellement décodé : les lettres se fixent de gauche à droite selon `progress` (0 → 1). */
export function scramble(word: string, progress: number): string {
  const fixed = Math.floor(word.length * Math.max(0, Math.min(1, progress)));
  return Array.from(word, (ch, i) => (i < fixed ? ch : randomGlyph(SCRAMBLE_GLYPHS))).join("");
}
