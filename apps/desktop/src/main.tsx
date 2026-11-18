import { StrictMode, Suspense, lazy } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles/tokens.css";
import "./styles/base.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("Élément racine #root introuvable");
}

// Catalogue du système de design : disponible uniquement en développement,
// via l'adresse http://localhost:1420/#catalogue. Exclu de la version publiée.
const DesignCatalog = import.meta.env.DEV
  ? lazy(() => import("./dev/DesignCatalog").then((m) => ({ default: m.DesignCatalog })))
  : null;

const showCatalog = DesignCatalog !== null && window.location.hash === "#catalogue";

createRoot(root).render(
  <StrictMode>
    {showCatalog && DesignCatalog ? (
      <Suspense fallback={null}>
        <DesignCatalog />
      </Suspense>
    ) : (
      <App />
    )}
  </StrictMode>,
);
