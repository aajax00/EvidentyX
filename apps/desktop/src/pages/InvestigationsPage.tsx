import { useEffect, useRef, useState } from "react";
import { Archive, ArchiveRestore, FileSearch, FolderSearch, Pencil, Plus, SearchX, Trash2 } from "lucide-react";
import {
  listInvestigations,
  toCommandError,
  type CommandError,
  type Investigation,
  type InvestigationSort,
  type InvestigationStatus,
} from "../api";
import { Button, EmptyState, IconButton, SearchField, SegmentedControl, Select, Spinner } from "../components/ui";
import { formatDateTime, formatRelative } from "../format";
import { useInvestigations } from "../investigations/InvestigationsProvider";
import { StatusBadge } from "../investigations/StatusBadge";
import { useRouter } from "../shell/router";

type StatusFilter = InvestigationStatus | "all";

const SORT_OPTIONS: ReadonlyArray<{ value: InvestigationSort; label: string }> = [
  { value: "updated_desc", label: "Dernière modification" },
  { value: "created_desc", label: "Date de création" },
  { value: "name_asc", label: "Nom (A → Z)" },
];

export function InvestigationsPage() {
  const { all, revision, openCreate, openRename, openDelete, toggleArchive } = useInvestigations();
  const { navigate } = useRouter();
  const [status, setStatus] = useState<StatusFilter>("active");
  const [text, setText] = useState("");
  const [debouncedText, setDebouncedText] = useState("");
  const [sort, setSort] = useState<InvestigationSort>("updated_desc");
  const [items, setItems] = useState<Investigation[] | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  // Recherche lancée 150 ms après la dernière frappe.
  useEffect(() => {
    const t = window.setTimeout(() => setDebouncedText(text), 150);
    return () => window.clearTimeout(t);
  }, [text]);

  useEffect(() => {
    let cancelled = false;
    listInvestigations({ ...(status === "all" ? {} : { status }), text: debouncedText, sort })
      .then((list) => {
        if (cancelled) return;
        setItems(list);
        setError(null);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(toCommandError(e));
      });
    return () => {
      cancelled = true;
    };
  }, [status, debouncedText, sort, revision]);

  // « / » place le curseur dans la recherche (hors champ de saisie).
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      const target = e.target as HTMLElement;
      if (e.key === "/" && !["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName)) {
        e.preventDefault();
        searchRef.current?.focus();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const counts = {
    active: all.filter((i) => i.status === "active").length,
    archived: all.filter((i) => i.status === "archived").length,
    all: all.length,
  };
  const filtering = debouncedText.trim() !== "";

  return (
    <div className="page page--wide">
      <div className="toolbar">
        <div className="toolbar__search">
          <SearchField
            ref={searchRef}
            label="Rechercher une enquête"
            placeholder="Rechercher par nom ou description…"
            title="Raccourci : /"
            value={text}
            onChange={setText}
          />
        </div>
        <SegmentedControl<StatusFilter>
          label="Statut"
          value={status}
          onChange={setStatus}
          options={[
            { value: "active", label: "Actives", count: counts.active },
            { value: "archived", label: "Archivées", count: counts.archived },
            { value: "all", label: "Toutes", count: counts.all },
          ]}
        />
        <Select label="Trier par" value={sort} options={SORT_OPTIONS} onChange={setSort} />
        <div className="toolbar__spacer" />
        <Button variant="primary" icon={Plus} onClick={openCreate}>
          Nouvelle enquête
        </Button>
      </div>

      <section className="table-frame" aria-label="Liste des enquêtes">
        {error ? (
          <EmptyState icon={SearchX} title="Impossible de lire les enquêtes">
            {error.message}
          </EmptyState>
        ) : items === null ? (
          <p className="page-inline table-frame__loading">
            <Spinner size="sm" /> Chargement…
          </p>
        ) : items.length === 0 ? (
          filtering || (status !== "all" && counts.all > 0) ? (
            <EmptyState
              icon={SearchX}
              title="Aucune enquête ne correspond"
              action={
                <Button
                  onClick={() => {
                    setText("");
                    setStatus("all");
                  }}
                >
                  Réinitialiser les filtres
                </Button>
              }
            >
              {filtering ? `Aucun résultat pour « ${debouncedText.trim()} ».` : "Aucune enquête avec ce statut."}
            </EmptyState>
          ) : (
            <EmptyState
              icon={FolderSearch}
              title="Aucune enquête pour l'instant"
              action={
                <Button variant="primary" icon={Plus} onClick={openCreate}>
                  Créer une enquête
                </Button>
              }
            >
              Une enquête regroupe les preuves d'un incident et leur analyse. Les données restent sur ce poste.
            </EmptyState>
          )
        ) : (
          <>
            <table className="data-table">
              <thead>
                <tr>
                  <th scope="col">Nom</th>
                  <th scope="col" className="data-table__status">
                    Statut
                  </th>
                  <th scope="col" className="data-table__date">
                    Modifiée
                  </th>
                  <th scope="col" className="data-table__date">
                    Créée
                  </th>
                  <th scope="col" className="data-table__actions">
                    <span className="sr-only">Actions</span>
                  </th>
                </tr>
              </thead>
              <tbody>
                {items.map((inv) => (
                  <tr key={inv.id}>
                    <td>
                      <div className="data-table__name-cell">
                        <span className={`case-icon case-icon--${inv.status}`} aria-hidden="true">
                          <FileSearch size={15} />
                        </span>
                        <div className="data-table__name-text">
                          <button
                            type="button"
                            className="data-table__link"
                            onClick={() => navigate({ page: "investigation", id: inv.id })}
                          >
                            {inv.name}
                          </button>
                          {inv.description && <p className="data-table__sub">{inv.description}</p>}
                        </div>
                      </div>
                    </td>
                    <td>
                      <StatusBadge status={inv.status} />
                    </td>
                    <td className="data-table__muted" title={formatDateTime(inv.updatedAt)}>
                      {formatRelative(inv.updatedAt)}
                    </td>
                    <td className="data-table__muted mono">{formatDateTime(inv.createdAt)}</td>
                    <td>
                      <div className="data-table__row-actions">
                        <IconButton
                          icon={Pencil}
                          label={inv.status === "archived" ? "Renommer (réactivez d'abord l'enquête)" : "Renommer"}
                          disabled={inv.status === "archived"}
                          onClick={() => openRename(inv)}
                        />
                        <IconButton
                          icon={inv.status === "active" ? Archive : ArchiveRestore}
                          label={inv.status === "active" ? "Archiver" : "Réactiver"}
                          onClick={() => void toggleArchive(inv)}
                        />
                        <IconButton icon={Trash2} label="Supprimer" variant="danger" onClick={() => openDelete(inv)} />
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <p className="table-frame__footer">
              {items.length} enquête{items.length > 1 ? "s" : ""}
              {items.length !== counts.all && ` sur ${counts.all}`}
            </p>
          </>
        )}
      </section>
    </div>
  );
}
