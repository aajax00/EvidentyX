# Système de design

> Document vivant : il décrit les jetons et composants **actuellement** disponibles dans `apps/desktop/src`.

## Principes

L'esthétique est celle d'un **poste d'analyse forensique** : sobre et dense, surfaces sombres légèrement bleutées posées sur une grille de coordonnées très discrète, bordures fines de 1 px, **angles droits partout** (seule exception : l'anneau du chargeur « X »), ni ombres décoratives ni dégradés. Un seul accent, le **bleu du logo** : focus, sélection, éléments actifs, action principale et le X de la marque. **Thème sombre uniquement.**

Repères visuels :

- **Marque** : « EVIDENTY**X** » en capitales, le X en bleu, à côté du logo complet, dans un bandeau sombre en haut de la barre latérale.
- **Largeur** : les pages occupent toute la largeur disponible ; les grilles passent sur une colonne aux petites largeurs.
- **Panneaux** : bandeau d'en-tête surélevé (`--color-surface-raised`) avec titre en petites capitales, corps compact ; fond `--color-surface-alpha` qui laisse deviner la grille.
- **Données** : chiffres, adresses, heures, identifiants et en-têtes de tableau en IBM Plex Mono ; étiquettes (`Badge`) en capitales mono sur fond et bordure teintés.
- **En-tête de page** : titre, description courte et actions alignées à droite, séparés du contenu par un filet.
- **États** : seules les anomalies sont colorées (danger, avertissement) ; pas de pastille verte « tout va bien ».

1. **Une seule source de vérité** : toutes les valeurs visuelles (couleurs, tailles, espacements, durées) sont des variables CSS définies dans [`styles/tokens.css`](../apps/desktop/src/styles/tokens.css). Aucun composant ne contient de valeur codée en dur.
2. **Hors ligne** : polices IBM Plex Sans et IBM Plex Mono embarquées dans l'application (paquets `@fontsource`, licence SIL OFL 1.1), aucune ressource externe (compatible avec la politique de sécurité du contenu stricte).
3. **Densité** : l'outil affiche beaucoup de données ; corps de texte à 13 px, contrôles de 24 à 30 px de haut, données techniques en monospace (classe `.mono`).
4. **Accessibilité** : contraste suffisant, anneau de focus visible au clavier, libellés reliés aux champs, messages d'erreur annoncés, respect de `prefers-reduced-motion`.
5. **Honnêteté** : un bouton a toujours une action. Le typage de `Button` impose un `onClick` ou `type="submit"`.

## Logo

Logo source en pleine résolution : `docs/assets/logo.png` (fond transparent). Fichiers dérivés dans `apps/desktop/src/assets/` :

| Fichier | Usage |
|---|---|
| `logo.png` | Logo en bleu ; ses zones bleu nuit sont éclaircies en bleu pour rester visibles sur le fond sombre |
| `mark.png` | Emblème seul (loupe à empreinte), pour la barre latérale repliée |

Composants `BrandLogo` et `BrandMark` : `src/shell/BrandLogo.tsx`. Les icônes de l'application (`src-tauri/icons/`) sont générées par `npx tauri icon src-tauri/icons/source.png` à partir du logo posé sur un carreau bleu nuit.

Les icônes de l'application posent le logo sur un **fond clair** (bleu très pâle), pour qu'il se distingue dans la barre des tâches et le dock sombres.

**Icône dans la barre des tâches (Linux)** : le bureau associe la fenêtre (identifiant `evidentyx-desktop`, nom du binaire) à un fichier `evidentyx-desktop.desktop`. Les paquets `.deb` et `.rpm` l'installent. Pour l'application lancée depuis les sources, `scripts/raccourci-linux.sh` le crée dans `~/.local/share/applications` (à relancer si le dépôt change de place).

## Thème

L'application n'a qu'un thème, sombre. Toutes les couleurs sont définies sur `:root` dans `tokens.css`.

## Jetons

| Famille | Préfixe | Exemples |
|---|---|---|
| Couleurs de surface | `--color-` | `bg`, `surface`, `surface-raised`, `surface-hover`, `surface-sunken`, `border`, `border-strong` |
| Couleurs de texte | `--color-text` | `text`, `text-muted`, `text-subtle`, `text-on-accent` |
| Action principale | `--color-primary` | `primary`, `primary-hover`, `text-on-primary` |
| Couleurs sémantiques | `--color-` | `accent`, `success`, `warning`, `danger`, `info` (+ variantes `-subtle`, `-hover`) |
| Gravité (phase 4) | `--color-severity-` | `low`, `medium`, `high`, `critical` |
| Typographie | `--text-`, `--weight-`, `--leading-`, `--tracking-`, `--font-` | `--text-md` (13 px), `--text-2xs` (titres de section), `--font-mono` |
| Espacements | `--space-` | grille de 4 px : `--space-1` (4 px) à `--space-12` (48 px) |
| Angles | `--radius` | toujours `0` : aucun élément arrondi |
| Bordures | `--border-width` | `1px` |
| Contrôles | `--control-` | `sm` (24 px), `md` (30 px) |
| Animation | `--duration-`, `--ease-` | `fast` (110 ms), `normal` (160 ms), `standard` |
| Plans | `--z-` | `dropdown`, `overlay`, `modal`, `toast` |

## Composants

Importés depuis `src/components/ui` :

| Composant | Rôle | Propriétés principales |
|---|---|---|
| `Button` | Action | `variant` (`primary` bleu, `secondary`, `ghost`, `danger` en contour rouge), `size` (`sm`, `md`), `loading`, `aria-pressed` pour une bascule, `onClick` ou `type="submit"` |
| `Badge` | Étiquette d'état, capitales en chasse fixe, bordure de la couleur du ton | `tone` (`neutral`, `accent`, `success`, `warning`, `danger`, `info`) |
| `Panel` | Conteneur titré : bandeau d'en-tête surélevé, titre en petites capitales | `title`, `actions` |
| `TextField` / `TextAreaField` | Saisie avec libellé | `label`, `hint`, `error`, `required` |
| `Spinner` | Chargeur « X » : anneau tournant autour du X bleu (seul élément circulaire) | `size` (`sm` 16 px, `md` 22 px), `label` (lu par les lecteurs d'écran) |
| `IconButton` | Action réduite à une icône | `icon`, `label` (aria-label + info-bulle), `variant` |
| `Dialog` | Boîte de dialogue modale (`<dialog>` natif : focus piégé, Échap, retour du focus) | `title`, `description`, `footer`, `width` |
| `SegmentedControl` | Choix exclusif (groupe radio, flèches ←/→) | `options` (avec `count`), `value`, `onChange` |
| `Select` | Liste déroulante native avec libellé en ligne | `label`, `options`, `value`, `onChange` |
| `SearchField` | Recherche avec icône et effacement (Échap) | `label`, `value`, `onChange` |
| `EmptyState` | État vide : pourquoi et quoi faire | `icon`, `title`, `action` |
| `Switch` | Interrupteur marche/arrêt (rôle `switch`), carré, avec libellé et description | `label`, `description`, `checked`, `onChange` |
| `ToastProvider` / `useToast` | Notifications brèves (5 s, `aria-live`) : liseré gauche de 4 px et barre de décompte de la couleur du ton | `notify(message, tone)` |

Les classes CSS des composants sont préfixées par `ui-` pour éviter les collisions. Classes utilitaires globales : `.mono` (données techniques), `.section-title` (titre en petites capitales), `.sr-only` (texte réservé aux lecteurs d'écran).

Mises en page partagées (`src/pages/pages.css`) : `.page-header` (titre, description, actions), `.kpi-grid` / `.kpi-card` (chiffres clés : libellé en petites capitales, valeur en mono, précision), `.data-table` (tableau dense, en-têtes en capitales mono), `.home-grid` (colonne principale et colonne latérale). Variante `ui-panel--flush` : panneau sans marge intérieure, pour un tableau ou une liste bord à bord.

**Tableau de bord** : en-tête de page ; quatre chiffres clés réels (enquêtes actives sur le total, preuves enregistrées avec intègres et analysées, événements extraits, dernière activité) ; tableau des enquêtes (référence `#id` en bleu, statut, preuves, événements, mise à jour) ; colonne latérale « Preuves récentes » (intégrité, enquête, fichier, taille, ajout) et « Feuille de route ».

**Guide d'utilisation** (`src/pages/GuidePage.tsx`, route `guide` avec ancre facultative) : sommaire fixe à gauche, une section par écran ; tout nouvel écran ou comportement s'y documente.

**Graphe des entités** (`src/timeline/EntityGraph.tsx`) : scène sur canevas à fond enfoncé avec sa propre grille de 40 px qui suit le zoom ; molette pour zoomer vers le curseur, glisser le fond pour déplacer, glisser une entité pour la déplacer ; barre de zoom (−, pourcentage, +, Ajuster, 1:1) en haut à droite ; légende en bas à gauche. Adresses IP en losange rouge, hôtes (machine qui a écrit le journal) en carré bleu, comptes utilisateurs en carré ambre. Chaque lien porte une flèche (source → destination), son type et son nombre d'événements : **connexion** (adresse → adresse), **depuis** (compte → adresse d'où il a été utilisé), **vue par** (adresse → hôte qui l'a journalisée). Une phrase au-dessus du graphe rappelle ce vocabulaire. Inspecteur à droite : nom, événements liés, relations écrites en phrases (« se connecte à… », « compte root utilisé depuis cette adresse »), clés de lecture factuelles quand les nombres le justifient (plusieurs comptes depuis une adresse, une adresse vers de nombreuses destinations…), « Afficher ses événements », « Isoler ses relations » (relations directes en pointillés rouges animés, paquets de la source vers la destination, le reste estompé), « Copier ». Équivalent clavier : liste d'entités réservée aux lecteurs d'écran.

## Coque applicative

La coque (`src/shell`) structure toutes les pages :

| Élément | Rôle |
|---|---|
| Barre latérale | Bandeau de marque (retour au tableau de bord), navigation par groupes (Opérations, Analyse, Système) puis enquêtes actives récentes. Page active : fond accentué, liseré gauche de 2 px, libellé en semi-gras. Compteur des enquêtes actives. Sections non disponibles grisées avec la mention « planifiée ». Pied : version, « hors ligne ». Repliable en barre d'icônes (Ctrl+B). |
| En-tête | Bouton de repli de la barre latérale, fil d'Ariane « EvidentyX / section / enquête », accès direct à la dernière enquête modifiée, palette (Ctrl+K). |
| Barre d'état | Version de l'application et du moteur, base SQLite locale (schéma, nombre d'enquêtes, chemin en info-bulle), « hors ligne · aucune télémétrie », horloge UTC. Anomalies seules en couleur. |
| Palette de commandes | Recherche sans tenir compte des accents ni de la casse ; ↑ ↓ pour choisir, Entrée pour exécuter, Échap pour fermer. |

Ajouter une page : déclarer sa route dans `Route` (`src/shell/router.tsx`), son titre dans `SECTION_TITLES` et son entrée dans `NAVIGATION` (`src/shell/navigation.ts`), puis la rendre dans `App.tsx`. Elle apparaît automatiquement dans la palette.

## Animations

Sobres et brèves ; toutes désactivées si le système demande moins d'animations (`prefers-reduced-motion`). L'écran d'ouverture et l'arrière-plan se désactivent aussi dans les Paramètres.

| Animation | Où | Durée |
|---|---|---|
| Écran d'ouverture : le logo se décode en ASCII (caractères brouillés qui se fixent de gauche à droite), puis le logo réel apparaît | Lancement de l'application (`src/motion/SplashScreen.tsx`) ; un clic ou une touche le passe | ~2,4 s |
| Arrière-plan « forensique » (copie exacte de la maquette, canevas fixe sur toute la fenêtre, derrière la coque) : grille de coordonnées de 56 px très discrète, 12 à 26 points carrés qui dérivent et se relient en équerre à moins de 170 px | Derrière toute l'interface, visible entre les panneaux (`src/motion/NetworkBackground.tsx`), 30 images/s, en pause fenêtre masquée | continue |
| Chargeur « X » : anneau qui tourne autour du X bleu | Chargements (`Spinner`) | 0,9 s par tour |
| Graphe : voisins isolés en pointillés défilants, paquet carré qui parcourt chaque lien | `EntityGraph`, uniquement quand « Isoler les voisins » est actif | continue |
| Notification : entrée par la droite, barre de décompte | `ToastProvider` | 180 ms, puis 5 s |
| Ouverture des boîtes de dialogue et de la palette | Fondu et léger déplacement | 160–180 ms |
| Changement de page | Fondu et montée de 6 px | 160 ms |

Le logo ASCII (`src/assets/logoAscii.ts`) est généré depuis `docs/assets/logo.png` par `python3 scripts/generate-ascii-logo.py` (dans `apps/desktop`, nécessite Pillow).

## Mode démonstration

Dans un navigateur (`npm run dev`, sans l'hôte Tauri), les commandes sont servies par un moteur en mémoire (`src/dev/mockBackend.ts`) avec quelques enquêtes fictives suffixées « (démo) ». Un bandeau et la barre d'état le signalent. Ce mode n'existe pas dans la version publiée ; la référence des règles reste le moteur Rust.

### Raccourcis clavier

| Raccourci | Action |
|---|---|
| Ctrl+K (⌘K sous macOS) | Ouvrir ou fermer la palette de commandes |
| Ctrl+B (⌘B) | Replier ou déplier la barre latérale |
| Ctrl+N (⌘N) | Nouvelle enquête |
| / | Rechercher dans la liste des enquêtes |

Icônes : [Lucide](https://lucide.dev) (`lucide-react`, licence ISC), taille 16 px dans la navigation.

## Catalogue

En développement, le catalogue affiche tous les jetons et composants :

```bash
cd apps/desktop
npm run dev
```

puis ouvrir <http://localhost:1420/#catalogue> (ou ajouter `#catalogue` dans la fenêtre de `npm run tauri dev`). Il est exclu de la version publiée.

## Ajouter un composant

1. Créer `src/components/ui/<Nom>.tsx` et `<nom>.css`, en n'utilisant que des jetons.
2. L'exporter depuis `src/components/ui/index.ts`.
3. L'ajouter au catalogue (`src/dev/DesignCatalog.tsx`) et à ce document.
