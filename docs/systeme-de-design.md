# Système de design

> Document vivant : il décrit les jetons et composants **actuellement** disponibles dans `apps/desktop/src`.

## Principes

L'esthétique est celle d'un outil d'analyste, inspirée de [COSINT](https://github.com/k0nrd/COSINT) : surfaces graphite neutres, bordures fines de 1 px, **angles droits partout (aucun arrondi)**, ombres quasi absentes, interface compacte. L'accent bleu acier est réservé au focus, à la sélection et aux liens ; l'action principale est un bouton clair sur fond sombre (sombre sur fond clair en thème clair).

1. **Une seule source de vérité** : toutes les valeurs visuelles (couleurs, tailles, espacements, durées) sont des variables CSS définies dans [`styles/tokens.css`](../apps/desktop/src/styles/tokens.css). Aucun composant ne contient de valeur codée en dur.
2. **Hors ligne** : polices IBM Plex Sans et IBM Plex Mono embarquées dans l'application (paquets `@fontsource`, licence SIL OFL 1.1), aucune ressource externe (compatible avec la politique de sécurité du contenu stricte).
3. **Densité** : l'outil affiche beaucoup de données ; corps de texte à 13 px, contrôles de 24 à 30 px de haut, données techniques en monospace (classe `.mono`).
4. **Accessibilité** : contraste suffisant dans les deux thèmes, anneau de focus visible au clavier, libellés reliés aux champs, messages d'erreur annoncés, respect de `prefers-reduced-motion`.
5. **Honnêteté** : un bouton a toujours une action. Le typage de `Button` impose un `onClick` ou `type="submit"`.

## Thèmes

Le thème **sombre** est le thème par défaut. Le thème clair s'active en posant `data-theme="light"` sur `<html>` (le réglage utilisateur arrivera avec les paramètres, étape 1.7).

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
| `Button` | Action | `variant` (`primary` clair, `secondary`, `ghost`, `danger` en contour rouge), `size` (`sm`, `md`), `loading`, `aria-pressed` pour une bascule, `onClick` ou `type="submit"` |
| `Badge` | Étiquette d'état | `tone` (`neutral`, `accent`, `success`, `warning`, `danger`, `info`) |
| `Panel` | Conteneur titré (titre en petites capitales) | `title`, `actions` |
| `TextField` / `TextAreaField` | Saisie avec libellé | `label`, `hint`, `error`, `required` |
| `Spinner` | Chargement | `size`, `label` (lu par les lecteurs d'écran) |

Les classes CSS des composants sont préfixées par `ui-` pour éviter les collisions. Classes utilitaires globales : `.mono` (données techniques), `.section-title` (titre en petites capitales), `.sr-only` (texte réservé aux lecteurs d'écran).

## Catalogue

En développement, le catalogue affiche tous les jetons et composants, avec un sélecteur de thème :

```bash
cd apps/desktop
npm run dev
```

puis ouvrir <http://localhost:1420/#catalogue> (ou ajouter `#catalogue` dans la fenêtre de `npm run tauri dev`). Il est exclu de la version publiée.

## Ajouter un composant

1. Créer `src/components/ui/<Nom>.tsx` et `<nom>.css`, en n'utilisant que des jetons.
2. L'exporter depuis `src/components/ui/index.ts`.
3. L'ajouter au catalogue (`src/dev/DesignCatalog.tsx`) et à ce document.
