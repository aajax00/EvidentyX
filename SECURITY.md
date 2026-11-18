# Politique de sécurité

## Signaler une vulnérabilité

**N'ouvrez pas de ticket public pour une vulnérabilité.**

Utilisez le signalement privé de GitHub : onglet **Security** du dépôt → **Report a vulnerability**.

Merci d'indiquer :

- la version concernée ;
- une description du problème et de son impact ;
- les étapes de reproduction, idéalement avec un fichier d'exemple minimal (anonymisé).

Nous accusons réception sous 7 jours et visons un correctif ou une mesure d'atténuation sous 90 jours.

## Versions prises en charge

| Version | Prise en charge |
|---|---|
| 0.x (dernière) | ✅ |
| Versions antérieures | ❌ |

## Modèle de menace (résumé)

EVIDENTYX analyse des fichiers potentiellement malveillants. Principes appliqués :

- Tout fichier importé est considéré comme **non fiable**.
- Les fichiers importés ne sont **jamais exécutés**.
- Aucune commande système n'est construite à partir de données importées.
- Les analyseurs (parsers) isolent leurs erreurs et appliquent des limites de ressources.
- Les fichiers sources sont ouverts en lecture seule et ne sont jamais modifiés.
- Les requêtes SQL sont paramétrées.
- L'application de bureau applique une politique de sécurité du contenu (CSP) stricte et des permissions Tauri minimales.
- Aucune donnée n'est transmise sur le réseau en fonctionnement normal.

## Signature des binaires

Tant que des certificats de signature ne sont pas disponibles, les paquets publiés **ne sont pas signés** ; c'est indiqué explicitement dans chaque version. Vérifiez les sommes SHA-256 publiées avec chaque version.
