// Catalogue du système de design — outil de développement uniquement.
// Accessible via http://localhost:1420/#catalogue avec `npm run dev` ou `npm run tauri dev`.
// Exclu de la version publiée (voir main.tsx).

import { useState, type FormEvent } from "react";
import { Archive, FolderSearch, Pencil, Plus, Trash2 } from "lucide-react";
import {
  Badge,
  Button,
  Dialog,
  EmptyState,
  IconButton,
  Panel,
  SearchField,
  SegmentedControl,
  Select,
  Spinner,
  TextAreaField,
  TextField,
  ToastProvider,
  useToast,
  type BadgeTone,
  type ButtonVariant,
} from "../components/ui";
import "./catalog.css";

const VARIANTS: ButtonVariant[] = ["primary", "secondary", "ghost", "danger"];
const TONES: BadgeTone[] = ["neutral", "accent", "success", "warning", "danger", "info"];
const COLORS = [
  "bg",
  "surface",
  "surface-raised",
  "surface-hover",
  "surface-sunken",
  "border",
  "border-strong",
  "text",
  "text-muted",
  "text-subtle",
  "accent",
  "primary",
  "success",
  "warning",
  "danger",
  "info",
  "severity-low",
  "severity-medium",
  "severity-high",
  "severity-critical",
];

export function DesignCatalog() {
  return (
    <ToastProvider>
      <Catalog />
    </ToastProvider>
  );
}

function Catalog() {
  const { notify } = useToast();
  const [segment, setSegment] = useState<"a" | "b" | "c">("a");
  const [sortValue, setSortValue] = useState<"x" | "y">("x");
  const [search, setSearch] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [clicks, setClicks] = useState(0);
  const [loading, setLoading] = useState(false);
  const [name, setName] = useState("");
  const [submitted, setSubmitted] = useState<string | null>(null);

  function simulateLoading() {
    setLoading(true);
    window.setTimeout(() => setLoading(false), 1500);
  }

  function onSubmit(event: FormEvent) {
    event.preventDefault();
    setSubmitted(name);
  }

  const nameError = name.trim() === "" && submitted !== null ? "Le nom est obligatoire." : undefined;

  return (
    <div className="catalog">
      <header className="catalog__header">
        <h1>Catalogue du système de design</h1>
        <Badge tone="warning">outil de développement</Badge>
      </header>

      <Panel title="Couleurs">
        <div className="catalog__swatches">
          {COLORS.map((c) => (
            <div key={c} className="catalog__swatch">
              <span className="catalog__chip" style={{ background: `var(--color-${c})` }} />
              <code>--color-{c}</code>
            </div>
          ))}
        </div>
      </Panel>

      <Panel title="Typographie">
        <div className="catalog__stack">
          <h1>Titre 1 — Enquête « Intrusion serveur web »</h1>
          <h2>Titre 2 — Chronologie</h2>
          <h3>Titre 3 — Événements corrélés</h3>
          <p>Corps de texte : chaque corrélation renvoie aux événements sources qui la justifient.</p>
          <p className="catalog__muted">Texte secondaire : informations complémentaires.</p>
          <p className="section-title">Titre de section</p>
          <code>2026-11-18T19:53:04Z 192.168.1.24 → 10.0.0.5:443 sha256:9f86d0…</code>
        </div>
      </Panel>

      <Panel title="Boutons" actions={<Badge>{clicks} clic(s)</Badge>}>
        <div className="catalog__row">
          {VARIANTS.map((v) => (
            <Button key={v} variant={v} onClick={() => setClicks((n) => n + 1)}>
              {v}
            </Button>
          ))}
        </div>
        <div className="catalog__row">
          {VARIANTS.map((v) => (
            <Button key={v} variant={v} size="sm" onClick={() => setClicks((n) => n + 1)}>
              {v} sm
            </Button>
          ))}
        </div>
        <div className="catalog__row">
          <Button variant="primary" loading={loading} onClick={simulateLoading}>
            Simuler un chargement
          </Button>
          <Button disabled onClick={() => setClicks((n) => n + 1)}>
            Désactivé
          </Button>
        </div>
      </Panel>

      <Panel title="Boutons avec icône">
        <div className="catalog__row">
          <Button variant="primary" icon={Plus} onClick={() => setClicks((n) => n + 1)}>
            Nouvelle enquête
          </Button>
          <Button icon={Pencil} onClick={() => setClicks((n) => n + 1)}>
            Renommer
          </Button>
          <Button variant="danger" icon={Trash2} onClick={() => setClicks((n) => n + 1)}>
            Supprimer
          </Button>
          <IconButton icon={Pencil} label="Renommer" onClick={() => setClicks((n) => n + 1)} />
          <IconButton icon={Archive} label="Archiver" onClick={() => setClicks((n) => n + 1)} />
          <IconButton icon={Trash2} label="Supprimer" variant="danger" onClick={() => setClicks((n) => n + 1)} />
        </div>
      </Panel>

      <Panel title="Filtres">
        <div className="catalog__row">
          <div style={{ width: 280 }}>
            <SearchField label="Recherche" placeholder="Rechercher…" value={search} onChange={setSearch} />
          </div>
          <SegmentedControl
            label="Statut"
            value={segment}
            onChange={setSegment}
            options={[
              { value: "a", label: "Actives", count: 12 },
              { value: "b", label: "Archivées", count: 3 },
              { value: "c", label: "Toutes", count: 15 },
            ]}
          />
          <Select
            label="Trier par"
            value={sortValue}
            onChange={setSortValue}
            options={[
              { value: "x", label: "Dernière modification" },
              { value: "y", label: "Nom (A → Z)" },
            ]}
          />
        </div>
      </Panel>

      <Panel title="Dialogue, notifications, état vide">
        <div className="catalog__row">
          <Button onClick={() => setDialogOpen(true)}>Ouvrir un dialogue</Button>
          <Button onClick={() => notify("Opération réussie (démonstration).")}>Notification</Button>
          <Button onClick={() => notify("Échec de l'opération (démonstration).", "danger")}>Notification d'erreur</Button>
        </div>
        <EmptyState icon={FolderSearch} title="Aucun élément">
          Explique pourquoi la liste est vide et propose quoi faire.
        </EmptyState>
        {dialogOpen && (
          <Dialog
            title="Titre du dialogue"
            description="Texte d'introduction du dialogue."
            onClose={() => setDialogOpen(false)}
            footer={
              <>
                <Button onClick={() => setDialogOpen(false)}>Annuler</Button>
                <Button variant="primary" onClick={() => setDialogOpen(false)}>
                  Confirmer
                </Button>
              </>
            }
          >
            Contenu du dialogue.
          </Dialog>
        )}
      </Panel>

      <Panel title="Badges et indicateurs">
        <div className="catalog__row">
          {TONES.map((t) => (
            <Badge key={t} tone={t}>
              {t}
            </Badge>
          ))}
          <Spinner size="sm" />
          <Spinner />
        </div>
      </Panel>

      <Panel title="Champs de formulaire">
        <form className="catalog__stack" onSubmit={onSubmit} noValidate>
          <TextField
            label="Nom de l'enquête"
            required
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Ex. Intrusion serveur web"
            hint="120 caractères maximum."
            error={nameError}
          />
          <TextAreaField label="Description" placeholder="Contexte, périmètre…" />
          <TextField label="Champ désactivé" disabled value="Lecture seule" readOnly />
          <div className="catalog__row">
            <Button type="submit" variant="primary">
              Valider (démonstration)
            </Button>
            {submitted !== null && nameError === undefined && (
              <span className="catalog__muted">Saisi : « {submitted} » — rien n'est enregistré.</span>
            )}
          </div>
        </form>
      </Panel>
    </div>
  );
}
