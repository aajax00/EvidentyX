import { useEffect, useState } from "react";
import {
  Activity,
  Archive,
  ArchiveRestore,
  ArrowLeft,
  FileText,
  FolderX,
  HardDriveUpload,
  Network,
  Pencil,
  ShieldAlert,
  Trash2,
  type LucideIcon,
} from "lucide-react";
import {
  getInvestigation,
  toCommandError,
  updateInvestigationDescription,
  type CommandError,
  type Investigation,
} from "../api";
import { Badge, Button, EmptyState, Panel, Spinner, TextAreaField, useToast } from "../components/ui";
import { formatDateTime, formatRelative } from "../format";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { StatusBadge } from "../investigations/StatusBadge";
import { EvidencePanel } from "../evidence/EvidencePanel";
import { OverviewPanel } from "../overview/OverviewPanel";
import { useRouter } from "../shell/router";

const MODULES: ReadonlyArray<{ icon: LucideIcon; name: string; text: string; plannedFor: string }> = [
  { icon: HardDriveUpload, name: "Analyse des preuves", text: "Lecture des journaux et captures réseau, événements extraits.", plannedFor: "phase 2" },
  { icon: Activity, name: "Chronologie", text: "Événements normalisés, triés et filtrables.", plannedFor: "phase 3" },
  { icon: Network, name: "Graphe", text: "Entités et relations reconstituées.", plannedFor: "phase 3" },
  { icon: ShieldAlert, name: "Détections", text: "Règles versionnées et explicables.", plannedFor: "phase 4" },
  { icon: FileText, name: "Rapports", text: "Exports PDF, JSON et CSV.", plannedFor: "phase 4" },
];

export function InvestigationPage({ id }: { id: number }) {
  const { revision, openRename, openDelete, toggleArchive, refresh } = useInvestigations();
  const { navigate } = useRouter();
  const [investigation, setInvestigation] = useState<Investigation | null>(null);
  const [error, setError] = useState<CommandError | null>(null);

  useEffect(() => {
    let cancelled = false;
    getInvestigation(id)
      .then((inv) => {
        if (cancelled) return;
        setInvestigation(inv);
        setError(null);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(toCommandError(e));
      });
    return () => {
      cancelled = true;
    };
  }, [id, revision]);

  if (error) {
    return (
      <div className="page">
        <Panel>
          <EmptyState
            icon={FolderX}
            title={error.code === "not_found" ? "Enquête introuvable" : "Impossible d'ouvrir l'enquête"}
            action={
              <Button icon={ArrowLeft} onClick={() => navigate({ page: "investigations" })}>
                Retour aux enquêtes
              </Button>
            }
          >
            {error.message}
          </EmptyState>
        </Panel>
      </div>
    );
  }

  if (!investigation || investigation.id !== id) {
    return (
      <div className="page">
        <p className="page-inline">
          <Spinner size="sm" /> Ouverture de l'enquête…
        </p>
      </div>
    );
  }

  const archived = investigation.status === "archived";

  return (
    <div className="page">
      <section className="case-header" aria-labelledby="case-title">
        <div className="case-header__main">
          <div className="case-header__title-row">
            <h2 id="case-title" className="case-header__title">
              {investigation.name}
            </h2>
            <StatusBadge status={investigation.status} />
          </div>
          <dl className="case-header__meta">
            <div>
              <dt>Identifiant</dt>
              <dd className="mono">#{investigation.id}</dd>
            </div>
            <div>
              <dt>Créée</dt>
              <dd className="mono">{formatDateTime(investigation.createdAt)}</dd>
            </div>
            <div>
              <dt>Modifiée</dt>
              <dd title={formatDateTime(investigation.updatedAt)}>{formatRelative(investigation.updatedAt)}</dd>
            </div>
          </dl>
        </div>
        <div className="case-header__actions">
          <Button icon={Pencil} disabled={archived} onClick={() => openRename(investigation)}>
            Renommer
          </Button>
          <Button icon={archived ? ArchiveRestore : Archive} onClick={() => void toggleArchive(investigation)}>
            {archived ? "Réactiver" : "Archiver"}
          </Button>
          <Button variant="danger" icon={Trash2} onClick={() => openDelete(investigation)}>
            Supprimer
          </Button>
        </div>
      </section>

      {archived && (
        <p className="notice" role="note">
          <Archive size={14} aria-hidden="true" />
          Enquête archivée : elle est en lecture seule. Réactivez-la pour la modifier.
        </p>
      )}

      <DescriptionPanel
        investigation={investigation}
        readOnly={archived}
        onSaved={(updated) => {
          setInvestigation(updated);
          refresh();
        }}
      />

      <EvidencePanel investigationId={investigation.id} readOnly={archived} />

      <OverviewPanel investigationId={investigation.id} />

      <Panel title="Contenu de l'enquête" actions={<Badge>planifié</Badge>}>
        <p className="text-muted panel-intro">
          Les modules d'analyse arriveront au fil des prochaines phases. Ils apparaîtront ici, rattachés à cette
          enquête.
        </p>
        <ul className="module-grid">
          {MODULES.map(({ icon: Icon, name, text, plannedFor }) => (
            <li key={name} className="module-card" aria-disabled="true">
              <Icon size={16} aria-hidden="true" className="module-card__icon" />
              <div>
                <p className="module-card__name">
                  {name} <span className="module-card__planned">{plannedFor}</span>
                </p>
                <p className="module-card__text">{text}</p>
              </div>
            </li>
          ))}
        </ul>
      </Panel>
    </div>
  );
}

function DescriptionPanel({
  investigation,
  readOnly,
  onSaved,
}: {
  investigation: Investigation;
  readOnly: boolean;
  onSaved: (updated: Investigation) => void;
}) {
  const { notify } = useToast();
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(investigation.description);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function start() {
    setDraft(investigation.description);
    setError(null);
    setEditing(true);
  }

  async function save() {
    setSaving(true);
    setError(null);
    try {
      const updated = await updateInvestigationDescription(investigation.id, draft);
      onSaved(updated);
      notify("Description enregistrée.");
      setEditing(false);
    } catch (e) {
      setError(toCommandError(e).message);
    } finally {
      setSaving(false);
    }
  }

  return (
    <Panel
      title="Description"
      actions={
        !editing &&
        !readOnly && (
          <Button size="sm" variant="ghost" icon={Pencil} onClick={start}>
            Modifier
          </Button>
        )
      }
    >
      {editing ? (
        <div className="form-stack">
          <TextAreaField
            label="Description"
            rows={6}
            autoFocus
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Escape") {
                e.preventDefault();
                setEditing(false);
              }
            }}
            hint="4 000 caractères au maximum."
            error={error ?? undefined}
          />
          <div className="form-actions">
            <Button onClick={() => setEditing(false)}>Annuler</Button>
            <Button variant="primary" loading={saving} onClick={() => void save()}>
              Enregistrer
            </Button>
          </div>
        </div>
      ) : investigation.description ? (
        <p className="description-text">{investigation.description}</p>
      ) : (
        <p className="text-muted">Aucune description.{!readOnly && " Ajoutez le contexte, le périmètre et les systèmes concernés."}</p>
      )}
    </Panel>
  );
}
