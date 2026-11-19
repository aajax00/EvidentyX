import logo from "../assets/logo.png";
import mark from "../assets/mark.png";

/** Logo EVIDENTYX, dans ses couleurs bleues (zones bleu nuit éclaircies pour le fond sombre). */
export function BrandLogo({ className }: { className?: string }) {
  return <img src={logo} alt="EVIDENTYX" className={`brand-logo ${className ?? ""}`} draggable={false} />;
}

/** Emblème seul (loupe à empreinte), pour les espaces réduits. */
export function BrandMark({ size = 28 }: { size?: number }) {
  return <img src={mark} alt="EVIDENTYX" width={size} height={size} className="brand-mark" draggable={false} />;
}
