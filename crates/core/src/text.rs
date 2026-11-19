//! Outils de texte déterministes, sans dépendance.

/// Forme de comparaison d'un texte : minuscules, accents latins retirés,
/// ligatures dépliées, espaces regroupés. « Ça  Œuvre » → « ca oeuvre ».
///
/// Couvre les lettres accentuées du français et des langues latines courantes
/// (Latin-1 et Latin étendu A usuels). Les autres caractères sont conservés en
/// minuscules : la fonction ne perd jamais d'information au point de rendre
/// deux textes sans rapport identiques.
pub fn fold(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        match fold_char(c) {
            Folded::One(f) => out.push(f),
            Folded::Two(a, b) => {
                out.push(a);
                out.push(b);
            }
        }
    }
    out
}

enum Folded {
    One(char),
    Two(char, char),
}

fn fold_char(c: char) -> Folded {
    use Folded::{One, Two};
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => One('a'),
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => One('c'),
        'ď' | 'đ' => One('d'),
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => One('e'),
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => One('g'),
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => One('i'),
        'ł' | 'ĺ' | 'ļ' | 'ľ' => One('l'),
        'ñ' | 'ń' | 'ņ' | 'ň' => One('n'),
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' => One('o'),
        'ŕ' | 'ř' => One('r'),
        'ś' | 'ŝ' | 'ş' | 'š' => One('s'),
        'ţ' | 'ť' => One('t'),
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => One('u'),
        'ý' | 'ÿ' => One('y'),
        'ź' | 'ż' | 'ž' => One('z'),
        'æ' => Two('a', 'e'),
        'œ' => Two('o', 'e'),
        'ß' => Two('s', 's'),
        other => One(other),
    }
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn retire_accents_et_casse() {
        assert_eq!(fold("Éléphant À Noël"), "elephant a noel");
        assert_eq!(fold("ÇA ŒUVRE ÆTHER straße"), "ca oeuvre aether strasse");
        assert_eq!(fold("Łódź Kraków"), "lodz krakow");
    }

    #[test]
    fn regroupe_les_espaces() {
        assert_eq!(fold("  a \t b\n\nc  "), "a b c");
        assert_eq!(fold(""), "");
    }

    #[test]
    fn conserve_les_autres_caracteres() {
        assert_eq!(fold("192.168.1.24 — Σ"), "192.168.1.24 — σ");
    }
}
