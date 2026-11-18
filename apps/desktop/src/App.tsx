import { useState } from "react";
import { EngineProvider } from "./engine";
import { AboutPage } from "./pages/AboutPage";
import { HomePage } from "./pages/HomePage";
import { AppShell } from "./shell/AppShell";
import type { PageId } from "./shell/navigation";
import "./pages/pages.css";

export function App() {
  const [page, setPage] = useState<PageId>("home");

  return (
    <EngineProvider>
      <AppShell page={page} onNavigate={setPage}>
        {page === "home" && <HomePage />}
        {page === "about" && <AboutPage />}
      </AppShell>
    </EngineProvider>
  );
}
