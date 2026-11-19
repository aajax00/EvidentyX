import { useId, useState, type FormEvent } from "react";
import { toCommandError, type CommandError, type Investigation } from "../api";
import { Button, Dialog, TextAreaField, TextField } from "../components/ui";

export type FormMode = { kind: "create" } | { kind: "rename"; investigation: Investigation };

export interface InvestigationFormDialogProps {
  mode: FormMode;
  onSubmit: (values: { name: string; description: string }) => Promise<void>;
  onClose: () => void;
}

/**
 * Création ou renommage d'une enquête. La validation fait autorité côté moteur :
 * les erreurs renvoyées sont affichées sous le champ concerné.
 */
export function InvestigationFormDialog({ mode, onSubmit, onClose }: InvestigationFormDialogProps) {
  const creating = mode.kind === "create";
  const [name, setName] = useState(creating ? "" : mode.investigation.name);
  const [description, setDescription] = useState("");
  const [error, setError] = useState<CommandError | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const formId = useId();

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSubmitting(true);
    setError(null);
    try {
      await onSubmit({ name, description });
    } catch (e) {
      setError(toCommandError(e));
      setSubmitting(false);
    }
  }

  const fieldError = (field: CommandError["field"]) => (error && error.field === field ? error.message : undefined);
  const generalError = error && !error.field ? error.message : null;

  return (
    <Dialog
      title={creating ? "Nouvelle enquête" : "Renommer l'enquête"}
      description={
        creating ? "Une enquête regroupe les preuves et l'analyse d'un incident." : `Nom actuel : ${mode.investigation.name}`
      }
      onClose={onClose}
      footer={
        <>
          <Button onClick={onClose}>Annuler</Button>
          <Button type="submit" form={formId} variant="primary" loading={submitting}>
            {creating ? "Créer l'enquête" : "Renommer"}
          </Button>
        </>
      }
    >
      <form id={formId} className="form-stack" onSubmit={submit} noValidate>
        <TextField
          label="Nom"
          required
          autoFocus
          maxLength={200}
          placeholder="Ex. Intrusion serveur web"
          value={name}
          onChange={(e) => setName(e.target.value)}
          hint="120 caractères au maximum."
          error={fieldError("name")}
        />
        {creating && (
          <TextAreaField
            label="Description"
            placeholder="Contexte, périmètre, systèmes concernés…"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            hint="Facultative. Modifiable ensuite."
            error={fieldError("description")}
          />
        )}
        {generalError && (
          <p className="form-error" role="alert">
            {generalError}
          </p>
        )}
      </form>
    </Dialog>
  );
}
