// Navigation entre les écrans (application de bureau : pas d'URL, un état typé).

import { createContext, useContext, useMemo, useState, type ReactNode } from "react";

export type Route =
  | { page: "home" }
  | { page: "investigations" }
  | { page: "investigation"; id: number }
  | { page: "timeline"; id: number }
  | { page: "settings" }
  | { page: "about" };

export type PageId = Route["page"];

interface RouterApi {
  route: Route;
  navigate: (route: Route) => void;
}

const RouterContext = createContext<RouterApi | null>(null);

export function RouterProvider({ children, initial = { page: "home" } }: { children: ReactNode; initial?: Route }) {
  const [route, navigate] = useState<Route>(initial);
  const api = useMemo(() => ({ route, navigate }), [route]);
  return <RouterContext.Provider value={api}>{children}</RouterContext.Provider>;
}

export function useRouter(): RouterApi {
  const api = useContext(RouterContext);
  if (!api) throw new Error("useRouter doit être utilisé sous <RouterProvider>");
  return api;
}

/** Page de la barre latérale à surligner pour une route (une enquête et sa chronologie relèvent de « Enquêtes »). */
export function sectionOf(route: Route): Exclude<PageId, "investigation" | "timeline"> {
  return route.page === "investigation" || route.page === "timeline" ? "investigations" : route.page;
}
