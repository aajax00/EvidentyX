//! Syslog : RFC 5424, RFC 3164 (BSD) et format fichier de rsyslog (heure ISO 8601).
//!
//! - RFC 5424 : `<34>1 2003-10-11T22:14:15.003Z hôte app pid msgid [données] message`
//! - RFC 3164 : `<34>Oct 11 22:14:15 hôte app[pid]: message` (`<PRI>` facultatif)
//! - rsyslog  : `2026-11-16T00:00:21.123+01:00 hôte app[pid]: message`
//!
//! La RFC 3164 ne donne ni année ni fuseau. L'année est déduite de la date du
//! fichier : la plus récente qui ne place pas l'événement après elle (un jour de
//! tolérance). La base de l'heure le signale (`InferredYear`). Sans date de
//! fichier, l'événement reste sans heure : rien n'est inventé.
//!
//! Utilisateur, adresse et issue sont repris des messages de sshd, sudo et du
//! pare-feu du noyau, uniquement quand ils y figurent mot pour mot.

use crate::fields::clip_chars;
use crate::json::{read_line_bounded, trim_ascii};
use crate::time::{days_from_civil, parse_iso, year_of_days};
use crate::{Format, ParseContext, ParseError, ParseReport, MAX_RECORD_BYTES};
use evidentyx_core::event::{MAX_ATTRIBUTES, MAX_ATTRIBUTE_KEY_CHARS, MAX_ATTRIBUTE_VALUE_CHARS};
use evidentyx_core::{
    EventKind, EventTime, NewEvent, Outcome, SourceRef, TimeBasis, TimePrecision, Timestamp,
};
use std::io::{BufReader, Read};
use std::sync::atomic::Ordering;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const SEVERITIES: [&str; 8] = [
    "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
];
const FACILITIES: [&str; 24] = [
    "kern", "user", "mail", "daemon", "auth", "syslog", "lpr", "news", "uucp", "cron", "authpriv",
    "ftp", "ntp", "audit", "alert", "clock", "local0", "local1", "local2", "local3", "local4",
    "local5", "local6", "local7",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    Rfc5424,
    Rfc3164,
    Iso,
}

/// Heure telle qu'écrite dans la ligne.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Stamp {
    Full(EventTime),
    /// Mois, jour et secondes dans la journée, sans année (RFC 3164).
    NoYear {
        month: u32,
        day: u32,
        seconds: i64,
    },
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Line<'a> {
    variant: Variant,
    pri: Option<u8>,
    stamp: Stamp,
    host: Option<&'a str>,
    app: Option<&'a str>,
    pid: Option<&'a str>,
    msgid: Option<&'a str>,
    /// Données structurées : (`identifiant.clé`, valeur).
    data: Vec<(String, String)>,
    message: &'a str,
}

/// `<PRI>` de 0 à 191 ; renvoie la valeur et le reste.
fn take_pri(s: &str) -> Option<(u8, &str)> {
    let rest = s.strip_prefix('<')?;
    let (num, rest) = rest.split_once('>')?;
    if num.is_empty() || num.len() > 3 || !num.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let pri: u8 = num.parse().ok()?;
    (pri <= 191).then_some((pri, rest))
}

fn nil(s: &str) -> Option<&str> {
    (s != "-" && !s.is_empty()).then_some(s)
}

/// Données structurées RFC 5424 ; renvoie les paires et le reste.
fn take_data(s: &str) -> Option<(Vec<(String, String)>, &str)> {
    if let Some(rest) = s.strip_prefix('-') {
        return Some((Vec::new(), rest));
    }
    let mut out = Vec::new();
    let mut rest = s;
    if !rest.starts_with('[') {
        return None;
    }
    while let Some(body) = rest.strip_prefix('[') {
        let end = body.find([' ', ']'])?;
        let id = &body[..end];
        if id.is_empty() {
            return None;
        }
        let mut cur = &body[end..];
        loop {
            if let Some(after) = cur.strip_prefix(']') {
                rest = after;
                break;
            }
            let param = cur.strip_prefix(' ')?;
            let (key, value) = param.split_once("=\"")?;
            if key.is_empty() || key.contains([' ', ']', '"']) {
                return None;
            }
            // Échappements \" \\ \] ; toute autre barre oblique est littérale.
            let mut text = String::new();
            let mut chars = value.char_indices().peekable();
            let mut close = None;
            while let Some((i, c)) = chars.next() {
                match c {
                    '"' => {
                        close = Some(i);
                        break;
                    }
                    '\\' => match chars.peek() {
                        Some(&(_, n @ ('"' | '\\' | ']'))) => {
                            text.push(n);
                            chars.next();
                        }
                        _ => text.push('\\'),
                    },
                    _ => text.push(c),
                }
            }
            out.push((format!("{id}.{key}"), text));
            cur = &value[close? + 1..];
        }
    }
    Some((out, rest))
}

fn parse_5424(s: &str) -> Option<Line<'_>> {
    let (pri, rest) = take_pri(s)?;
    let rest = rest.strip_prefix("1 ")?;
    let mut parts = rest.splitn(6, ' ');
    let mut next = || parts.next();
    let (time, host, app, pid, msgid) = (next()?, next()?, next()?, next()?, next()?);
    let (data, rest) = take_data(next()?)?;
    let message = match rest.strip_prefix(' ') {
        Some(m) => m,
        None if rest.is_empty() => "",
        None => return None,
    };
    let stamp = match nil(time) {
        None => Stamp::Absent,
        Some(t) => Stamp::Full(parse_iso(t)?),
    };
    Some(Line {
        variant: Variant::Rfc5424,
        pri: Some(pri),
        stamp,
        host: nil(host),
        app: nil(app),
        pid: nil(pid),
        msgid: nil(msgid),
        data,
        message: message.trim_start_matches('\u{feff}').trim(),
    })
}

/// `app[pid]: message` ; sans étiquette reconnaissable, tout est message.
fn split_tag(s: &str) -> (Option<&str>, Option<&str>, &str) {
    let s = s.trim_start();
    let end = s
        .find(|c: char| c.is_whitespace() || c == ':' || c == '[')
        .unwrap_or(s.len());
    let (app, rest) = s.split_at(end);
    if app.is_empty() {
        return (None, None, s);
    }
    if let Some(inner) = rest.strip_prefix('[') {
        if let Some((pid, after)) = inner.split_once(']') {
            if let Some(msg) = after.strip_prefix(':') {
                return (Some(app), nil(pid), msg.trim());
            }
        }
        return (None, None, s);
    }
    match rest.strip_prefix(':') {
        Some(msg) => (Some(app), None, msg.trim()),
        None => (None, None, s),
    }
}

fn two_digits(s: &str) -> Option<u32> {
    (s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit()))
        .then(|| s.parse().ok())
        .flatten()
}

/// `Oct 11 22:14:15 ` ou `Oct  1 …` ; renvoie l'heure et le reste.
fn take_bsd_time(s: &str) -> Option<(Stamp, &str)> {
    let month = MONTHS.iter().position(|m| s.starts_with(m))? as u32 + 1;
    let after = s.get(3..)?.strip_prefix(' ')?;
    let after = after.strip_prefix(' ').unwrap_or(after);
    let (day, after) = after.split_once(' ')?;
    if day.is_empty() || day.len() > 2 || !day.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let day: u32 = day.parse().ok()?;
    let (clock, rest) = after.split_once(' ')?;
    let mut p = clock.split(':');
    let (h, m, sec) = (
        two_digits(p.next()?)?,
        two_digits(p.next()?)?,
        two_digits(p.next()?)?,
    );
    if p.next().is_some() || h > 23 || m > 59 || sec > 60 || !(1..=31).contains(&day) {
        return None;
    }
    let seconds = i64::from(h * 3600 + m * 60 + sec.min(59));
    Some((
        Stamp::NoYear {
            month,
            day,
            seconds,
        },
        rest,
    ))
}

fn parse_3164(s: &str) -> Option<Line<'_>> {
    let (pri, body) = match take_pri(s) {
        Some((p, r)) => (Some(p), r),
        None => (None, s),
    };
    let (stamp, rest) = take_bsd_time(body)?;
    let (host, rest) = rest.split_once(' ')?;
    let (app, pid, message) = split_tag(rest);
    Some(Line {
        variant: Variant::Rfc3164,
        pri,
        stamp,
        host: nil(host),
        app,
        pid,
        msgid: None,
        data: Vec::new(),
        message,
    })
}

/// Format fichier de rsyslog. Une étiquette `app:` est exigée : sans elle,
/// n'importe quel journal daté serait pris pour du Syslog.
fn parse_iso_line(s: &str) -> Option<Line<'_>> {
    let (pri, body) = match take_pri(s) {
        Some((p, r)) => (Some(p), r),
        None => (None, s),
    };
    let (time, rest) = body.split_once(' ')?;
    if !time.contains('T') {
        return None;
    }
    let stamp = Stamp::Full(parse_iso(time)?);
    let (host, rest) = rest.split_once(' ')?;
    if host.is_empty() || host.ends_with(':') {
        return None;
    }
    let (app, pid, message) = split_tag(rest);
    app?;
    Some(Line {
        variant: Variant::Iso,
        pri,
        stamp,
        host: nil(host),
        app,
        pid,
        msgid: None,
        data: Vec::new(),
        message,
    })
}

fn parse_line(s: &str) -> Option<Line<'_>> {
    parse_5424(s)
        .or_else(|| parse_3164(s))
        .or_else(|| parse_iso_line(s))
}

/// Au moins 80 % des premières lignes complètes sont du Syslog.
pub fn detect(head: &[u8]) -> Option<Format> {
    let text = String::from_utf8_lossy(head);
    let mut lines: Vec<&str> = text.lines().take(50).collect();
    if lines.len() > 1 && lines.len() < 50 && !text.ends_with('\n') {
        lines.pop(); // dernière ligne possiblement tronquée
    }
    let lines: Vec<&str> = lines
        .into_iter()
        .map(|l| l.trim_start_matches('\u{feff}').trim())
        .filter(|l| !l.is_empty())
        .collect();
    let ok = lines.iter().filter(|l| parse_line(l).is_some()).count();
    (ok > 0 && ok * 5 >= lines.len() * 4).then_some(Format::Syslog)
}

/// Année la plus récente qui ne place pas l'événement plus d'un jour après la
/// date de référence.
fn infer_year(month: u32, day: u32, seconds: i64, reference: Timestamp) -> Option<EventTime> {
    let ref_secs = reference.unix_millis().div_euclid(1000);
    let ref_year = year_of_days(ref_secs.div_euclid(86_400));
    // 9 ans : couvre l'écart maximal entre deux 29 février.
    (ref_year - 8..=ref_year + 1).rev().find_map(|year| {
        let t = parse_iso(&format!("{year:04}-{month:02}-{day:02}"))?;
        let secs = t.utc_micros / 1_000_000 + seconds;
        debug_assert_eq!(
            t.utc_micros / 86_400_000_000,
            days_from_civil(year, month, day)
        );
        (secs <= ref_secs + 86_400).then_some(EventTime {
            utc_micros: secs * 1_000_000,
            precision: TimePrecision::Second,
            basis: TimeBasis::InferredYear,
        })
    })
}

/// Mot qui suit `marker` dans le message.
fn word_after<'a>(message: &'a str, marker: &str) -> Option<&'a str> {
    let rest = &message[message.find(marker)? + marker.len()..];
    let word = rest.split_whitespace().next()?;
    (!word.is_empty()).then_some(word)
}

/// `CLÉ=valeur` d'un message du pare-feu.
fn kv<'a>(message: &'a str, key: &str) -> Option<&'a str> {
    message
        .split_whitespace()
        .find_map(|w| w.strip_prefix(key)?.strip_prefix('='))
        .filter(|v| !v.is_empty())
}

/// Reprend du message les champs qui y figurent explicitement.
fn enrich(e: &mut NewEvent, app: &str, message: &str) {
    let base = app.rsplit('/').next().unwrap_or(app);
    if base == "sshd" || base == "sshd-session" {
        let outcome = if message.starts_with("Accepted ") {
            Some(Outcome::Success)
        } else if message.starts_with("Failed ") || message.starts_with("Invalid user ") {
            Some(Outcome::Failure)
        } else {
            None
        };
        if outcome.is_some() {
            e.outcome = outcome;
            // « for invalid user X from … », « for X from … », « Invalid user X from … ».
            // Le nom vient de l'attaquant : on lit entre les repères, en partant de la
            // fin, pour qu'un nom contenant « from » ne décale pas l'adresse.
            if let Some(at) = message.rfind(" from ") {
                let (before, after) = (&message[..at], &message[at + 6..]);
                let user = before
                    .split_once(" for invalid user ")
                    .or_else(|| before.split_once(" for "))
                    .map(|(_, u)| u)
                    .or_else(|| before.strip_prefix("Invalid user "));
                e.user = user
                    .map(str::trim)
                    .filter(|u| !u.is_empty())
                    .map(str::to_owned);
                e.source_ip = after.split_whitespace().next().and_then(|w| w.parse().ok());
                e.source_port = word_after(after, "port ").and_then(|w| w.parse().ok());
            }
        }
    } else if base == "sudo" {
        if let Some((user, rest)) = message.split_once(" : ") {
            if rest.contains("COMMAND=") && !user.trim().is_empty() {
                e.user = Some(user.trim().to_owned());
                let refused = [
                    "NOT in sudoers",
                    "command not allowed",
                    "incorrect password",
                ];
                if refused.iter().any(|r| rest.contains(r)) {
                    e.outcome = Some(Outcome::Failure);
                }
            }
        }
    } else if base == "kernel" && kv(message, "SRC").is_some() && kv(message, "DST").is_some() {
        e.kind = EventKind::Network;
        e.source_ip = kv(message, "SRC").and_then(|v| v.parse().ok());
        e.destination_ip = kv(message, "DST").and_then(|v| v.parse().ok());
        e.source_port = kv(message, "SPT").and_then(|v| v.parse().ok());
        e.destination_port = kv(message, "DPT").and_then(|v| v.parse().ok());
        if message.contains("[UFW BLOCK]") {
            e.outcome = Some(Outcome::Failure);
        } else if message.contains("[UFW ALLOW]") {
            e.outcome = Some(Outcome::Success);
        }
    }
}

/// Catégorie d'après l'application, puis le service Syslog (facility).
fn kind(app: Option<&str>, pri: Option<u8>) -> EventKind {
    let base = app.map(|a| a.rsplit('/').next().unwrap_or(a).to_lowercase());
    match base.as_deref() {
        Some(
            "sshd" | "sshd-session" | "sudo" | "su" | "login" | "systemd-logind" | "passwd"
            | "polkitd" | "gdm-password",
        ) => return EventKind::Authentication,
        Some("nginx" | "apache2" | "apache" | "httpd") => return EventKind::Web,
        Some(
            "named" | "dnsmasq" | "dhclient" | "dhcpd" | "networkmanager" | "wpa_supplicant"
            | "firewalld",
        ) => return EventKind::Network,
        Some("kernel" | "systemd" | "cron" | "crond" | "anacron") => return EventKind::System,
        _ => {}
    }
    match pri.map(|p| p >> 3) {
        Some(4 | 10) => EventKind::Authentication,
        Some(0 | 3 | 5 | 9) => EventKind::System,
        _ => EventKind::Other,
    }
}

fn to_event(
    line: &Line<'_>,
    raw: &str,
    source: SourceRef,
    reference: Option<Timestamp>,
) -> NewEvent {
    let message = if line.message.is_empty() {
        clip_chars(raw, 200) // pas de message dans la source : début de la ligne
    } else {
        line.message.to_owned()
    };
    let mut e = NewEvent::new(source, kind(line.app, line.pri), message, raw);
    e.time = match line.stamp {
        Stamp::Full(t) => Some(t),
        Stamp::NoYear {
            month,
            day,
            seconds,
        } => reference.and_then(|r| infer_year(month, day, seconds, r)),
        Stamp::Absent => None,
    };
    e.host = line.host.map(str::to_owned);
    if let Some(app) = line.app {
        enrich(&mut e, app, line.message);
    }

    let mut put = |k: &str, v: &str| {
        if e.attributes.len() < MAX_ATTRIBUTES {
            e.attributes.insert(
                clip_chars(k, MAX_ATTRIBUTE_KEY_CHARS),
                clip_chars(v, MAX_ATTRIBUTE_VALUE_CHARS),
            );
        }
    };
    for (k, v) in [("app", line.app), ("pid", line.pid), ("msgid", line.msgid)] {
        if let Some(v) = v {
            put(k, v);
        }
    }
    if let Some(p) = line.pri {
        put("facility", FACILITIES[usize::from(p >> 3)]);
        put("severity", SEVERITIES[usize::from(p & 7)]);
    }
    for (k, v) in &line.data {
        put(k, v);
    }
    e
}

pub fn parse(
    reader: impl Read,
    ctx: ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<ParseReport, ParseError> {
    let mut report = ParseReport::new(Format::Syslog);
    let mut reader = BufReader::with_capacity(1 << 16, reader);
    let (mut offset, mut line_no) = (0u64, 0u64);
    let (mut n5424, mut n3164, mut niso) = (0u64, 0u64, 0u64);
    let mut buf = Vec::new();
    loop {
        if ctx.cancel.load(Ordering::Relaxed) {
            return Err(ParseError::Cancelled);
        }
        buf.clear();
        let n = read_line_bounded(&mut reader, &mut buf)?;
        if n == 0 {
            break;
        }
        line_no += 1;
        let start = offset;
        offset += n as u64;
        report.bytes = offset;
        if line_no.is_multiple_of(1024) {
            (ctx.on_progress)(offset);
        }
        if buf.len() > MAX_RECORD_BYTES {
            report.records += 1;
            report.error(
                line_no,
                start,
                format!("ligne trop longue (plus de {MAX_RECORD_BYTES} octets)"),
            );
            continue;
        }
        let bytes = trim_ascii(&buf);
        let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
        if bytes.is_empty() {
            continue;
        }
        report.records += 1;
        let text = String::from_utf8_lossy(bytes);
        let Some(line) = parse_line(&text) else {
            report.error(line_no, start, "ligne Syslog non reconnue");
            continue;
        };
        let source = SourceRef {
            evidence_id: ctx.evidence_id,
            byte_offset: start,
            line: Some(line_no),
            record: Some(report.records),
        };
        let event = to_event(&line, &text, source, ctx.reference_time);
        if let Err(e) = event.validate() {
            report.error(line_no, start, e.to_string());
            continue;
        }
        sink(event).map_err(ParseError::Sink)?;
        report.events += 1;
        match line.variant {
            Variant::Rfc5424 => n5424 += 1,
            Variant::Rfc3164 => n3164 += 1,
            Variant::Iso => niso += 1,
        }
    }
    let mut parts = Vec::new();
    if n5424 > 0 {
        parts.push("RFC 5424");
    }
    if niso > 0 {
        parts.push("heure ISO");
    }
    if n3164 > 0 {
        parts.push(if ctx.reference_time.is_some() {
            "RFC 3164 (année déduite de la date du fichier)"
        } else {
            "RFC 3164 (année inconnue : sans heure)"
        });
    }
    report.layout = (!parts.is_empty()).then(|| parts.join(" + "));
    (ctx.on_progress)(offset);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detect as detect_any;
    use evidentyx_core::EvidenceId;
    use std::sync::atomic::AtomicBool;

    fn at(iso: &str) -> Timestamp {
        Timestamp::from_unix_millis(parse_iso(iso).unwrap().utc_micros / 1000)
    }

    fn run(input: &[u8], reference: Option<Timestamp>) -> (ParseReport, Vec<NewEvent>) {
        let cancel = AtomicBool::new(false);
        let mut events = Vec::new();
        let r = crate::parse(
            Format::Syslog,
            input,
            ParseContext {
                evidence_id: EvidenceId::new(1),
                reference_time: reference,
                on_progress: &mut |_| {},
                cancel: &cancel,
            },
            &mut |e| {
                events.push(e);
                Ok(())
            },
        )
        .unwrap();
        (r, events)
    }

    fn micros(iso: &str) -> i64 {
        parse_iso(iso).unwrap().utc_micros
    }

    #[test]
    fn exemples_de_la_rfc_5424() {
        let input = "<34>1 2003-10-11T22:14:15.003Z mymachine.example.com su - ID47 - \u{feff}'su root' failed for lonvick on /dev/pts/8\n\
<165>1 2003-08-24T05:14:15.000003-07:00 192.0.2.1 myproc 8710 - - %% It's time to make the do-nuts.\n\
<165>1 2003-10-11T22:14:15.003Z mymachine.example.com evntslog - ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Appli\\\"ca]t\"][examplePriority@32473 class=\"high\"]\n";
        // Un « ] » non échappé dans une valeur est toléré : la valeur se termine au guillemet.
        let (r, ev) = run(input.as_bytes(), None);
        assert_eq!((r.records, r.events, r.errors), (3, 3, 0));
        assert_eq!(r.layout.as_deref(), Some("RFC 5424"));
        assert_eq!(ev[0].message, "'su root' failed for lonvick on /dev/pts/8");
        assert_eq!(ev[0].kind, EventKind::Authentication);
        assert_eq!(ev[0].host.as_deref(), Some("mymachine.example.com"));
        assert_eq!(ev[0].time.unwrap().basis, TimeBasis::Explicit);
        assert_eq!(ev[0].attributes["facility"], "auth");
        assert_eq!(ev[0].attributes["severity"], "crit");
        assert_eq!(ev[0].attributes["msgid"], "ID47");
        assert!(!ev[0].attributes.contains_key("pid"), "« - » = absent");
        assert_eq!(
            ev[1].time.unwrap().utc_micros,
            micros("2003-08-24T12:14:15.000003Z")
        );
        assert_eq!(ev[1].attributes["pid"], "8710");
        assert_eq!(
            ev[2].attributes["exampleSDID@32473.eventSource"],
            "Appli\"ca]t"
        );
        assert_eq!(ev[2].attributes["examplePriority@32473.class"], "high");
        assert!(!ev[2].message.is_empty());
    }

    #[test]
    fn heure_absente_en_5424() {
        let (_, ev) = run(b"<13>1 - - - - - - bonjour\n", None);
        assert!(ev[0].time.is_none());
        assert_eq!(ev[0].host, None);
        assert_eq!(ev[0].message, "bonjour");
    }

    #[test]
    fn rfc_3164_sshd_sudo_et_decalages() {
        let input = "<38>Nov 16 10:00:00 srv-01 sshd[42]: Failed password for invalid user admin from 203.0.113.7 port 51234 ssh2\r\n\
n'importe quoi\r\n\
Nov  6 10:00:01 srv-01 sshd[43]: Accepted publickey for alice from 2001:db8::9 port 22000 ssh2: RSA\r\n\
Nov 16 10:00:02 srv-01 sudo: alice : TTY=pts/0 ; PWD=/ ; USER=root ; COMMAND=/bin/ls\r\n\
Nov 16 10:00:03 srv-01 last message repeated 2 times\r\n";
        let (r, ev) = run(input.as_bytes(), Some(at("2026-11-20T00:00:00Z")));
        assert_eq!((r.records, r.events, r.errors), (5, 4, 1));
        assert_eq!(r.first_errors[0].line, 2);
        let a = &ev[0];
        assert_eq!(a.user.as_deref(), Some("admin"));
        assert_eq!(a.source_ip.unwrap().to_string(), "203.0.113.7");
        assert_eq!(a.source_port, Some(51234));
        assert_eq!(a.outcome, Some(Outcome::Failure));
        assert_eq!(a.attributes["app"], "sshd");
        assert_eq!(a.attributes["pid"], "42");
        let t = a.time.unwrap();
        assert_eq!(
            (t.utc_micros, t.basis),
            (micros("2026-11-16T10:00:00Z"), TimeBasis::InferredYear)
        );
        let b = &ev[1];
        assert_eq!(b.source.line, Some(3));
        assert_eq!(
            &input.as_bytes()[b.source.byte_offset as usize..][..6],
            b"Nov  6"
        );
        assert_eq!(b.outcome, Some(Outcome::Success));
        assert_eq!(b.user.as_deref(), Some("alice"));
        assert_eq!(b.source_ip.unwrap().to_string(), "2001:db8::9");
        assert_eq!(ev[2].user.as_deref(), Some("alice"));
        assert_eq!(
            ev[2].outcome, None,
            "sudo n'indique pas la réussite de la commande"
        );
        assert_eq!(ev[3].message, "last message repeated 2 times");
        assert!(!ev[3].attributes.contains_key("app"));
    }

    #[test]
    fn nom_d_utilisateur_hostile() {
        let line = b"Nov 16 10:00:00 srv sshd[1]: Failed password for invalid user x from 1.2.3.4 port 1 from 203.0.113.7 port 999 ssh2\n";
        let (_, ev) = run(line, None);
        assert_eq!(ev[0].source_ip.unwrap().to_string(), "203.0.113.7");
        assert_eq!(ev[0].source_port, Some(999));
        assert_eq!(ev[0].user.as_deref(), Some("x from 1.2.3.4 port 1"));
    }

    #[test]
    fn annee_deduite() {
        let year = |m, d, reference: &str| infer_year(m, d, 0, at(reference)).map(|t| t.utc_micros);
        // Fichier de janvier contenant des lignes de décembre.
        assert_eq!(
            year(12, 31, "2026-01-02T00:00:00Z"),
            Some(micros("2025-12-31"))
        );
        assert_eq!(
            year(1, 1, "2026-01-02T00:00:00Z"),
            Some(micros("2026-01-01"))
        );
        // Tolérance d'un jour (fuseau) autour du nouvel an.
        assert_eq!(
            year(1, 1, "2025-12-31T23:30:00Z"),
            Some(micros("2026-01-01"))
        );
        // 29 février : dernière année bissextile.
        assert_eq!(
            year(2, 29, "2026-06-01T00:00:00Z"),
            Some(micros("2024-02-29"))
        );
        assert_eq!(year(2, 30, "2026-06-01T00:00:00Z"), None);
    }

    #[test]
    fn sans_date_de_fichier_aucune_heure_inventee() {
        let (r, ev) = run(b"Nov 16 10:00:00 srv cron[1]: (root) CMD (x)\n", None);
        assert!(ev[0].time.is_none());
        assert_eq!(
            r.layout.as_deref(),
            Some("RFC 3164 (année inconnue : sans heure)")
        );
    }

    #[test]
    fn pare_feu_et_format_rsyslog() {
        let input = b"2026-11-16T00:00:21.123456+01:00 srv kernel: [12.3] [UFW BLOCK] IN=eth0 OUT= SRC=198.51.100.4 DST=10.0.0.2 PROTO=TCP SPT=4000 DPT=22\n";
        let (r, ev) = run(input, None);
        assert_eq!(r.layout.as_deref(), Some("heure ISO"));
        let e = &ev[0];
        assert_eq!(e.kind, EventKind::Network);
        assert_eq!(e.destination_port, Some(22));
        assert_eq!(e.source_ip.unwrap().to_string(), "198.51.100.4");
        assert_eq!(e.outcome, Some(Outcome::Failure));
        let t = e.time.unwrap();
        assert_eq!(
            (t.utc_micros, t.precision),
            (
                micros("2026-11-15T23:00:21.123456Z"),
                TimePrecision::Microsecond
            )
        );
    }

    #[test]
    fn lignes_mutees_sans_panique() {
        let seeds = [
            "<34>1 2003-10-11T22:14:15.003Z h a 1 ID [x@1 k=\"v\\\"é\"] msg é",
            "<38>Nov  6 10:00:00 srv sshd[42]: Failed password for invalid user é from 1.2.3.4 port 5 ssh2",
            "2026-11-16T00:00:21.1+01:00 srv kernel: [UFW BLOCK] SRC=1.2.3.4 DST=5.6.7.8 DPT=22",
            "Nov 16 10:00:02 srv sudo: alice : USER=root ; COMMAND=/bin/ls",
        ];
        let extra = [
            "", "é", "[", "]", "\"", "\\", "<", ">", " ", ":", "=", "\u{feff}", "-",
        ];
        let reference = Some(at("2026-11-20T00:00:00Z"));
        let source = SourceRef {
            evidence_id: EvidenceId::new(1),
            byte_offset: 0,
            line: Some(1),
            record: None,
        };
        for seed in seeds {
            let chars: Vec<char> = seed.chars().collect();
            for i in 0..=chars.len() {
                for cut in 0..3 {
                    for x in extra {
                        let mut s: String = chars[..i].iter().collect();
                        s.push_str(x);
                        s.extend(chars.iter().skip(i + cut));
                        if let Some(l) = parse_line(&s) {
                            let e = to_event(&l, &s, source, reference);
                            assert!(e.validate().is_ok(), "{s}");
                        }
                        let _ = detect(s.as_bytes());
                    }
                }
            }
        }
    }

    #[test]
    fn lignes_invalides() {
        for bad in [
            "<192>1 - - - - - - x",
            "<>1 - - - - - - x",
            "<34>1 - - - - - [a b=\"c] x",
            "<34>1 - - - - - [] x",
            "<34>1 hier h a - - - x",
            "Foo 16 10:00:00 srv a: b",
            "Nov 32 10:00:00 srv a: b",
            "Nov 16 25:00:00 srv a: b",
            "Nov 16 10:00",
            "Novembre",
            "2026-11-16T00:00:21Z service démarré sans étiquette",
            "<34",
            "é",
            "",
        ] {
            assert!(parse_line(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn detection() {
        assert_eq!(
            detect_any(
                "auth.log",
                b"Nov 16 10:00:00 srv sshd[42]: Accepted password\n"
            ),
            Some(Format::Syslog)
        );
        // Des virgules dans les messages ne font pas d'un Syslog un CSV.
        assert_eq!(
            detect_any("x", b"Nov 16 10:00:00 s a: b,c\nNov 16 10:00:01 s a: d,e\n"),
            Some(Format::Syslog)
        );
        assert_eq!(
            detect_any("x", b"<13>1 - h a - - - m\n"),
            Some(Format::Syslog)
        );
        assert_eq!(
            detect_any("x.csv", b"timestamp,host\n2026-11-16T00:00:21Z,srv\n"),
            Some(Format::Csv)
        );
        assert_eq!(
            detect_any("x.txt", b"2026-11-16T00:00:21Z bonjour le monde\n"),
            None
        );
    }
}
