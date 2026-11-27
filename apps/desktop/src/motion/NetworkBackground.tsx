import { useEffect, useRef } from "react";

/** Pas de la grille de coordonnées (px). */
const GRID = 56;
/** Distance maximale d'une liaison entre deux points (px). */
const LINK_DISTANCE = 170;
const FRAME_MS = 33;

interface Node {
  x: number;
  y: number;
  vx: number;
  vy: number;
}

/**
 * Arrière-plan « forensique » (repris de la maquette EvidentyX) : grille de
 * coordonnées très discrète et 12 à 26 points carrés qui dérivent lentement,
 * reliés en équerre quand ils se rapprochent. Purement décoratif
 * (aria-hidden), en pause quand la fenêtre est masquée. Le parent décide de
 * l'afficher (réglage utilisateur, réduction des animations).
 */
export function NetworkBackground({ className }: { className?: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    let w = 0;
    let h = 0;
    let nodes: Node[] = [];
    let raf = 0;
    let last = 0;

    function resize() {
      if (!canvas || !ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const rect = canvas.getBoundingClientRect();
      w = rect.width;
      h = rect.height;
      canvas.width = Math.max(1, Math.round(w * dpr));
      canvas.height = Math.max(1, Math.round(h * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      const count = Math.min(26, Math.max(12, Math.floor((w * h) / 62000)));
      nodes = Array.from({ length: count }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * 0.22,
        vy: (Math.random() - 0.5) * 0.22,
      }));
    }

    function draw(now: number) {
      raf = requestAnimationFrame(draw);
      if (now - last < FRAME_MS || !ctx) return;
      last = now;
      ctx.clearRect(0, 0, w, h);

      // Grille orthogonale très discrète.
      ctx.strokeStyle = "rgba(34, 43, 58, 0.22)";
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let x = 0; x < w; x += GRID) {
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h);
      }
      for (let y = 0; y < h; y += GRID) {
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
      }
      ctx.stroke();

      // Points et liaisons en équerre.
      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i]!;
        n.x += n.vx;
        n.y += n.vy;
        if (n.x < 10 || n.x > w - 10) n.vx *= -1;
        if (n.y < 10 || n.y > h - 10) n.vy *= -1;
        for (let j = i + 1; j < nodes.length; j++) {
          const m = nodes[j]!;
          const dist = Math.hypot(n.x - m.x, n.y - m.y);
          if (dist >= LINK_DISTANCE) continue;
          ctx.strokeStyle = `rgba(61, 143, 240, ${((1 - dist / LINK_DISTANCE) * 0.11).toFixed(3)})`;
          ctx.beginPath();
          ctx.moveTo(n.x, n.y);
          ctx.lineTo(m.x, n.y);
          ctx.lineTo(m.x, m.y);
          ctx.stroke();
        }
        ctx.fillStyle = "rgba(49, 60, 79, 0.55)";
        ctx.fillRect(n.x - 1.5, n.y - 1.5, 3, 3);
      }
    }

    function start() {
      cancelAnimationFrame(raf);
      if (!document.hidden) raf = requestAnimationFrame(draw);
    }

    resize();
    start();
    const observer = new ResizeObserver(resize);
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
