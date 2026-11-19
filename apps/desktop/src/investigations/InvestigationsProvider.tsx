// État partagé des enquêtes et actions associées (création, renommage,
// archivage, suppression), avec leurs boîtes de dialogue et notifications.

import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import {
  createInvestigation,
  deleteInvestigation,
  listInvestigations,
  renameInvestigation,
  setInvestigationStatus,
  toCommandError,
  type CommandError,
  type Investigation,
} from "../api";
import { useToast } from "../components/ui";
import { useRouter } from "../shell/router";
import { DeleteDialog } from "./DeleteDialog";
import { InvestigationFormDialog, type FormMode } from "./InvestigationFormDialog";

interface InvestigationsApi {
  /** Toutes les enquêtes, de la plus récemment modifiée à la plus ancienne. */
  all: Investigation[];
  loading: boolean;
  error: CommandError | null;
  /** Incrémenté après chaque modification : les vues filtrées s'y abonnent pour se recharger. */
  revision: number;
  refresh: () => void;
  openCreate: () => void;
  openRename: (investigation: Investigation) => void;
  openDelete: (investigation: Investigation) => void;
  toggleArchive: (investigation: Investigation) => Promise<void>;
}

const InvestigationsContext = createContext<InvestigationsApi | null>(null);

type DialogState = { kind: "form"; mode: FormMode } | { kind: "delete"; investigation: Investigation } | null;

export function InvestigationsProvider({ children }: { children: ReactNode }) {
  const { route, navigate } = useRouter();
  const { notify } = useToast();
  const [all, setAll] = useState<Investigation[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<CommandError | null>(null);
  const [revision, setRevision] = useState(0);
  const [dialog, setDialog] = useState<DialogState>(null);

  const refresh = useCallback(() => setRevision((r) => r + 1), []);

  useEffect(() => {
    let cancelled = false;
    listInvestigations()
      .then((list) => {
        if (cancelled) return;
        setAll(list);
        setError(null);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(toCommandError(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [revision]);

  const closeDialog = useCallback(() => setDialog(null), []);

  const toggleArchive = useCallback(
    async (investigation: Investigation) => {
      const target = investigation.status === "active" ? "archived" : "active";
      try {
        await setInvestigationStatus(investigation.id, target);
        notify(target === "archived" ? `« ${investigation.name} » archivée.` : `« ${investigation.name} » réactivée.`);
      } catch (e) {
        notify(toCommandError(e).message, "danger");
      }
      refresh();
    },
    [notify, refresh],
  );

  const api = useMemo<InvestigationsApi>(
    () => ({
      all,
      loading,
      error,
      revision,
      refresh,
      openCreate: () => setDialog({ kind: "form", mode: { kind: "create" } }),
      openRename: (investigation) => setDialog({ kind: "form", mode: { kind: "rename", investigation } }),
      openDelete: (investigation) => setDialog({ kind: "delete", investigation }),
      toggleArchive,
    }),
    [all, loading, error, revision, refresh, toggleArchive],
  );

  return (
    <InvestigationsContext.Provider value={api}>
      {children}

      {dialog?.kind === "form" && (
        <InvestigationFormDialog
          mode={dialog.mode}
          onClose={closeDialog}
          onSubmit={async ({ name, description }) => {
            if (dialog.mode.kind === "create") {
              const created = await createInvestigation(name, description);
              notify(`Enquête « ${created.name} » créée.`);
              navigate({ page: "investigation", id: created.id });
            } else {
              const renamed = await renameInvestigation(dialog.mode.investigation.id, name);
              notify(`Enquête renommée en « ${renamed.name} ».`);
            }
            setDialog(null);
            refresh();
          }}
        />
      )}

      {dialog?.kind === "delete" && (
        <DeleteDialog
          investigation={dialog.investigation}
          onClose={closeDialog}
          onConfirm={async () => {
            const { id, name } = dialog.investigation;
            await deleteInvestigation(id);
            notify(`Enquête « ${name} » supprimée.`);
            setDialog(null);
            if (route.page === "investigation" && route.id === id) navigate({ page: "investigations" });
            refresh();
          }}
        />
      )}
    </InvestigationsContext.Provider>
  );
}

export function useInvestigations(): InvestigationsApi {
  const api = useContext(InvestigationsContext);
  if (!api) throw new Error("useInvestigations doit être utilisé sous <InvestigationsProvider>");
  return api;
}
