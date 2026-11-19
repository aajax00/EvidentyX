import { Panel, Spinner } from "../components/ui";
import { useEngine } from "../engine";
import { BrandLogo } from "../shell/BrandLogo";

const PRINCIPLES: ReadonlyArray<[string, string]> = [
  ["Déterministe", "Aucune IA générative : une même entrée et les mêmes règles produisent toujours le même résultat."],
  ["Explicable", "Chaque corrélation et chaque détection renvoie aux événements sources qui la justifient."],
  ["Respectueuse des preuves", "Les fichiers sources ne sont jamais modifiés ; leur intégrité est garantie par SHA-256."],
  ["Locale", "Aucune télémétrie, aucun compte, aucun envoi de données."],
];

export function AboutPage() {
  const engine = useEngine();

  return (
    <div className="page">
      <section className="about-hero">
        <BrandLogo className="about-hero__logo" />
        <div>
          <h2 className="about-hero__title">EVIDENTYX</h2>
          <p className="text-muted">Plateforme d'investigation numérique et de reconstruction d'incidents.</p>
          <p className="about-hero__motto">Chaque trace. Chaque connexion. Chaque vérité.</p>
        </div>
      </section>

      <Panel title="Versions">
        {engine.kind === "loading" && (
          <p className="page-inline">
            <Spinner size="sm" label="Connexion au moteur" /> Connexion au moteur…
          </p>
        )}
        {engine.kind === "unavailable" && <p className="page-error">{engine.reason}</p>}
        {engine.kind === "ready" && (
          <dl className="page-facts">
            <dt>Moteur</dt>
            <dd className="mono">evidentyx-core {engine.info.engineVersion}</dd>
            <dt>Application</dt>
            <dd className="mono">{engine.info.appVersion}</dd>
            <dt>Plateforme</dt>
            <dd className="mono">{engine.info.platform}</dd>
            <dt>Licence</dt>
            <dd>Apache 2.0</dd>
          </dl>
        )}
      </Panel>

      <Panel title="Principes">
        <dl className="page-principles">
          {PRINCIPLES.map(([name, text]) => (
            <div key={name}>
              <dt>{name}</dt>
              <dd>{text}</dd>
            </div>
          ))}
        </dl>
      </Panel>
    </div>
  );
}
