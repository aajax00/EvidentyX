import { useEffect, useRef } from "react";
import { FIELD_GLYPHS, MONO_FONT, randomGlyph } from "./glyphs";
import { useReducedMotion } from "./useReducedMotion";

export interface AsciiFieldProps {
  className?: string;
  /** Taille des caractères en pixels. */
  fontSize?: number;
}

const FPS = 10;
const CHANGE_RATE = 0.02;

/**
 * Fond décoratif : grille de caractères qui changent lentement, traversée par
 * une bande plus lumineuse. Purement visuel (aria-hidden), en pause quand la
 * fenêtre est masquée, figé si le système demande moins d'animations.
 */
export function AsciiField({ className, fontSize = 12 }: AsciiFieldProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const reduced = useReducedMotion();

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    let cols = 0;
    let rows = 0;
    let cellW = 0;
    let cellH = 0;
    let grid: string[] = [];
    let frame = 0;
    let timer = 0;

    function layout() {
      if (!canvas || !ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const { width, height } = canvas.getBoundingClientRect();
      canvas.width = Math.max(1, Math.round(width * dpr));
      canvas.height = Math.max(1, Math.round(height * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.font = `${fontSize}px ${MONO_FONT}`;
      ctx.textBaseline = "top";
      cellW = ctx.measureText("0").width || fontSize * 0.6;
      cellH = fontSize * 1.5;
      cols = Math.ceil(width / cellW);
      rows = Math.ceil(height / cellH);
      grid = Array.from({ length: cols * rows }, () => (Math.random() < 0.55 ? randomGlyph(FIELD_GLYPHS) : " "));
    }

    function draw() {
      if (!canvas || !ctx) return;
      const { width, height } = canvas.getBoundingClientRect();
      ctx.clearRect(0, 0, width, height);
      // Bande lumineuse diagonale qui parcourt la grille.
      const band = (frame * 0.6) % (cols + rows + 30);
      for (let y = 0; y < rows; y++) {
        for (let x = 0; x < cols; x++) {
          const ch = grid[y * cols + x];
          if (!ch || ch === " ") continue;
          const distance = Math.abs(x + y - band);
          const glow = distance < 6 ? 1 - distance / 6 : 0;
          ctx.fillStyle = `rgba(61, 143, 240, ${0.16 + glow * 0.5})`;
          ctx.fillText(ch, x * cellW, y * cellH);
        }
      }
    }

    function tick() {
      frame++;
      const changes = Math.ceil(grid.length * CHANGE_RATE);
      for (let i = 0; i < changes; i++) {
        const index = Math.floor(Math.random() * grid.length);
        grid[index] = Math.random() < 0.55 ? randomGlyph(FIELD_GLYPHS) : " ";
      }
      draw();
    }

    function start() {
      window.clearInterval(timer);
      if (!reduced && !document.hidden) timer = window.setInterval(tick, 1000 / FPS);
    }

    layout();
    draw();
    void document.fonts.ready.then(() => {
      layout();
      draw();
    });
    start();

    const observer = new ResizeObserver(() => {
      layout();
      draw();
    });
    observer.observe(canvas);
    document.addEventListener("visibilitychange", start);
    return () => {
      window.clearInterval(timer);
      observer.disconnect();
      document.removeEventListener("visibilitychange", start);
    };
  }, [fontSize, reduced]);

  return <canvas ref={canvasRef} className={className} aria-hidden="true" />;
}
