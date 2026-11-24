//! Journaux d'accès web : formats `common` et `combined` d'Apache et de Nginx.
//!
//! `IP ident utilisateur [10/Oct/2000:13:55:36 -0700] "GET /a HTTP/1.1" 200 2326 "référent" "agent"`
//!
//! Variantes tolérées : `vhost:port` en tête (`vhost_combined` d'Apache),
//! champs entre guillemets supplémentaires en fin de ligne (Nginx). L'heure
//! porte toujours son fuseau : elle est exacte. Les journaux d'erreurs ne sont
//! pas lus.

use crate::fields::clip_chars;
use crate::json::{read_line_bounded, trim_ascii};
use crate::time::parse_iso;
use crate::{Format, ParseContext, ParseError, ParseReport, MAX_RECORD_BYTES};
use evidentyx_core::event::{MAX_ATTRIBUTE_KEY_CHARS, MAX_ATTRIBUTE_VALUE_CHARS};
use evidentyx_core::{EventKind, EventTime, NewEvent, Outcome, SourceRef};
use std::io::{BufReader, Read};
use std::net::IpAddr;
use std::sync::atomic::Ordering;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    vhost: Option<String>,
    client: String,
    ident: Option<String>,
    user: Option<String>,
    time: EventTime,
    request: String,
    status: Option<u16>,
    bytes: Option<String>,
    referer: Option<String>,
    agent: Option<String>,
    extra: Vec<String>,
}

fn dash(s: &str) -> Option<String> {
    (s != "-" && !s.is_empty()).then(|| s.to_owned())
}

/// Mot suivant (jusqu'à l'espace) ; renvoie le mot et le reste.
fn word(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start_matches(' ');
    let end = s.find(' ').unwrap_or(s.len());
    (end > 0).then(|| (&s[..end], &s[end..]))
}

/// Chaîne entre guillemets avec échappements `\"` et `\\` ; renvoie le texte et le reste.
fn quoted(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start_matches(' ').strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = s.char_indices();
    while let Some((i, c)) = chars.next() {
        match c {
            '"' => return Some((out, &s[i + 1..])),
            '\\' => match chars.next() {
                Some((_, n @ ('"' | '\\'))) => out.push(n),
                Some((_, n)) => {
                    out.push('\\');
                    out.push(n);
                }
                None => out.push('\\'),
            },
            _ => out.push(c),
        }
    }
    None
}

/// `[10/Oct/2000:13:55:36 -0700]` ; renvoie l'heure et le reste.
fn bracket_time(s: &str) -> Option<(EventTime, &str)> {
    let s = s.trim_start_matches(' ').strip_prefix('[')?;
    let (inner, rest) = s.split_once(']')?;
    let (date, zone) = inner.split_once(' ')?;
    let mut p = date.splitn(3, '/');
    let (day, month, rest_date) = (p.next()?, p.next()?, p.next()?);
    let month = MONTHS.iter().position(|m| *m == month)? + 1;
    let (year, clock) = rest_date.split_once(':')?;
    if day.len() != 2 || year.len() != 4 || zone.len() != 5 {
        return None;
    }
    let iso = format!(
        "{year}-{month:02}-{day}T{clock}{}:{}",
        &zone[..3],
        &zone[3..]
    );
    Some((parse_iso(&iso)?, rest))
}

fn parse_line(s: &str) -> Option<Entry> {
    let (first, rest) = word(s)?;
    // `vhost:port IP …` : le premier mot n'est pas l'adresse si le deuxième en est une.
    let (vhost, client, rest) = match word(rest) {
        Some((second, after))
            if second.parse::<IpAddr>().is_ok()
                && first.contains(':')
                && first.parse::<IpAddr>().is_err() =>
        {
            (Some(first.to_owned()), second, after)
        }
        _ => (None, first, rest),
    };
    let (ident, rest) = word(rest)?;
    let (user, rest) = word(rest)?;
    let (time, rest) = bracket_time(rest)?;
    let (request, rest) = quoted(rest)?;
    let (status, rest) = word(rest)?;
    let status = match status {
        "-" => None,
        s if s.len() == 3 => Some(s.parse::<u16>().ok().filter(|c| (100..600).contains(c))?),
        _ => return None,
    };
    let (bytes, mut rest) = word(rest)?;
    if bytes != "-" && !bytes.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut quoted_fields = Vec::new();
    while let Some((q, after)) = quoted(rest) {
        quoted_fields.push(q);
        rest = after;
    }
    if !rest.trim().is_empty() {
        return None;
    }
    let mut q = quoted_fields.into_iter();
    Some(Entry {
        vhost,
        client: client.to_owned(),
        ident: dash(ident),
        user: dash(user),
        time,
        request,
        status,
        bytes: dash(bytes),
        referer: q.next().as_deref().and_then(dash),
        agent: q.next().as_deref().and_then(dash),
        extra: q.collect(),
    })
}

/// Au moins 80 % des premières lignes complètes sont des journaux d'accès.
pub fn detect(head: &[u8]) -> Option<Format> {
    let text = String::from_utf8_lossy(head);
    let mut lines: Vec<&str> = text.lines().take(50).collect();
    if lines.len() > 1 && !text.ends_with('\n') {
        lines.pop(); // dernière ligne possiblement tronquée
    }
    let lines: Vec<&str> = lines
        .into_iter()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let ok = lines.iter().filter(|l| parse_line(l).is_some()).count();
    (ok > 0 && ok * 5 >= lines.len() * 4).then_some(Format::WebAccess)
}

fn to_event(e: Entry, raw: &str, source: SourceRef) -> NewEvent {
    let mut parts = e.request.splitn(3, ' ');
    let (method, path, version) = (parts.next(), parts.next(), parts.next());
    // Une requête mal formée (sonde, binaire) reste telle quelle dans `http.request`.
    let well_formed = path.is_some()
        && method.is_some_and(|m| !m.is_empty() && m.bytes().all(|b| b.is_ascii_uppercase()));
    let message = match (well_formed, e.status) {
        (true, Some(s)) => format!("{} {} → {s}", method.unwrap_or(""), path.unwrap_or("")),
        (true, None) => format!("{} {}", method.unwrap_or(""), path.unwrap_or("")),
        (false, Some(s)) => format!("requête non standard → {s}"),
        (false, None) => "requête non standard".to_owned(),
    };
    let mut ev = NewEvent::new(source, EventKind::Web, clip_chars(&message, 500), raw);
    ev.time = Some(e.time);
    ev.host = e
        .vhost
        .as_deref()
        .map(|v| v.rsplit_once(':').map_or(v, |(h, _)| h))
        .filter(|h| !h.is_empty())
        .map(str::to_owned);
    ev.source_ip = e.client.parse().ok();
    ev.user = e.user;
    ev.action = well_formed.then(|| method.unwrap_or("").to_owned());
    // shortcut: 4xx et 5xx comptent comme échec (requête non servie), sans distinguer
    // refus et erreur serveur ; affiner avec la détection (phase 4).
    ev.outcome = e.status.map(|s| {
        if s < 400 {
            Outcome::Success
        } else {
            Outcome::Failure
        }
    });

    let mut put = |k: &str, v: &str| {
        ev.attributes.insert(
            clip_chars(k, MAX_ATTRIBUTE_KEY_CHARS),
            clip_chars(v, MAX_ATTRIBUTE_VALUE_CHARS),
        );
    };
    if ev.source_ip.is_none() {
        put("client", &e.client); // nom d'hôte résolu au lieu d'une adresse
    }
    if well_formed {
        put("http.method", method.unwrap_or(""));
        put("url.path", path.unwrap_or(""));
        if let Some(v) = version {
            put("http.version", v);
        }
    } else if !e.request.is_empty() && e.request != "-" {
        put("http.request", &e.request);
    }
    if let Some(s) = e.status {
        put("http.status", &s.to_string());
    }
    for (k, v) in [
        ("ident", &e.ident),
        ("http.bytes", &e.bytes),
        ("http.referer", &e.referer),
        ("user_agent", &e.agent),
        ("vhost", &e.vhost),
    ] {
        if let Some(v) = v {
            put(k, v);
        }
    }
    for (i, v) in e.extra.iter().enumerate().take(16) {
        if v != "-" && !v.is_empty() {
            put(&format!("extra.{}", i + 1), v);
        }
    }
    ev
}

pub fn parse(
    reader: impl Read,
    ctx: ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<ParseReport, ParseError> {
    let mut report = ParseReport::new(Format::WebAccess);
    let mut reader = BufReader::with_capacity(1 << 16, reader);
    let (mut offset, mut line_no, mut combined) = (0u64, 0u64, 0u64);
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
        if bytes.is_empty() {
            continue;
        }
        report.records += 1;
        let text = String::from_utf8_lossy(bytes);
        let Some(entry) = parse_line(&text) else {
            report.error(line_no, start, "ligne de journal d'accès non reconnue");
            continue;
        };
        combined += u64::from(entry.agent.is_some() || entry.referer.is_some());
        let source = SourceRef {
            evidence_id: ctx.evidence_id,
            byte_offset: start,
            line: Some(line_no),
            record: Some(report.records),
        };
        let event = to_event(entry, &text, source);
        if let Err(e) = event.validate() {
            report.error(line_no, start, e.to_string());
            continue;
        }
        sink(event).map_err(ParseError::Sink)?;
        report.events += 1;
    }
    if report.events > 0 {
        report.layout = Some(if combined > 0 {
            "format combined".into()
        } else {
            "format common".into()
        });
    }
    (ctx.on_progress)(offset);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use evidentyx_core::{EvidenceId, TimeBasis};
    use std::sync::atomic::AtomicBool;

    fn run(input: &[u8]) -> (ParseReport, Vec<NewEvent>) {
        let cancel = AtomicBool::new(false);
        let mut events = Vec::new();
        let r = crate::parse(
            Format::WebAccess,
            input,
            ParseContext {
                evidence_id: EvidenceId::new(1),
                reference_time: None,
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

    #[test]
    fn common_et_combined() {
        let input = b"127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326\n\
203.0.113.9 - - [16/Nov/2026:10:00:00 +0100] \"POST /login?next=%2F HTTP/1.1\" 401 - \"https://ex.com/\" \"Mozilla/5.0 \\\"x\\\"\"\n\
pas un journal\n";
        let (r, ev) = run(input);
        assert_eq!((r.records, r.events, r.errors), (3, 2, 1));
        assert_eq!(r.layout.as_deref(), Some("format combined"));
        let a = &ev[0];
        assert_eq!(a.user.as_deref(), Some("frank"));
        assert_eq!(a.action.as_deref(), Some("GET"));
        assert_eq!(a.outcome, Some(Outcome::Success));
        assert_eq!(a.message, "GET /apache_pb.gif → 200");
        let t = a.time.unwrap();
        assert_eq!(
            (t.utc_micros, t.basis),
            (
                parse_iso("2000-10-10T20:55:36Z").unwrap().utc_micros,
                TimeBasis::Explicit
            )
        );
        assert_eq!(a.attributes["http.bytes"], "2326");
        let b = &ev[1];
        assert_eq!(b.source.line, Some(2));
        assert_eq!(&input[b.source.byte_offset as usize..][..3], b"203");
        assert_eq!(b.source_ip.unwrap().to_string(), "203.0.113.9");
        assert_eq!(b.outcome, Some(Outcome::Failure));
        assert_eq!(b.attributes["url.path"], "/login?next=%2F");
        assert_eq!(b.attributes["user_agent"], "Mozilla/5.0 \"x\"");
        assert!(!b.attributes.contains_key("http.bytes"));
    }

    #[test]
    fn vhost_extras_et_requete_non_standard() {
        let line = b"www.ex.com:443 2001:db8::1 - - [16/Nov/2026:10:00:00 +0000] \"\\x16\\x03\\x01\" 400 157 \"-\" \"-\" \"10.0.0.7\"\n";
        let (_, ev) = run(line);
        let e = &ev[0];
        assert_eq!(e.host.as_deref(), Some("www.ex.com"));
        assert_eq!(e.source_ip.unwrap().to_string(), "2001:db8::1");
        assert_eq!(e.action, None);
        assert_eq!(e.message, "requête non standard → 400");
        assert_eq!(e.attributes["http.request"], "\\x16\\x03\\x01");
        assert_eq!(e.attributes["extra.1"], "10.0.0.7");
        assert!(!e.attributes.contains_key("user_agent"));
    }

    #[test]
    fn client_par_nom() {
        let (_, ev) =
            run(b"proxy.ex.com - - [16/Nov/2026:10:00:00 +0000] \"GET / HTTP/1.1\" 304 0\n");
        assert_eq!(ev[0].source_ip, None);
        assert_eq!(ev[0].attributes["client"], "proxy.ex.com");
    }

    #[test]
    fn lignes_invalides_et_mutations() {
        for bad in [
            "1.2.3.4 - - [16/Nov/2026:10:00:00] \"GET / HTTP/1.1\" 200 1",
            "1.2.3.4 - - [16/Foo/2026:10:00:00 +0000] \"GET /\" 200 1",
            "1.2.3.4 - - [16/Nov/2026:10:00:00 +0000] GET / 200 1",
            "1.2.3.4 - - [16/Nov/2026:10:00:00 +0000] \"GET /\" 2000 1",
            "1.2.3.4 - - [16/Nov/2026:10:00:00 +0000] \"GET /\" 200 1 garbage",
            "",
            "é",
        ] {
            assert!(parse_line(bad).is_none(), "{bad}");
        }
        let seed: Vec<char> = "a:1 1.2.3.4 - é [16/Nov/2026:10:00:00 +0100] \"GET /é HTTP/1.1\" 200 5 \"r\\\"\" \"u\"".chars().collect();
        let src = SourceRef {
            evidence_id: EvidenceId::new(1),
            byte_offset: 0,
            line: Some(1),
            record: None,
        };
        for i in 0..=seed.len() {
            for x in ["", "\"", "\\", "[", "]", " ", "é", "-"] {
                let mut s: String = seed[..i].iter().collect();
                s.push_str(x);
                s.extend(seed.iter().skip(i + 1));
                if let Some(e) = parse_line(&s) {
                    assert!(to_event(e, &s, src).validate().is_ok(), "{s}");
                }
            }
        }
    }

    #[test]
    fn detection() {
        use crate::detect as d;
        let l = b"127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET / HTTP/1.0\" 200 2326\n";
        assert_eq!(d("access.log", l), Some(Format::WebAccess));
        assert_eq!(
            d("auth.log", b"Nov 16 10:00:00 srv sshd[1]: x\n"),
            Some(Format::Syslog)
        );
        assert_eq!(d("x.csv", b"a,b\n1,2\n"), Some(Format::Csv));
    }
}
