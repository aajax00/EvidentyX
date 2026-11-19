import { useState } from "react";
import { Trash2 } from "lucide-react";
import { toCommandError, type Investigation } from "../api";
import { Button, Dialog } from "../components/ui";

export interface DeleteDialogProps {
  investigation: Investigation;
  onConfirm: () => Promise<void>;
  onClose: () => void;
}

/** Confirmation explicite avant une suppression définitive. */
export function DeleteDialog({ investigation, onConfirm, onClose }: DeleteDialogProps) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function confirm() {
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
    } catch (e) {
      setError(toCommandError(e).message);
      setBusy(false);
    }
  }

  return (
    <Dialog
      width="sm"
      title="Supprimer l'enquête ?"
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose} autoFocus>
            Annuler
          </Button>
          <Button variant="danger" icon={Trash2} loading={busy} onClick={confirm}>
            Supprimer définitivement
          </Button>
        </>
      }
    >
      <div className="form-stack">
        <p>
          L'enquête <strong>« {investigation.name} »</strong> sera supprimée de la base locale. Cette action est
          irréversible.
        </p>
        {investigation.status === "active" && (
          <p className="text-muted">Pour la conserver en lecture seule, archivez-la plutôt.</p>
        )}
        {error && (
          <p className="form-error" role="alert">
            {error}
          </p>
        )}
      </div>
    </Dialog>
  );
}
