# Système de design

> Document vivant : il décrit les jetons et composants **actuellement** disponibles dans `apps/desktop/src`.

## Principes

Direction artistique « console d'investigation » : fond **bleu nuit profond**, surfaces superposées à peine éclairées par le haut (`--surface-panel`), bordures fines de 1 px, **angles droits partout (aucun arrondi)**. Accent principal : le **bleu du logo** (actions, sélection, focus, données). Accent secondaire **violet**, réservé aux dégradés de marque (`--gradient-brand` : élément actif de la navigation, bandeau du tableau de bord, graphiques) et jamais seul sur une action. Lisibilité d'abord : corps de texte 14 px, titres de panneau 16 px en clair, chiffres clés 28 px ; peu d'éléments par écran, chaque chiffre affiché est réel. **Thème sombre uniquement.**

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

## Thème

L'application n'a qu'un thème, sombre. Toutes les couleurs sont définies sur `:root` dans `tokens.css`.

## Jetons

| Famille | Préfixe | Exemples |
|---|---|---|
| Couleurs de surface | `--color-` | `bg`, `surface`, `surface-raised`, `surface-hover`, `surface-sunken`, `border`, `border-strong` |
| Couleurs de texte | `--color-text` | `text`, `text-muted`, `text-subtle`, `text-on-accent` |
| Action principale | `--color-primary` | `primary`, `primary-hover`, `text-on-primary` |
| Couleurs sémantiques | `--color-` | `accent`, `success`, `warning`, `danger`, `info` (+ variantes `-subtle`, `-hover`) |
| Marque | `--color-violet`, `--gradient-brand`, `--gradient-brand-vertical`, `--surface-panel` | dégradé bleu → violet ; fond des panneaux et cartes |
| Graphiques | `--chart-1` à `--chart-7` | séries dans l'ordre : bleu, violet, cyan, vert, ambre, rouge, gris |
| Gravité (phase 4) | `--color-severity-` | `low`, `medium`, `high`, `critical` |
| Typographie | `--text-`, `--weight-`, `--leading-`, `--tracking-`, `--font-` | `--text-md` (14 px, corps), `--text-lg` (16 px, titres de panneau), `--text-2xl` (28 px, chiffres clés), `--text-3xl` (32 px, titre du tableau de bord), `--text-2xs` (titres de section), `--font-mono` |
| Espacements | `--space-` | grille de 4 px : `--space-1` (4 px) à `--space-12` (48 px) |
| Angles | `--radius` | toujours `0` : aucun élément arrondi |
| Bordures | `--border-width` | `1px` |
| Contrôles | `--control-` | `sm` (28 px), `md` (34 px) |
| Animation | `--duration-`, `--ease-` | `fast` (110 ms), `normal` (160 ms), `standard` |
| Plans | `--z-` | `dropdown`, `overlay`, `modal`, `toast` |

## Composants

Importés depuis `src/components/ui` :

| Composant | Rôle | Propriétés principales |
|---|---|---|
| `Button` | Action | `variant` (`primary` bleu, `secondary`, `ghost`, `danger` en contour rouge), `size` (`sm`, `md`), `loading`, `aria-pressed` pour une bascule, `onClick` ou `type="submit"` |
| `Badge` | Étiquette d'état | `tone` (`neutral`, `accent`, `success`, `warning`, `danger`, `info`) |
| `Panel` | Conteneur titré (titre en petites capitales) | `title`, `actions` |
| `TextField` / `TextAreaField` | Saisie avec libellé | `label`, `hint`, `error`, `required` |
| `Spinner` | Chargement | `size`, `label` (lu par les lecteurs d'écran) |
| `IconButton` | Action réduite à une icône | `icon`, `label` (aria-label + info-bulle), `variant` |
| `Dialog` | Boîte de dialogue modale (`<dialog>` natif : focus piégé, Échap, retour du focus) | `title`, `description`, `footer`, `width` |
| `SegmentedControl` | Choix exclusif (groupe radio, flèches ←/→) | `options` (avec `count`), `value`, `onChange` |
| `Select` | Liste déroulante native avec libellé en ligne | `label`, `options`, `value`, `onChange` |
| `SearchField` | Recherche avec icône et effacement (Échap) | `label`, `value`, `onChange` |
| `EmptyState` | État vide : pourquoi et quoi faire | `icon`, `title`, `action` |
| `Switch` | Interrupteur marche/arrêt (rôle `switch`), carré, avec libellé et description | `label`, `description`, `checked`, `onChange` |
| `ToastProvider` / `useToast` | Notifications brèves (5 s, `aria-live`) | `notify(message, tone)` |

Les classes CSS des composants sont préfixées par `ui-` pour éviter les collisions. Classes utilitaires globales : `.mono` (données techniques), `.section-title` (titre en petites capitales), `.sr-only` (texte réservé aux lecteurs d'écran).

## Coque applicative

La coque (`src/shell`) structure toutes les pages :

| Élément | Rôle |
|---|---|
| Barre latérale | Navigation par groupes, puis les enquêtes actives récentes. Page active : fond en dégradé de marque et liseré gauche bleu → violet. Sections non disponibles grisées avec la mention « planifiée » et leur échéance en info-bulle. Repliable en barre d'icônes. |
| En-tête | Fil d'Ariane (section, enquête ouverte) et accès à la palette ; sur le tableau de bord, la devise. |
| Barre d'état | État du moteur et de la base locale (chemin en info-bulle), nombre d'enquêtes. |
| Palette de commandes | Recherche sans tenir compte des accents ni de la casse ; ↑ ↓ pour choisir, Entrée pour exécuter, Échap pour fermer. |

Ajouter une page : déclarer sa route dans `Route` (`src/shell/router.tsx`), son titre dans `SECTION_TITLES` et son entrée dans `NAVIGATION` (`src/shell/navigation.ts`), puis la rendre dans `App.tsx`. Elle apparaît automatiquement dans la palette.

## Animations

Sobres et brèves ; toutes désactivées si le système demande moins d'animations (`prefers-reduced-motion`). L'écran d'ouverture et l'arrière-plan se désactivent aussi dans les Paramètres.

| Animation | Où | Durée |
|---|---|---|
| Écran d'ouverture : le logo se décode en ASCII (caractères brouillés qui se fixent de gauche à droite), puis le logo réel apparaît | Lancement de l'application (`src/motion/SplashScreen.tsx`) ; un clic ou une touche le passe | ~2,4 s |
| Arrière-plan : points carrés qui dérivent lentement et se relient quand ils se rapprochent | Derrière toute l'interface, visible entre les panneaux (`src/motion/NetworkBackground.tsx`), 30 images/s, en pause fenêtre masquée | continue |
| Bandeau du tableau de bord : relief en fil de fer bleu → violet, en perspective, qui ondule lentement et s'estompe sous le texte | `src/motion/TerrainBanner.tsx`, 24 images/s ; image fixe si l'animation d'arrière-plan est coupée | continue |
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
