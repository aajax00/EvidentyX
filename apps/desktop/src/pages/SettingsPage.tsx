import { useEffect, useState } from "react";
import { getStorageInfo, type InvestigationSort, type StatusFilter, type StorageInfo } from "../api";
import { Panel, SegmentedControl, Select, Switch, useToast } from "../components/ui";
import { useReducedMotion } from "../motion/useReducedMotion";
import { useSettings } from "../settings/SettingsProvider";
import { Kbd } from "../shell/Kbd";
import type { Settings } from "../api";

const SORT_OPTIONS: ReadonlyArray<{ value: InvestigationSort; label: string }> = [
  { value: "updated_desc", label: "Dernière modification" },
  { value: "created_desc", label: "Date de création" },
  { value: "name_asc", label: "Nom (A → Z)" },
];

const SHORTCUTS: ReadonlyArray<[string, string]> = [
  ["mod+k", "Ouvrir la palette de commandes"],
  ["mod+n", "Nouvelle enquête"],
  ["mod+b", "Replier ou déplier la barre latérale"],
  ["/", "Rechercher dans la liste des enquêtes"],
  ["Échap", "Fermer une boîte de dialogue ou la palette"],
];

export function SettingsPage() {
  const { settings, update } = useSettings();
  const { notify } = useToast();
  const reducedMotion = useReducedMotion();
  const [storage, setStorage] = useState<StorageInfo | null>(null);

  useEffect(() => {
    getStorageInfo().then(setStorage).catch(() => setStorage(null));
  }, []);

  const change = (changes: Partial<Settings>) => {
    update(changes).catch((e: { message?: string }) =>
      notify(`Réglage non enregistré : ${e.message ?? "erreur inconnue"}`, "danger"),
    );
  };

  return (
    <div className="page page--narrow">
      <Panel title="Animations">
        {reducedMotion && (
          <p className="settings-note">
            Votre système demande de réduire les animations : elles restent désactivées, quels que soient ces réglages.
          </p>
        )}
        <Switch
          label="Écran d'ouverture"
          description="Le logo se dessine au lancement de l'application (environ 2 secondes, passable d'un clic)."
          checked={settings.splashScreen}
          disabled={reducedMotion}
          onChange={(v) => change({ splashScreen: v })}
        />
        <Switch
          label="Animation d'arrière-plan"
          description="Réseau de points derrière l'interface et relief animé du tableau de bord (fixe si désactivé)."
          checked={settings.backgroundAnimation}
          disabled={reducedMotion}
          onChange={(v) => change({ backgroundAnimation: v })}
        />
      </Panel>

      <Panel title="Interface">
        <Switch
          label="Barre latérale repliée au démarrage"
          description="Elle reste repliable à tout moment avec Ctrl+B."
          checked={settings.sidebarCollapsed}
          onChange={(v) => change({ sidebarCollapsed: v })}
        />
      </Panel>

      <Panel title="Liste des enquêtes">
        <div className="settings-row">
          <div className="settings-row__text">
            <span className="settings-row__label">Affichage par défaut</span>
            <span className="settings-row__description">Statut sélectionné à l'ouverture de la liste.</span>
          </div>
          <SegmentedControl<StatusFilter>
            label="Statut par défaut"
            value={settings.defaultStatusFilter}
            onChange={(v) => change({ defaultStatusFilter: v })}
            options={[
              { value: "active", label: "Actives" },
              { value: "archived", label: "Archivées" },
              { value: "all", label: "Toutes" },
            ]}
          />
        </div>
        <div className="settings-row">
          <div className="settings-row__text">
            <span className="settings-row__label">Tri par défaut</span>
            <span className="settings-row__description">Ordre appliqué à l'ouverture de la liste.</span>
          </div>
          <Select label="Tri" value={settings.defaultSort} options={SORT_OPTIONS} onChange={(v) => change({ defaultSort: v })} />
        </div>
      </Panel>

      <Panel title="Données">
        <dl className="page-facts">
          <dt>Base de données</dt>
          <dd className="mono selectable">{storage?.path ?? "—"}</dd>
          <dt>Version du schéma</dt>
          <dd className="mono">{storage?.schemaVersion ?? "—"}</dd>
          <dt>Confidentialité</dt>
          <dd>Les données restent sur ce poste : aucun compte, aucune télémétrie, aucun envoi.</dd>
        </dl>
        {storage?.error && <p className="page-error settings-error">{storage.error}</p>}
      </Panel>

      <Panel title="Raccourcis clavier">
        <dl className="shortcut-list">
          {SHORTCUTS.map(([keys, action]) => (
            <div key={keys}>
              <dt>
                <Kbd shortcut={keys} />
              </dt>
              <dd>{action}</dd>
            </div>
          ))}
        </dl>
      </Panel>
    </div>
  );
}
