//! Correspondance entre les champs d'un enregistrement et le modèle canonique.
//!
//! Partagée par les formats à champs nommés (JSON, CSV…). Les noms sont
//! comparés sans casse ni séparateurs (`source_ip`, `sourceIp`, `source.ip`
//! désignent le même champ). Un champ reconnu mais à valeur inexploitable (une
//! « adresse IP » qui n'en est pas une) reste dans les attributs : rien n'est
//! perdu ni inventé. La catégorie est déduite par mots-clés (heuristique
//! documentée), à défaut `other`.

use crate::time;
use evidentyx_core::event::{MAX_ATTRIBUTES, MAX_ATTRIBUTE_KEY_CHARS, MAX_ATTRIBUTE_VALUE_CHARS};
use evidentyx_core::{EventKind, EventTime, NewEvent, Outcome, SourceRef};
use std::collections::BTreeMap;
use std::net::IpAddr;

/// Valeur scalaire d'un champ.
#[derive(Debug, Clone, PartialEq)]
pub enum Scalar {
    Text(String),
    /// Nombre, avec sa forme textuelle d'origine.
    Number(f64, String),
    Bool(bool),
}

impl Scalar {
    fn text(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Number(_, s) => s.clone(),
            Self::Bool(b) => b.to_string(),
        }
    }
}

pub(crate) const TIME: &[&str] = &[
    "timestamp",
    "@timestamp",
    "timestamputc",
    "time",
    "ts",
    "datetime",
    "date",
    "eventtime",
    "timecreated",
    "createdat",
    "logtime",
    "timegenerated",
    "starttime",
    "eventdate",
    "lastrun",
    "horodatage",
    "dateheure",
];
const HOST: &[&str] = &[
    "host",
    "hostname",
    "computer",
    "computername",
    "devicename",
    "device",
    "server",
    "machine",
    "destinationhost",
    "poste",
    "hôte",
    "machinename",
];
const SRC_IP: &[&str] = &[
    "sourceip",
    "srcip",
    "src",
    "clientip",
    "sourceaddress",
    "srcaddr",
    "remoteaddr",
    "remoteip",
    "sourceipaddress",
    "ipaddress",
    "ip",
    "cip",
];
const DST_IP: &[&str] = &[
    "destinationip",
    "dstip",
    "destip",
    "dst",
    "destinationaddress",
    "dstaddr",
    "serverip",
    "targetip",
];
const SRC_PORT: &[&str] = &["sourceport", "srcport", "sport", "clientport"];
const DST_PORT: &[&str] = &[
    "destinationport",
    "dstport",
    "destport",
    "dport",
    "targetport",
    "serverport",
    "port",
];
const USER: &[&str] = &[
    "user",
    "username",
    "account",
    "accountname",
    "login",
    "userid",
    "targetusername",
    "subjectusername",
    "usr",
    "utilisateur",
    "compte",
];
const ACTION: &[&str] = &[
    "action",
    "eventtype",
    "event",
    "activity",
    "operation",
    "eventname",
    "eventaction",
    "évènement",
    "événement",
    "evenement",
];
const OUTCOME: &[&str] = &["outcome", "status", "result", "eventoutcome"];
const MESSAGE: &[&str] = &[
    "message",
    "msg",
    "description",
    "summary",
    "log",
    "text",
    "details",
    "mapdescription",
    "commentaire",
    "libellé",
    "libelle",
];

pub(crate) fn norm(key: &str) -> String {
    // Les mêmes noms de champs reviennent à chaque enregistrement : la forme
    // normalisée est mémorisée (cache borné, par fil d'exécution).
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<String, String>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    CACHE.with(|cache| {
        if let Some(n) = cache.borrow().get(key) {
            return n.clone();
        }
        let n: String = key
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | '.' | ' '))
            .flat_map(char::to_lowercase)
            .collect();
        let mut cache = cache.borrow_mut();
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(key.to_owned(), n.clone());
        n
    })
}

/// Champs restant à consommer, indexés par nom normalisé.
struct Fields {
    by_norm: BTreeMap<String, String>, // nom normalisé → nom d'origine
    values: BTreeMap<String, Scalar>,  // nom d'origine → valeur
}

impl Fields {
    /// Premier alias présent, non vide ; renvoie (nom d'origine, valeur).
    fn find(&self, aliases: &[&str]) -> Option<(String, Scalar)> {
        aliases.iter().find_map(|a| {
            let original = self.by_norm.get(*a)?;
            let value = self.values.get(original)?;
            (!value.text().trim().is_empty()).then(|| (original.clone(), value.clone()))
        })
    }

    fn take(&mut self, original: &str) {
        self.values.remove(original);
        self.by_norm.retain(|_, v| v != original);
    }

    /// Consomme le premier alias dont la valeur est acceptée par `accept`.
    fn take_with<T>(
        &mut self,
        aliases: &[&str],
        accept: impl Fn(&Scalar) -> Option<T>,
    ) -> Option<T> {
        for a in aliases {
            let Some(original) = self.by_norm.get(*a).cloned() else {
                continue;
            };
            let Some(value) = self.values.get(&original) else {
                continue;
            };
            if let Some(v) = accept(value) {
                self.take(&original);
                return Some(v);
            }
        }
        None
    }
}

fn parse_time(v: &Scalar) -> Option<EventTime> {
    match v {
        Scalar::Number(n, _) => time::parse_epoch(*n),
        Scalar::Text(s) => time::parse_text(s),
        Scalar::Bool(_) => None,
    }
}

fn parse_port(v: &Scalar) -> Option<u16> {
    v.text().trim().parse::<u16>().ok()
}

fn parse_ip(v: &Scalar) -> Option<IpAddr> {
    v.text().trim().parse::<IpAddr>().ok()
}

fn non_empty_text(v: &Scalar) -> Option<String> {
    let t = v.text();
    let t = t.trim();
    (!t.is_empty() && t != "-").then(|| t.to_owned())
}

fn parse_outcome(v: &Scalar) -> Option<Outcome> {
    match v {
        Scalar::Bool(true) => Some(Outcome::Success),
        Scalar::Bool(false) => Some(Outcome::Failure),
        _ => match v.text().trim().to_lowercase().as_str() {
            "success" | "succeeded" | "successful" | "ok" | "allowed" | "allow" | "accepted"
            | "accept" | "permitted" | "pass" | "passed" | "true" => Some(Outcome::Success),
            "failure" | "failed" | "fail" | "error" | "denied" | "deny" | "blocked" | "block"
            | "rejected" | "reject" | "dropped" | "false" | "unauthorized" | "forbidden" => {
                Some(Outcome::Failure)
            }
            _ => None,
        },
    }
}

const KIND_KEYWORDS: &[(EventKind, &[&str])] = &[
    (
        EventKind::Authentication,
        &[
            "login",
            "logon",
            "logoff",
            "logout",
            "auth",
            "password",
            "ssh",
            "sudo",
            "kerberos",
            "ntlm",
            "credential",
            "mfa",
            "2fa",
            "signin",
            "sign-in",
        ],
    ),
    (
        EventKind::Web,
        &[
            "http",
            "url",
            "uri",
            "useragent",
            "user_agent",
            "statuscode",
            "status_code",
            "request",
            "referer",
            "referrer",
        ],
    ),
    (
        EventKind::File,
        &["file", "filename", "filepath", "directory", "folder"],
    ),
    (
        EventKind::Process,
        &[
            "process",
            "pid",
            "command",
            "cmdline",
            "commandline",
            "executable",
            "exe",
        ],
    ),
    (
        EventKind::Network,
        &[
            "connection",
            "flow",
            "protocol",
            "packet",
            "firewall",
            "dns",
            "port",
            "tcp",
            "udp",
            "icmp",
            "bytesin",
            "bytes_in",
        ],
    ),
    (
        EventKind::System,
        &[
            "service", "kernel", "reboot", "shutdown", "systemd", "startup", "boot",
        ],
    ),
];

/// Catégorie : mots-clés de l'action et du message d'abord, puis noms de champs.
fn infer_kind(action: Option<&str>, message: Option<&str>, keys: &[String]) -> EventKind {
    let text = format!("{} {}", action.unwrap_or(""), message.unwrap_or("")).to_lowercase();
    for (kind, words) in KIND_KEYWORDS {
        if words.iter().any(|w| text.contains(w)) {
            return *kind;
        }
    }
    kind_from_keys(keys)
}

/// Catégorie déduite des seuls noms de champs ; mémorisée par jeu de noms
/// (identique d'une ligne à l'autre dans un même fichier).
fn kind_from_keys(keys: &[String]) -> EventKind {
    use std::hash::{Hash, Hasher};
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<u64, EventKind>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    keys.hash(&mut h);
    let sig = h.finish();
    if let Some(k) = CACHE.with(|c| c.borrow().get(&sig).copied()) {
        return k;
    }
    let joined = keys
        .iter()
        .map(|k| k.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    let kind = KIND_KEYWORDS
        .iter()
        .find(|(_, words)| words.iter().any(|w| joined.contains(w)))
        .map_or(EventKind::Other, |(k, _)| *k);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 1024 {
            c.clear();
        }
        c.insert(sig, kind);
    });
    kind
}

fn clip_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

/// Indications propres à un fichier (ex. colonne d'horodatage choisie pour un CSV).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hints {
    /// Champ à essayer en premier pour l'horodatage.
    pub time_field: Option<String>,
}

/// Construit un événement à partir des champs d'un enregistrement.
pub fn map_fields(source: SourceRef, raw: &str, fields: Vec<(String, Scalar)>) -> NewEvent {
    map_fields_with(source, raw, fields, &Hints::default())
}

/// Comme `map_fields`, avec des indications propres au fichier.
pub fn map_fields_with(
    source: SourceRef,
    raw: &str,
    fields: Vec<(String, Scalar)>,
    hints: &Hints,
) -> NewEvent {
    let keys: Vec<String> = fields.iter().map(|(k, _)| k.clone()).collect();
    let mut f = Fields {
        by_norm: BTreeMap::new(),
        values: BTreeMap::new(),
    };
    for (k, v) in fields {
        f.by_norm.entry(norm(&k)).or_insert_with(|| k.clone());
        f.values.insert(k, v);
    }

    let hinted = hints.time_field.as_deref().and_then(|k| {
        let t = parse_time(f.values.get(k)?)?;
        f.take(k);
        Some(t)
    });
    let time = hinted.or_else(|| f.take_with(TIME, parse_time));
    let host = f.take_with(HOST, non_empty_text);
    let source_ip = f.take_with(SRC_IP, parse_ip);
    let destination_ip = f.take_with(DST_IP, parse_ip);
    let source_port = f.take_with(SRC_PORT, parse_port);
    let destination_port = f.take_with(DST_PORT, parse_port);
    let user = f.take_with(USER, non_empty_text);
    let action = f.take_with(ACTION, non_empty_text);
    let outcome = f.take_with(OUTCOME, parse_outcome);
    let message = f.find(MESSAGE).and_then(|(k, v)| {
        let text = non_empty_text(&v)?;
        f.take(&k);
        Some(text)
    });

    let kind = infer_kind(action.as_deref(), message.as_deref(), &keys);
    let message = message.unwrap_or_else(|| {
        // Pas de message dans la source : résumé composé uniquement de valeurs présentes.
        let parts: Vec<String> = [
            action.clone(),
            user.as_ref().map(|u| format!("utilisateur {u}")),
            source_ip.map(|ip| format!("depuis {ip}")),
            host.as_ref().map(|h| format!("sur {h}")),
        ]
        .into_iter()
        .flatten()
        .collect();
        if parts.is_empty() {
            clip_chars(raw.trim(), 200)
        } else {
            parts.join(" · ")
        }
    });

    let mut event = NewEvent::new(source, kind, message, raw);
    event.time = time;
    event.host = host;
    event.source_ip = source_ip;
    event.destination_ip = destination_ip;
    event.source_port = source_port;
    event.destination_port = destination_port;
    event.user = user;
    event.action = action;
    event.outcome = outcome;

    let mut omitted = 0usize;
    for (k, v) in f.values {
        let value = v.text();
        if value.trim().is_empty() {
            continue;
        }
        if event.attributes.len() >= MAX_ATTRIBUTES - 1 {
            omitted += 1;
            continue;
        }
        let key = clip_chars(&k, MAX_ATTRIBUTE_KEY_CHARS);
        event
            .attributes
            .insert(key, clip_chars(&value, MAX_ATTRIBUTE_VALUE_CHARS));
    }
    if omitted > 0 {
        event
            .attributes
            .insert("evidentyx.attributs_omis".into(), omitted.to_string());
    }
    event
}

#[cfg(test)]
mod tests {
    use super::*;
    use evidentyx_core::{EvidenceId, TimeBasis};

    fn src() -> SourceRef {
        SourceRef {
            evidence_id: EvidenceId::new(1),
            byte_offset: 0,
            line: Some(1),
            record: None,
        }
    }

    fn t(s: &str) -> Scalar {
        Scalar::Text(s.into())
    }

    #[test]
    fn champs_reconnus_quelle_que_soit_la_graphie() {
        let e = map_fields(
            src(),
            "raw",
            vec![
                ("timestamp_utc".into(), t("2026-11-16T00:00:21Z")),
                ("Source-IP".into(), t("198.51.100.40")),
                (
                    "destination.port".into(),
                    Scalar::Number(443.0, "443".into()),
                ),
                ("userName".into(), t("jdupont")),
                ("action".into(), t("login")),
                ("status".into(), t("blocked")),
                ("message".into(), t("Échec d'authentification")),
                ("rule_id".into(), t("R-12")),
            ],
        );
        assert_eq!(e.time.unwrap().basis, TimeBasis::Explicit);
        assert_eq!(e.source_ip.unwrap().to_string(), "198.51.100.40");
        assert_eq!(e.destination_port, Some(443));
        assert_eq!(e.user.as_deref(), Some("jdupont"));
        assert_eq!(e.outcome, Some(Outcome::Failure));
        assert_eq!(e.kind, EventKind::Authentication);
        assert_eq!(e.message, "Échec d'authentification");
        assert_eq!(
            e.attributes.get("rule_id").map(String::as_str),
            Some("R-12")
        );
        assert!(
            !e.attributes.contains_key("userName"),
            "champ normalisé retiré des attributs"
        );
        assert!(e.validate().is_ok());
    }

    #[test]
    fn valeur_inexploitable_conservee_en_attribut() {
        let e = map_fields(
            src(),
            "raw",
            vec![
                ("src_ip".into(), t("pas-une-ip")),
                ("status".into(), t("en attente")),
            ],
        );
        assert_eq!(e.source_ip, None);
        assert_eq!(
            e.attributes.get("src_ip").map(String::as_str),
            Some("pas-une-ip")
        );
        assert_eq!(e.outcome, None);
        assert_eq!(
            e.attributes.get("status").map(String::as_str),
            Some("en attente")
        );
    }

    #[test]
    fn message_compose_sans_rien_inventer() {
        let e = map_fields(
            src(),
            "{\"a\":1}",
            vec![
                ("action".into(), t("file_access")),
                ("user".into(), t("deploy")),
            ],
        );
        assert_eq!(e.message, "file_access · utilisateur deploy");
        assert_eq!(e.kind, EventKind::File);
        let e = map_fields(src(), "  brut  ", vec![("x".into(), t("1"))]);
        assert_eq!(e.message, "brut");
        assert_eq!(e.kind, EventKind::Other);
    }

    #[test]
    fn tiret_et_vide_traites_comme_absents() {
        let e = map_fields(
            src(),
            "raw",
            vec![("user".into(), t("-")), ("host".into(), t("  "))],
        );
        assert_eq!(e.user, None);
        assert_eq!(e.host, None);
    }
}
