import { useEffect, type ReactNode } from "react";
import { Kbd } from "../shell/Kbd";

/** Sections du guide, dans l'ordre du sommaire. */
const SECTIONS: ReadonlyArray<{ id: string; title: string }> = [
  { id: "guide-demarrer", title: "Démarrer une enquête" },
  { id: "guide-preuves", title: "Preuves et intégrité" },
  { id: "guide-analyse", title: "Analyser une preuve" },
  { id: "guide-chronologie", title: "Chronologie" },
  { id: "guide-regroupements", title: "Regroupements et pivots" },
  { id: "guide-graphe", title: "Graphe des entités" },
  { id: "guide-raccourcis", title: "Raccourcis clavier" },
  { id: "guide-confidentialite", title: "Confidentialité" },
];

function Section({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section id={id} className="ui-panel guide-section" aria-labelledby={`${id}-titre`}>
      <header className="ui-panel__header">
        <h3 id={`${id}-titre`} className="ui-panel__title">
          {title}
        </h3>
      </header>
      <div className="ui-panel__body guide-section__body">{children}</div>
    </section>
  );
}

/**
 * Guide d'utilisation intégré : ce que fait chaque écran et comment lire les
 * résultats. `anchor` : section à afficher à l'ouverture.
 */
export function GuidePage({ anchor }: { anchor?: string | undefined }) {
  useEffect(() => {
    if (anchor) document.getElementById(anchor)?.scrollIntoView({ block: "start" });
  }, [anchor]);

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <h2 className="page-header__title">Guide d'utilisation</h2>
          <p className="page-header__desc">
            Ce que fait chaque écran, comment lire les résultats et ce qu'ils ne disent pas.
          </p>
        </div>
      </header>

      <div className="guide-layout">
        <nav className="ui-panel guide-toc" aria-label="Sommaire du guide">
          <header className="ui-panel__header">
            <h3 className="ui-panel__title">Sommaire</h3>
          </header>
          <ol className="guide-toc__list">
            {SECTIONS.map((s, i) => (
              <li key={s.id}>
                <a
                  href={`#${s.id}`}
                  onClick={(e) => {
                    e.preventDefault();
                    document.getElementById(s.id)?.scrollIntoView({ block: "start" });
                  }}
                >
                  <span className="mono">{String(i + 1).padStart(2, "0")}</span> {s.title}
                </a>
              </li>
            ))}
          </ol>
        </nav>

        <div className="guide-content">
          <Section id="guide-demarrer" title="Démarrer une enquête">
            <p>
              Une <strong>enquête</strong> regroupe les preuves et l'analyse d'un incident. Créez-la depuis le tableau de
              bord ou la page Enquêtes (<Kbd shortcut="mod+n" />), donnez-lui un nom et, si besoin, une description du
              périmètre.
            </p>
            <p>
              Une enquête <strong>archivée</strong> passe en lecture seule : ses preuves et ses événements restent
              consultables mais ne peuvent plus être modifiés. Réactivez-la pour reprendre le travail.
            </p>
          </Section>

          <Section id="guide-preuves" title="Preuves et intégrité">
            <p>
              Ajoutez des fichiers avec « Ajouter des preuves » ou en les glissant sur la page de l'enquête. EVIDENTYX
              ne copie ni ne modifie jamais un fichier : il enregistre son chemin, sa taille et son{" "}
              <strong>empreinte SHA-256</strong>, une signature qui change dès qu'un seul octet change.
            </p>
            <dl className="guide-terms">
              <dt>Intègre</dt>
              <dd>Le fichier a la même empreinte qu'à son ajout.</dd>
              <dt>Modifiée</dt>
              <dd>Le fichier a changé : ses événements sont retirés et il ne peut plus être analysé.</dd>
              <dt>Introuvable</dt>
              <dd>Le fichier n'est plus à son emplacement (déplacé, supprimé, disque débranché).</dd>
            </dl>
            <p>« Tout vérifier » recalcule l'empreinte de chaque preuve. Un même contenu n'est accepté qu'une fois par enquête.</p>
          </Section>

          <Section id="guide-analyse" title="Analyser une preuve">
            <p>
              Le bouton <strong>Analyser</strong> reconnaît le format et extrait des <strong>événements</strong> : une
              ligne de journal, une ligne de CSV ou un paquet réseau deviennent chacun un événement, avec son heure, ses
              adresses, son utilisateur, son issue et le texte d'origine.
            </p>
            <p>
              Formats lus : JSON et JSON Lines, CSV (séparateur deviné), Syslog (RFC 3164 et 5424), journaux d'accès
              Apache et Nginx, captures PCAP et PCAPNG.
            </p>
            <p>
              Pendant la lecture, l'empreinte est recalculée : si le fichier ne correspond plus, l'analyse est refusée.
              Une barre indique l'avancement et l'analyse peut être annulée (rien de cette analyse n'est alors conservé).
              Le <strong>rapport d'analyse</strong> indique ce qui a été lu, retenu, déjà présent ou illisible (avec la
              ligne ou le paquet en cause). Réanalyser n'ajoute que ce qui manque.
            </p>
          </Section>

          <Section id="guide-chronologie" title="Chronologie">
            <p>
              Ouvrez-la avec le bouton <strong>Chronologie</strong> d'une enquête : tous ses événements, du plus ancien
              au plus récent ; ceux sans heure sont à la fin. La liste reste fluide même avec des millions d'événements.
            </p>
            <dl className="guide-terms">
              <dt>Filtres</dt>
              <dd>
                Texte du message, catégorie, preuve, issue, qualité de l'heure, adresse IP (exacte si elle est
                complète, sinon par son début : « 203.0.113. »), utilisateur, hôte et période. « Réinitialiser » les
                efface.
              </dd>
              <dt>Fuseau</dt>
              <dd>Heures affichées en heure locale ou en UTC (bascule en haut de la chronologie, ou dans les Paramètres).</dd>
              <dt>≈ après l'heure</dt>
              <dd>
                Heure incertaine : la source ne donnait pas de fuseau (lue comme UTC) ou pas d'année (déduite de la date
                du fichier). À recouper avant d'en tirer une conclusion.
              </dd>
              <dt>Détail</dt>
              <dd>
                Un clic sur une ligne ouvre l'événement : champs reconnus, attributs d'origine et texte brut, avec la
                preuve, la ligne et l'octet d'où il vient.
              </dd>
            </dl>
          </Section>

          <Section id="guide-regroupements" title="Regroupements et pivots">
            <p>
              La vue <strong>Regroupements</strong> rassemble les événements retenus par les filtres par adresse IP,
              utilisateur, hôte ou flux réseau (source → destination:port), avec leur nombre, leurs échecs et leur
              première et dernière apparition.
            </p>
            <p>
              <strong>Pivoter</strong>, c'est partir d'une entité pour afficher tous ses événements : cliquez sur un
              groupe, ou sur une adresse, un utilisateur ou un hôte dans le détail d'un événement. La période et les
              autres filtres sont conservés.
            </p>
          </Section>

          <Section id="guide-graphe" title="Graphe des entités">
            <p>
              Le graphe dessine <strong>ce que les journaux disent</strong>, sans rien en déduire : qui apparaît avec qui,
              et combien de fois. Il suit les filtres de la chronologie.
            </p>
            <h4 className="guide-subtitle">Les formes : les entités</h4>
            <dl className="guide-terms">
              <dt>
                <span className="guide-shape guide-shape--ip" aria-hidden="true" /> Losange rouge
              </dt>
              <dd>Une adresse IP.</dd>
              <dt>
                <span className="guide-shape guide-shape--host" aria-hidden="true" /> Carré bleu
              </dt>
              <dd>Un hôte : la machine qui a écrit le journal (le serveur qui a enregistré la connexion, par exemple).</dd>
              <dt>
                <span className="guide-shape guide-shape--user" aria-hidden="true" /> Carré ambre
              </dt>
              <dd>Un compte utilisateur.</dd>
            </dl>
            <h4 className="guide-subtitle">Les traits : les relations</h4>
            <p>
              Un trait relie deux entités citées dans les mêmes événements. La flèche au milieu donne le sens, le texte
              donne le type et le nombre d'événements.
            </p>
            <dl className="guide-terms">
              <dt>connexion</dt>
              <dd>Une adresse en a contacté une autre (adresse source → adresse destination).</dd>
              <dt>depuis</dt>
              <dd>Un compte a été utilisé à partir de cette adresse (tentative de connexion SSH, par exemple).</dd>
              <dt>vue par</dt>
              <dd>Cette adresse apparaît dans le journal de cet hôte.</dd>
            </dl>
            <h4 className="guide-subtitle">Manipuler</h4>
            <ul className="guide-list">
              <li>Molette : zoom vers le curseur ; glisser le fond : déplacer la vue ; glisser une entité : la déplacer.</li>
              <li>Barre en haut à droite : zoom −/+, « Ajuster » (tout voir), « 1:1 » (taille réelle).</li>
              <li>Boutons « Adresses IP », « Hôtes », « Utilisateurs » : masquer ou afficher un type d'entité.</li>
              <li>Au clavier : les entités sont aussi listées pour les lecteurs d'écran.</li>
            </ul>
            <h4 className="guide-subtitle">L'inspecteur</h4>
            <p>
              Un clic sur une entité l'ouvre à droite : nombre d'événements liés et liste de ses relations en phrases
              (« se connecte à 10.0.0.5 », « compte root utilisé depuis cette adresse »). Un clic sur une relation passe
              à l'entité voisine.
            </p>
            <ul className="guide-list">
              <li>
                <strong>Afficher ses événements</strong> : revient à la chronologie, filtrée sur cette entité.
              </li>
              <li>
                <strong>Isoler ses relations</strong> : estompe tout le reste. Les liens directs passent en pointillés
                rouges et de petits carrés circulent de la source vers la destination.
              </li>
              <li>
                <strong>Copier</strong> : copie la valeur (adresse, compte, hôte).
              </li>
            </ul>
            <h4 className="guide-subtitle">Lire le graphe</h4>
            <p>Quelques situations courantes, que l'inspecteur signale quand les nombres les font apparaître :</p>
            <dl className="guide-terms">
              <dt>Plusieurs comptes depuis une adresse</dt>
              <dd>
                Courant derrière un NAT, un proxy ou un poste partagé. Si ce sont surtout des échecs, cela peut signaler
                une tentative sur plusieurs comptes (« pulvérisation de mots de passe ») : vérifiez avec « Afficher ses
                événements » puis le filtre Issue : Échec.
              </dd>
              <dt>Un compte depuis plusieurs adresses</dt>
              <dd>Normal pour un utilisateur qui change de poste ou de réseau ; à vérifier si les adresses sont inconnues ou utilisées en même temps.</dd>
              <dt>Une adresse vers de nombreuses destinations</dt>
              <dd>Poste très actif, relais, ou balayage (scan) : regardez la chronologie de cette adresse.</dd>
              <dt>Une adresse contactée par de nombreuses autres</dt>
              <dd>Serveur exposé, ou cible d'un trafic à examiner.</dd>
            </dl>
            <p className="guide-note">
              Limites : seuls les 60 liens les plus fréquents sont dessinés (affinez les filtres pour voir les autres) ;
              un lien relève une présence commune dans les journaux, pas une intention ni une compromission.
            </p>
          </Section>

          <Section id="guide-raccourcis" title="Raccourcis clavier">
            <dl className="guide-terms">
              <dt>
                <Kbd shortcut="mod+k" />
              </dt>
              <dd>Palette de commandes : aller à une page, une enquête, une action.</dd>
              <dt>
                <Kbd shortcut="mod+n" />
              </dt>
              <dd>Nouvelle enquête.</dd>
              <dt>
                <Kbd shortcut="mod+b" />
              </dt>
              <dd>Replier ou déplier la barre latérale.</dd>
              <dt>
                <Kbd shortcut="/" />
              </dt>
              <dd>Aller à la recherche de la liste des enquêtes.</dd>
              <dt>Échap</dt>
              <dd>Fermer une boîte de dialogue, la palette ou le détail d'un événement.</dd>
            </dl>
          </Section>

          <Section id="guide-confidentialite" title="Confidentialité">
            <p>
              Tout reste sur cette machine : aucune connexion réseau, aucun compte, aucune télémétrie. La base de données
              est un fichier SQLite local ; son chemin s'affiche au survol de la barre d'état.
            </p>
          </Section>
        </div>
      </div>
    </div>
  );
}
