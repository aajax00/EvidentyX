import { useEffect, useRef } from "react";

/** Échantillons par ligne et nombre de lignes du maillage. */
const COLS = 72;
const ROWS = 28;
/** Images par seconde de l'animation (très lente, discrète). */
const FRAME_MS = 1000 / 24;

/** Reliefs : position (x, z dans [0, 1]), hauteur, étendue. Massif à droite. */
const PEAKS = [
  { x: 0.62, z: 0.3, h: 0.5, sx: 0.07, sz: 0.18 },
  { x: 0.75, z: 0.45, h: 0.85, sx: 0.06, sz: 0.2 },
  { x: 0.86, z: 0.25, h: 0.6, sx: 0.08, sz: 0.16 },
  { x: 0.95, z: 0.55, h: 0.45, sx: 0.07, sz: 0.22 },
  { x: 0.5, z: 0.6, h: 0.22, sx: 0.09, sz: 0.2 },
] as const;

function height(x: number, z: number, t: number): number {
  let h = 0;
  for (const p of PEAKS) {
    const dx = (x - p.x) / p.sx;
    const dz = (z - p.z) / p.sz;
    h += p.h * Math.exp(-0.5 * (dx * dx + dz * dz));
  }
  // Houle lente : le relief « respire » sans se déplacer.
  return h * (1 + 0.06 * Math.sin(t + x * 6 + z * 4)) + 0.025 * Math.sin(x * 14 + t * 0.7) * Math.cos(z * 9);
}

/**
 * Bandeau décoratif : relief en fil de fer bleu-violet, en perspective, qui
 * s'estompe vers la gauche (sous le texte). Purement décoratif (aria-hidden).
 * `animated` : faible ondulation ; sinon une image fixe. En pause quand la
 * fenêtre est masquée.
 */
export function TerrainBanner({ className, animated }: { className?: string; animated: boolean }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;

    let width = 0;
    let h = 0;
    let raf = 0;
    let last = 0;
    let t = 0;

    function layout() {
      if (!canvas || !ctx) return;
      const dpr = window.devicePixelRatio || 1;
      const rect = canvas.getBoundingClientRect();
      width = rect.width;
      h = rect.height;
      canvas.width = Math.max(1, Math.round(width * dpr));
      canvas.height = Math.max(1, Math.round(h * dpr));
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      draw();
    }

    /** Point du maillage projeté à l'écran. */
    function project(x: number, z: number): [number, number] {
      const p = 0.5 + 0.5 * z; // plus proche : plus large et plus bas
      const sx = width * (0.5 + (x - 0.5) * 1.3 * p);
      const horizon = h * 0.5;
      const sy = horizon + z * z * (h * 1.08 - horizon) - height(x, z, t) * h * 0.6 * p;
      return [sx, sy];
    }

    function draw() {
      if (!ctx || width === 0) return;
      ctx.clearRect(0, 0, width, h);
      const stroke = ctx.createLinearGradient(width * 0.3, 0, width, 0);
      stroke.addColorStop(0, "rgb(61 143 240)");
      stroke.addColorStop(0.55, "rgb(110 120 245)");
      stroke.addColorStop(1, "rgb(139 108 246)");
      ctx.strokeStyle = stroke;
      ctx.lineWidth = 1;

      const grid: Array<Array<[number, number]>> = [];
      for (let r = 0; r <= ROWS; r++) {
        const z = r / ROWS;
        const row: Array<[number, number]> = [];
        for (let c = 0; c <= COLS; c++) row.push(project(c / COLS, z));
        grid.push(row);
      }

      // Du fond vers l'avant : chaque ligne masque ce qui est derrière elle
      // (remplissage sombre), puis est tracée avec ses liaisons de profondeur.
      for (let r = 0; r <= ROWS; r++) {
        const row = grid[r]!;
        ctx.globalAlpha = 1;
        ctx.fillStyle = "rgb(10 14 24 / 0.82)";
        ctx.beginPath();
        row.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
        ctx.lineTo(width, h);
        ctx.lineTo(0, h);
        ctx.closePath();
        ctx.fill();

        const near = r / ROWS;
        ctx.globalAlpha = 0.18 + 0.62 * near ** 1.4;
        ctx.beginPath();
        row.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
        if (r > 0) {
          const prev = grid[r - 1]!;
          for (let c = 0; c <= COLS; c += 2) {
            ctx.moveTo(prev[c]![0], prev[c]![1]);
            ctx.lineTo(row[c]![0], row[c]![1]);
          }
        }
        ctx.stroke();
      }
      // Sommets lumineux.
      ctx.globalAlpha = 1;
      ctx.fillStyle = "rgb(160 190 255 / 0.55)";
      for (let r = 2; r <= ROWS; r += 3) {
        for (let c = 0; c <= COLS; c += 3) {
          const hv = height(c / COLS, r / ROWS, t);
          if (hv < 0.35) continue;
          const [x, y] = grid[r]![c]!;
          ctx.fillRect(x - 1, y - 1, 2, 2);
        }
      }

      // Estompage vers la gauche (texte) et vers le haut.
      ctx.globalCompositeOperation = "destination-out";
      const fadeX = ctx.createLinearGradient(0, 0, width, 0);
      fadeX.addColorStop(0, "rgb(0 0 0 / 1)");
      fadeX.addColorStop(0.3, "rgb(0 0 0 / 1)");
      fadeX.addColorStop(0.58, "rgb(0 0 0 / 0)");
      ctx.fillStyle = fadeX;
      ctx.fillRect(0, 0, width, h);
      const fadeY = ctx.createLinearGradient(0, 0, 0, h);
      fadeY.addColorStop(0, "rgb(0 0 0 / 0.8)");
      fadeY.addColorStop(0.18, "rgb(0 0 0 / 0)");
      ctx.fillStyle = fadeY;
      ctx.fillRect(0, 0, width, h);
      ctx.globalCompositeOperation = "source-over";
    }

    function step(now: number) {
      raf = requestAnimationFrame(step);
      if (now - last < FRAME_MS) return;
      last = now;
      t += 0.012;
      draw();
    }

    function start() {
      cancelAnimationFrame(raf);
      if (animated && !document.hidden) raf = requestAnimationFrame(step);
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
  }, [animated]);

  return <canvas ref={canvasRef} className={className} aria-hidden="true" />;
}
