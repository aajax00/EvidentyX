import { useEffect, useState, type ReactNode } from "react";
import { Spinner, ToastProvider } from "./components/ui";
import { EngineProvider } from "./engine";
import { SplashScreen } from "./motion/SplashScreen";
import { useReducedMotion } from "./motion/useReducedMotion";
import { InvestigationsProvider } from "./investigations/InvestigationsProvider";
import { AboutPage } from "./pages/AboutPage";
import { GuidePage } from "./pages/GuidePage";
import { HomePage } from "./pages/HomePage";
import { InvestigationPage } from "./pages/InvestigationPage";
import { InvestigationsPage } from "./pages/InvestigationsPage";
import { SettingsPage } from "./pages/SettingsPage";
import { SettingsProvider, useSettings } from "./settings/SettingsProvider";
import { TimelinePage } from "./timeline/TimelinePage";
import { AppShell } from "./shell/AppShell";
import { RouterProvider, useRouter } from "./shell/router";
import "./pages/pages.css";

export function App() {
  return (
    <EngineProvider>
      <SettingsProvider>
        <ToastProvider>
          <RouterProvider>
            <InvestigationsProvider>
              <LaunchGate>
                <AppShell>
                  <CurrentPage />
                </AppShell>
              </LaunchGate>
            </InvestigationsProvider>
          </RouterProvider>
        </ToastProvider>
      </SettingsProvider>
    </EngineProvider>
  );
}

/**
 * Attend la lecture des réglages (quelques millisecondes) avant d'afficher
 * l'interface, pour appliquer d'emblée la barre latérale et l'écran d'ouverture.
 */
function LaunchGate({ children }: { children: ReactNode }) {
  const { settings, loaded } = useSettings();
  const reducedMotion = useReducedMotion();
  const [splashDone, setSplashDone] = useState(false);

  if (!loaded) return <DatabaseWait />;
  const showSplash = settings.splashScreen && !reducedMotion && !splashDone;
  return (
    <>
      {children}
      {showSplash && <SplashScreen onDone={() => setSplashDone(true)} />}
    </>
  );
}

/**
 * Attente de la base : rien pendant 600 ms (cas courant), puis un message. La
 * mise à jour d'une grosse base peut durer une minute ou plus, une seule fois.
 */
function DatabaseWait() {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const t = window.setTimeout(() => setVisible(true), 600);
    return () => window.clearTimeout(t);
  }, []);
  if (!visible) return null;
  return (
    <div className="database-wait" role="status">
      <Spinner label="Ouverture de la base" />
      <p>Ouverture de la base locale…</p>
      <p className="text-muted">
        Après une mise à jour, une base volumineuse peut demander une minute ou plus, une seule fois. Ne fermez pas
        l'application.
      </p>
    </div>
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
    case "timeline":
      return <TimelinePage key={route.id} id={route.id} />;
    case "settings":
      return <SettingsPage />;
    case "guide":
      return <GuidePage anchor={route.anchor} />;
    case "about":
      return <AboutPage />;
  }
}
