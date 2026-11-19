import { useState } from "react";
import { ToastProvider } from "./components/ui";
import { EngineProvider } from "./engine";
import { SplashScreen } from "./motion/SplashScreen";
import { useReducedMotion } from "./motion/useReducedMotion";
import { InvestigationsProvider } from "./investigations/InvestigationsProvider";
import { AboutPage } from "./pages/AboutPage";
import { HomePage } from "./pages/HomePage";
import { InvestigationPage } from "./pages/InvestigationPage";
import { InvestigationsPage } from "./pages/InvestigationsPage";
import { AppShell } from "./shell/AppShell";
import { RouterProvider, useRouter } from "./shell/router";
import "./pages/pages.css";

export function App() {
  const reducedMotion = useReducedMotion();
  // Écran d'ouverture affiché une fois au lancement (sauf si les animations sont réduites).
  const [splash, setSplash] = useState(() => !window.matchMedia("(prefers-reduced-motion: reduce)").matches);

  return (
    <EngineProvider>
      {splash && !reducedMotion && <SplashScreen onDone={() => setSplash(false)} />}
      <ToastProvider>
        <RouterProvider>
          <InvestigationsProvider>
            <AppShell>
              <CurrentPage />
            </AppShell>
          </InvestigationsProvider>
        </RouterProvider>
      </ToastProvider>
    </EngineProvider>
  );
}

function CurrentPage() {
  const { route } = useRouter();
  switch (route.page) {
    case "home":
      return <HomePage />;
    case "investigations":
      return <InvestigationsPage />;
    case "investigation":
      return <InvestigationPage key={route.id} id={route.id} />;
    case "about":
      return <AboutPage />;
  }
}
