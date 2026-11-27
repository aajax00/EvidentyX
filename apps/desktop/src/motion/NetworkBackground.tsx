import { useEffect, useRef } from "react";

/**
 * Arrière-plan « forensique », copie exacte de la maquette EvidentyX : canevas
 * fixe sur toute la fenêtre, grille orthogonale de 56 px très discrète et 12 à
 * 26 points carrés qui dérivent et se relient en équerre à moins de 170 px.
 * Purement décoratif (aria-hidden), en pause quand la fenêtre est masquée. Le
 * parent décide de l'afficher (réglage utilisateur, réduction des animations).
 */
export function NetworkBackground({ className }: { className?: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    const ctx = canvas?.getContext("2d");
    if (!canvas || !ctx) return;
    let w = 0;
    let h = 0;
    let nodes: Array<{ x: number; y: number; vx: number; vy: number }> = [];
    let last = 0;
    let raf = 0;

    function resize() {
      if (!canvas) return;
      w = canvas.width = window.innerWidth;
      h = canvas.height = window.innerHeight;
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
      if (document.hidden || !ctx) return;
      if (now - last < 33) return;
      last = now;

      ctx.clearRect(0, 0, w, h);

      // Grille orthogonale très discrète (56px)
      ctx.strokeStyle = "rgba(34, 43, 58, 0.22)";
      ctx.lineWidth = 1;
      const step = 56;
      ctx.beginPath();
      for (let x = 0; x < w; x += step) {
        ctx.moveTo(x, 0);
        ctx.lineTo(x, h);
      }
      for (let y = 0; y < h; y += step) {
        ctx.moveTo(0, y);
        ctx.lineTo(w, y);
      }
      ctx.stroke();

      // Nœuds et liaisons orthogonales fines
      for (let i = 0; i < nodes.length; i++) {
        const n = nodes[i]!;
        n.x += n.vx;
        n.y += n.vy;
        if (n.x < 10 || n.x > w - 10) n.vx *= -1;
        if (n.y < 10 || n.y > h - 10) n.vy *= -1;

        for (let j = i + 1; j < nodes.length; j++) {
          const m = nodes[j]!;
          const dist = Math.hypot(n.x - m.x, n.y - m.y);
          if (dist < 170) {
            const alpha = (1 - dist / 170) * 0.11;
            ctx.strokeStyle = `rgba(61, 143, 240, ${alpha.toFixed(3)})`;
            ctx.beginPath();
            ctx.moveTo(n.x, n.y);
            ctx.lineTo(m.x, n.y);
            ctx.lineTo(m.x, m.y);
            ctx.stroke();
          }
        }
        ctx.fillStyle = "rgba(49, 60, 79, 0.55)";
        ctx.fillRect(n.x - 1.5, n.y - 1.5, 3, 3);
      }
    }

    window.addEventListener("resize", resize);
    resize();
    raf = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(raf);
      window.removeEventListener("resize", resize);
    };
  }, []);

  return <canvas ref={canvasRef} className={className} aria-hidden="true" />;
}
