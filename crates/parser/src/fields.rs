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

/// Nom comparable : minuscules, sans `_ - . ` ni espaces.
pub(crate) fn norm(key: &str) -> String {
    key.chars()
        .filter(|c| !matches!(c, '_' | '-' | '.' | ' '))
        .flat_map(char::to_lowercase)
        .collect()
}

/// Champ normalisé qu'un nom de champ peut remplir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Time,
    Host,
    SrcIp,
    DstIp,
    SrcPort,
    DstPort,
    User,
    Action,
    Outcome,
    Message,
}

const ROLES: [(Role, &[&str]); 10] = [
    (Role::Time, TIME),
    (Role::Host, HOST),
    (Role::SrcIp, SRC_IP),
    (Role::DstIp, DST_IP),
    (Role::SrcPort, SRC_PORT),
    (Role::DstPort, DST_PORT),
    (Role::User, USER),
    (Role::Action, ACTION),
    (Role::Outcome, OUTCOME),
    (Role::Message, MESSAGE),
];

/// Ce qu'un nom de champ indique : rôle (avec priorité d'alias) et catégorie
/// suggérée par le nom (rang dans `KIND_KEYWORDS`).
#[derive(Debug, Clone, Copy)]
struct KeyInfo {
    role: Option<(Role, usize)>,
    kind_rank: Option<usize>,
}

/// Les mêmes noms reviennent à chaque enregistrement d'un fichier : leur analyse
/// est mémorisée (cache borné, par fil d'exécution).
fn key_info(key: &str) -> KeyInfo {
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<String, KeyInfo>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    if let Some(info) = CACHE.with(|c| c.borrow().get(key).copied()) {
        return info;
    }
    let n = norm(key);
    let lower = key.to_lowercase();
    let info = KeyInfo {
        role: ROLES
            .iter()
            .find_map(|(role, aliases)| Some((*role, aliases.iter().position(|a| *a == n)?))),
        kind_rank: KIND_KEYWORDS
            .iter()
            .position(|(_, words)| words.iter().any(|w| lower.contains(w))),
    };
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() >= 4096 {
            c.clear();
        }
        c.insert(key.to_owned(), info);
    });
    info
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
fn infer_kind(action: Option<&str>, message: Option<&str>, key_rank: Option<usize>) -> EventKind {
    let text = format!("{} {}", action.unwrap_or(""), message.unwrap_or("")).to_lowercase();
    KIND_KEYWORDS
        .iter()
        .find(|(_, words)| words.iter().any(|w| text.contains(w)))
        .or_else(|| KIND_KEYWORDS.get(key_rank?))
        .map_or(EventKind::Other, |(k, _)| *k)
}

/// Texte limité à `max` caractères, copié seulement s'il faut couper.
fn clip_owned(s: String, max: usize) -> String {
    if s.len() <= max {
        s // octets ≤ max ⇒ caractères ≤ max
    } else {
        clip_chars(&s, max)
    }
}

pub(crate) fn clip_chars(s: &str, max: usize) -> String {
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
    // Une passe : rôle de chaque champ (mémorisé par nom), puis, pour chaque
    // rôle, le premier alias par priorité dont la valeur est exploitable.
    let infos: Vec<KeyInfo> = fields.iter().map(|(k, _)| key_info(k)).collect();
    let key_rank = infos.iter().filter_map(|i| i.kind_rank).min();
    let mut slots: Vec<Option<(String, Scalar)>> = fields.into_iter().map(Some).collect();

    fn take<T>(
        slots: &mut [Option<(String, Scalar)>],
        infos: &[KeyInfo],
        role: Role,
        accept: impl Fn(&Scalar) -> Option<T>,
    ) -> Option<T> {
        let mut candidates: Vec<(usize, usize)> = infos
            .iter()
            .enumerate()
            .filter_map(|(i, info)| match info.role {
                Some((r, priority)) if r == role && slots[i].is_some() => Some((priority, i)),
                _ => None,
            })
            .collect();
        candidates.sort_unstable();
        candidates.into_iter().find_map(|(_, i)| {
            let value = accept(&slots[i].as_ref()?.1)?;
            slots[i] = None;
            Some(value)
        })
    }

    let hinted = hints.time_field.as_deref().and_then(|k| {
        let i = slots
            .iter()
            .position(|s| s.as_ref().is_some_and(|(n, _)| n == k))?;
        let t = parse_time(&slots[i].as_ref()?.1)?;
        slots[i] = None;
        Some(t)
    });
    let f = &mut slots;
    let time = hinted.or_else(|| take(f, &infos, Role::Time, parse_time));
    let host = take(f, &infos, Role::Host, non_empty_text);
    let source_ip = take(f, &infos, Role::SrcIp, parse_ip);
    let destination_ip = take(f, &infos, Role::DstIp, parse_ip);
    let source_port = take(f, &infos, Role::SrcPort, parse_port);
    let destination_port = take(f, &infos, Role::DstPort, parse_port);
    let user = take(f, &infos, Role::User, non_empty_text);
    let action = take(f, &infos, Role::Action, non_empty_text);
    let outcome = take(f, &infos, Role::Outcome, parse_outcome);
    let message = take(f, &infos, Role::Message, non_empty_text);

    let kind = infer_kind(action.as_deref(), message.as_deref(), key_rank);
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
    for (k, v) in slots.into_iter().flatten() {
        let value = match v {
            Scalar::Text(t) | Scalar::Number(_, t) => t,
            Scalar::Bool(b) => b.to_string(),
        };
        if value.trim().is_empty() {
            continue;
        }
        if event.attributes.len() >= MAX_ATTRIBUTES - 1 {
            omitted += 1;
            continue;
        }
        event.attributes.insert(
            clip_owned(k, MAX_ATTRIBUTE_KEY_CHARS),
            clip_owned(value, MAX_ATTRIBUTE_VALUE_CHARS),
        );
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
