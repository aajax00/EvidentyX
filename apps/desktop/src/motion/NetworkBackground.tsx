import { useEffect, useRef } from "react";

/** Densité : un point pour cette surface (px²), entre 12 et 26 points. */
const AREA_PER_NODE = 62000;
const MIN_NODES = 12;
const MAX_NODES = 26;
/** Distance maximale d'un lien entre deux points (px). */
const LINK_DISTANCE = 170;
/** Pas de la grille de coordonnées (px). */
const GRID = 56;
/** Vitesse de dérive (px par image à 30 images/s). */
const SPEED = 0.11;
const FRAME_MS = 1000 / 30;

interface Node {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

/**
 * Animation d'arrière-plan « forensique » : grille de coordonnées très discrète
 * et quelques points carrés qui dérivent lentement, reliés en équerre quand ils
 * se rapprochent.
 * Purement décorative (aria-hidden), en pause quand la fenêtre est masquée.
 * Le parent décide de l'afficher (réglage utilisateur, réduction des animations).
 */
export function NetworkBackground({ className }: { className?: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    let width = 0;
    let height = 0;
    let nodes: Node[] = [];
    let raf = 0;
    let last = 0;

    function layout() {
      if (!canvas || !ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const rect = canvas.getBoundingClientRect();
      width = rect.width;
      height = rect.height;
      canvas.width = Math.max(1, Math.round(width * dpr));
      canvas.height = Math.max(1, Math.round(height * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const count = Math.min(MAX_NODES, Math.max(MIN_NODES, Math.round((width * height) / AREA_PER_NODE)));
      // Conserve les points existants pour éviter un saut visuel au redimensionnement.
      nodes = nodes.filter((n) => n.x <= width && n.y <= height).slice(0, count);
      while (nodes.length < count) {
        const angle = Math.random() * Math.PI * 2;
        nodes.push({
          x: Math.random() * width,
          y: Math.random() * height,
          vx: Math.cos(angle) * SPEED * (0.4 + Math.random()),
          vy: Math.sin(angle) * SPEED * (0.4 + Math.random()),
        });
      }
    }

    function step(now: number) {
      raf = requestAnimationFrame(step);
      if (now - last < FRAME_MS) return;
      last = now;
      if (!ctx) return;

      for (const n of nodes) {
        n.x += n.vx;
        n.y += n.vy;
        if (n.x < 0 || n.x > width) n.vx *= -1;
        if (n.y < 0 || n.y > height) n.vy *= -1;
      }

      ctx.clearRect(0, 0, width, height);
      ctx.lineWidth = 1;

      // Grille de coordonnées très discrète.
      ctx.strokeStyle = "rgba(34, 43, 58, 0.22)";
      ctx.beginPath();
      for (let x = 0; x < width; x += GRID) {
        ctx.moveTo(x + 0.5, 0);
        ctx.lineTo(x + 0.5, height);
      }
      for (let y = 0; y < height; y += GRID) {
        ctx.moveTo(0, y + 0.5);
        ctx.lineTo(width, y + 0.5);
      }
      ctx.stroke();

      // Liaisons orthogonales (en équerre) entre points proches.
      for (let i = 0; i < nodes.length; i++) {
        const a = nodes[i]!;
        for (let j = i + 1; j < nodes.length; j++) {
          const b = nodes[j]!;
          const d = Math.hypot(a.x - b.x, a.y - b.y);
          if (d > LINK_DISTANCE) continue;
          ctx.strokeStyle = `rgba(61, 143, 240, ${((1 - d / LINK_DISTANCE) * 0.11).toFixed(3)})`;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      }
      ctx.fillStyle = "rgba(49, 60, 79, 0.55)";
      for (const n of nodes) ctx.fillRect(n.x - 1.5, n.y - 1.5, 3, 3);
    }

    function start() {
      cancelAnimationFrame(raf);
      if (!document.hidden) raf = requestAnimationFrame(step);
    }

    layout();
    start();
    const observer = new ResizeObserver(layout);
    observer.observe(canvas);
    document.addEventListener("visibilitychange", start);
    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      document.removeEventListener("visibilitychange", start);
    };
  }, []);

  return <canvas ref={canvasRef} className={className} aria-hidden="true" />;
}
