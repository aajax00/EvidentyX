import { useEffect, useRef } from "react";

/** Densité : un point pour cette surface (px²), plafonnée. */
const AREA_PER_NODE = 16000;
const MAX_NODES = 110;
/** Distance maximale d'un lien entre deux points (px). */
const LINK_DISTANCE = 140;
/** Vitesse de dérive (px par image à 30 images/s). */
const SPEED = 0.18;
const FRAME_MS = 1000 / 30;

interface Node {
  x: number;
  y: number;
  vx: number;
  vy: number;
  /** Phase de pulsation, pour quelques points plus lumineux. */
  phase: number;
}

/**
 * Animation d'arrière-plan : des points carrés dérivent lentement et se relient
 * quand ils se rapprochent, comme un graphe d'entités en cours de construction.
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
      const count = Math.min(MAX_NODES, Math.round((width * height) / AREA_PER_NODE));
      // Conserve les points existants pour éviter un saut visuel au redimensionnement.
      nodes = nodes.filter((n) => n.x <= width && n.y <= height).slice(0, count);
      while (nodes.length < count) {
        const angle = Math.random() * Math.PI * 2;
        nodes.push({
          x: Math.random() * width,
          y: Math.random() * height,
          vx: Math.cos(angle) * SPEED * (0.4 + Math.random()),
          vy: Math.sin(angle) * SPEED * (0.4 + Math.random()),
          phase: Math.random() * Math.PI * 2,
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
        n.phase += 0.02;
      }

      ctx.clearRect(0, 0, width, height);
      ctx.lineWidth = 1;
      for (let i = 0; i < nodes.length; i++) {
        const a = nodes[i]!;
        for (let j = i + 1; j < nodes.length; j++) {
          const b = nodes[j]!;
          const dx = a.x - b.x;
          const dy = a.y - b.y;
          const d2 = dx * dx + dy * dy;
          if (d2 > LINK_DISTANCE * LINK_DISTANCE) continue;
          const alpha = (1 - Math.sqrt(d2) / LINK_DISTANCE) * 0.2;
          ctx.strokeStyle = `rgba(61, 143, 240, ${alpha})`;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      }
      for (const n of nodes) {
        const glow = Math.max(0, Math.sin(n.phase)) ** 6;
        ctx.fillStyle = `rgba(99, 166, 244, ${0.38 + glow * 0.5})`;
        const size = 2 + glow;
        ctx.fillRect(n.x - size / 2, n.y - size / 2, size, size);
      }
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
