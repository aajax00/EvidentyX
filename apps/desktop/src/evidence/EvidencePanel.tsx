import { useCallback, useEffect, useState } from "react";
import {
  Copy,
  FileArchive,
  FileText,
  HardDriveUpload,
  Network,
  RefreshCw,
  ScanSearch,
  ShieldCheck,
  Trash2,
  type LucideIcon,
} from "lucide-react";
import {
  addEvidence,
  analyzeEvidence,
  isTauri,
  listEvidence,
  pickEvidenceFiles,
  removeEvidence,
  toCommandError,
  verifyEvidence,
  type Evidence,
  type IntegrityStatus,
} from "../api";
import { Badge, Button, Dialog, EmptyState, IconButton, Panel, Spinner, useToast } from "../components/ui";
import { formatBytes, formatDateTime, formatRelative, shortDigest } from "../format";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import "./evidence.css";

const INTEGRITY: Record<IntegrityStatus, { label: string; tone: "success" | "danger" | "warning" }> = {
  intact: { label: "Intègre", tone: "success" },
  modified: { label: "Modifiée", tone: "danger" },
  missing: { label: "Introuvable", tone: "warning" },
};

function iconFor(fileName: string): LucideIcon {
  const ext = fileName.toLowerCase().split(".").pop() ?? "";
  if (["pcap", "pcapng", "cap"].includes(ext)) return Network;
  if (["zip", "gz", "tar", "7z", "evtx"].includes(ext)) return FileArchive;
  return FileText;
}

function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/**
 * Preuves d'une enquête : ajout (sélecteur ou glisser-déposer), empreinte
 * SHA-256, vérification d'intégrité, retrait. Les fichiers ne sont jamais
 * copiés ni modifiés ; seule une référence est enregistrée.
 */
export function EvidencePanel({ investigationId, readOnly }: { investigationId: number; readOnly: boolean }) {
  const { notify } = useToast();
  const { refresh } = useInvestigations();
  const [items, setItems] = useState<Evidence[] | null>(null);
  const [pending, setPending] = useState<string[]>([]);
  const [verifying, setVerifying] = useState<Set<number>>(new Set());
  const [analyzing, setAnalyzing] = useState<Set<number>>(new Set());
  const [toRemove, setToRemove] = useState<Evidence | null>(null);
  const [removing, setRemoving] = useState(false);
  const [dragOver, setDragOver] = useState(false);

  const reload = useCallback(() => {
    listEvidence(investigationId)
      .then(setItems)
      .catch((e: unknown) => notify(toCommandError(e).message, "danger"));
  }, [investigationId, notify]);

  useEffect(reload, [reload]);

  const addPaths = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) return;
      setPending((p) => [...p, ...paths]);
      let added = 0;
      for (const path of paths) {
        try {
          await addEvidence(investigationId, path);
          added++;
        } catch (e) {
          notify(`${baseName(path)} : ${toCommandError(e).message}`, "danger");
        } finally {
          setPending((p) => {
            const i = p.indexOf(path);
            return i < 0 ? p : [...p.slice(0, i), ...p.slice(i + 1)];
          });
        }
      }
      if (added > 0) {
        notify(added === 1 ? "Preuve ajoutée, empreinte SHA-256 calculée." : `${added} preuves ajoutées.`);
        refresh();
      }
      reload();
    },
    [investigationId, notify, refresh, reload],
  );

  // Glisser-déposer de fichiers depuis le système (application de bureau).
  useEffect(() => {
    if (!isTauri() || readOnly) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void import("@tauri-apps/api/webview").then(({ getCurrentWebview }) =>
      getCurrentWebview()
        .onDragDropEvent((event) => {
          const p = event.payload;
          if (p.type === "enter" || p.type === "over") setDragOver(true);
          else if (p.type === "leave") setDragOver(false);
          else if (p.type === "drop") {
            setDragOver(false);
            void addPaths(p.paths);
          }
        })
        .then((fn) => {
          if (cancelled) fn();
          else unlisten = fn;
        }),
    );
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [addPaths, readOnly]);

  async function pick() {
    try {
      await addPaths(await pickEvidenceFiles());
    } catch (e) {
      notify(toCommandError(e).message, "danger");
    }
  }

  async function verify(ids: number[]) {
    setVerifying((v) => new Set([...v, ...ids]));
    const results: Evidence[] = [];
    for (const id of ids) {
      try {
        results.push(await verifyEvidence(id));
      } catch (e) {
        notify(toCommandError(e).message, "danger");
      } finally {
        setVerifying((v) => {
          const next = new Set(v);
          next.delete(id);
          return next;
        });
      }
    }
    const altered = results.filter((r) => r.integrity !== "intact");
    if (results.length > 0) {
      notify(
        altered.length === 0
          ? results.length === 1
            ? "Preuve intègre : empreinte identique."
            : `${results.length} preuves intègres.`
          : `${altered.length} preuve${altered.length > 1 ? "s" : ""} altérée${altered.length > 1 ? "s" : ""} ou introuvable${altered.length > 1 ? "s" : ""}.`,
        altered.length === 0 ? "success" : "danger",
      );
    }
    reload();
  }

  async function analyze(e: Evidence) {
    setAnalyzing((s) => new Set([...s, e.id]));
    try {
      const r = await analyzeEvidence(e.id);
      const n = (v: number) => v.toLocaleString("fr-FR");
      const parts = [`${r.format} · ${n(r.inserted)} événement${r.inserted > 1 ? "s" : ""} extrait${r.inserted > 1 ? "s" : ""}`];
      if (r.timeField) parts.push(`heure : ${r.timeField}`);
      else if (r.format === "CSV") parts.push("aucune colonne d'heure reconnue");
      if (r.duplicates > 0) parts.push(`${n(r.duplicates)} déjà présent${r.duplicates > 1 ? "s" : ""}`);
      if (r.errors > 0) {
        const first = r.firstErrors[0];
        parts.push(`${n(r.errors)} enregistrement${r.errors > 1 ? "s" : ""} illisible${r.errors > 1 ? "s" : ""}${first ? ` (ligne ${n(first.line)} : ${first.message})` : ""}`);
      }
      notify(`« ${e.fileName} » : ${parts.join(" · ")}.`, r.errors > 0 && r.inserted === 0 ? "danger" : "success");
      refresh();
    } catch (err) {
      notify(`« ${e.fileName} » : ${toCommandError(err).message}`, "danger");
    } finally {
      setAnalyzing((s) => {
        const next = new Set(s);
        next.delete(e.id);
        return next;
      });
      reload();
    }
  }

  async function copy(sha256: string) {
    try {
      await navigator.clipboard.writeText(sha256);
      notify("Empreinte SHA-256 copiée.");
    } catch {
      notify("Copie impossible : sélectionnez l'empreinte à la main.", "danger");
    }
  }

  const count = items?.length ?? 0;

  return (
    <Panel
      title={`Preuves${count ? ` (${count})` : ""}`}
      actions={
        <>
          {count > 0 && (
            <Button
              size="sm"
              variant="ghost"
              icon={ShieldCheck}
              loading={verifying.size > 0}
              onClick={() => void verify(items?.map((i) => i.id) ?? [])}
            >
              Tout vérifier
            </Button>
          )}
          {!readOnly && (
            <Button size="sm" variant="primary" icon={HardDriveUpload} onClick={() => void pick()}>
              Ajouter des preuves
            </Button>
          )}
        </>
      }
    >
      <div className={`evidence-zone${dragOver ? " evidence-zone--over" : ""}`}>
        {dragOver && <p className="evidence-zone__hint">Déposez les fichiers pour les ajouter à l'enquête</p>}

        {items === null ? (
          <p className="page-inline">
            <Spinner size="sm" /> Chargement…
          </p>
        ) : count === 0 && pending.length === 0 ? (
          <EmptyState
            icon={HardDriveUpload}
            title="Aucune preuve"
            action={
              !readOnly && (
                <Button variant="primary" icon={HardDriveUpload} onClick={() => void pick()}>
                  Ajouter des preuves
                </Button>
              )
            }
          >
            {readOnly
              ? "Enquête archivée : réactivez-la pour ajouter des preuves."
              : "Ajoutez des journaux ou des captures réseau, ou glissez-déposez-les ici. Les fichiers restent à leur place : EVIDENTYX ne les copie ni ne les modifie, il en calcule l'empreinte SHA-256."}
          </EmptyState>
        ) : (
          <ul className="evidence-list">
            {items?.map((e) => {
              const Icon = iconFor(e.fileName);
              const integrity = INTEGRITY[e.integrity];
              return (
                <li key={e.id} className="evidence-item">
                  <span className={`evidence-item__icon evidence-item__icon--${e.integrity}`} aria-hidden="true">
                    <Icon size={16} />
                  </span>
                  <div className="evidence-item__main">
                    <p className="evidence-item__name" title={e.path}>
                      {e.fileName}
                    </p>
                    <p className="evidence-item__path mono" title={e.path}>
                      {e.path}
                    </p>
                    <p className="evidence-item__events">
                      {e.eventCount === 0
                        ? "Non analysée"
                        : `${e.eventCount.toLocaleString("fr-FR")} événement${e.eventCount > 1 ? "s" : ""} extrait${e.eventCount > 1 ? "s" : ""}`}
                    </p>
                  </div>
                  <span className="evidence-item__size mono">{formatBytes(e.sizeBytes)}</span>
                  <span className="evidence-item__digest">
                    <code title={`SHA-256 : ${e.sha256}`}>{shortDigest(e.sha256)}</code>
                    <IconButton icon={Copy} label="Copier l'empreinte SHA-256" onClick={() => void copy(e.sha256)} />
                  </span>
                  <span className="evidence-item__integrity" title={`Vérifiée le ${formatDateTime(e.verifiedAt)}`}>
                    <Badge tone={integrity.tone}>{integrity.label}</Badge>
                    <span className="evidence-item__checked">{formatRelative(e.verifiedAt)}</span>
                  </span>
                  <span className="evidence-item__actions">
                    {!readOnly &&
                      (analyzing.has(e.id) ? (
                        <Spinner size="sm" label="Analyse en cours" />
                      ) : (
                        <IconButton
                          icon={ScanSearch}
                          label={e.eventCount > 0 ? "Réanalyser (ajoute seulement ce qui manque)" : "Analyser : extraire les événements"}
                          onClick={() => void analyze(e)}
                        />
                      ))}
                    {verifying.has(e.id) ? (
                      <Spinner size="sm" label="Vérification en cours" />
                    ) : (
                      <IconButton icon={RefreshCw} label="Vérifier l'intégrité" onClick={() => void verify([e.id])} />
                    )}
                    {!readOnly && (
                      <IconButton icon={Trash2} label="Retirer de l'enquête" variant="danger" onClick={() => setToRemove(e)} />
                    )}
                  </span>
                </li>
              );
            })}
            {pending.map((path, i) => (
              <li key={`pending-${i}-${path}`} className="evidence-item evidence-item--pending">
                <span className="evidence-item__icon" aria-hidden="true">
                  <Spinner size="sm" />
                </span>
                <div className="evidence-item__main">
                  <p className="evidence-item__name">{baseName(path)}</p>
                  <p className="evidence-item__path mono">Calcul de l'empreinte SHA-256…</p>
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      {toRemove && (
        <Dialog
          width="sm"
          title="Retirer la preuve ?"
          onClose={() => {
            if (!removing) setToRemove(null);
          }}
          footer={
            <>
              <Button onClick={() => setToRemove(null)} disabled={removing} autoFocus>
                Annuler
              </Button>
              <Button
                variant="danger"
                icon={Trash2}
                loading={removing}
                onClick={async () => {
                  const target = toRemove;
                  setRemoving(true);
                  try {
                    await removeEvidence(target.id);
                    notify(`« ${target.fileName} » retirée de l'enquête.`);
                    refresh();
                  } catch (e) {
                    notify(toCommandError(e).message, "danger");
                  } finally {
                    setRemoving(false);
                    setToRemove(null);
                  }
                  reload();
                }}
              >
                Retirer
              </Button>
            </>
          }
        >
          <div className="form-stack">
            <p>
              <strong>« {toRemove.fileName} »</strong> ne fera plus partie de l'enquête.
            </p>
            <p className="text-muted">
              Le fichier lui-même n'est ni supprimé ni modifié : il reste à son emplacement (
              <span className="mono">{toRemove.path}</span>).
            </p>
          </div>
        </Dialog>
      )}
    </Panel>
  );
}
