//! Captures réseau : PCAP (libpcap) et PCAPNG.
//!
//! Un événement par paquet : heure de capture, adresses, ports, protocole,
//! drapeaux TCP, clé de flux (identique dans les deux sens). Décodés :
//! Ethernet (VLAN 802.1Q / 802.1ad), Linux « cooked » (SLL, SLL2), boucle
//! locale BSD, IP brut ; IPv4, IPv6 (en-têtes d'extension), ARP ; TCP, UDP,
//! ICMP, ICMPv6. Le contenu applicatif n'est pas interprété.
//!
//! Le texte brut d'un paquet est l'hexadécimal de ses premiers octets (les
//! en-têtes) ; le paquet complet reste dans la preuve, au décalage indiqué.

use crate::{Format, ParseContext, ParseError, ParseReport, MAX_RECORD_BYTES};
use evidentyx_core::{EventKind, EventTime, NewEvent, SourceRef, TimeBasis, TimePrecision};
use std::fmt::Write as _;
use std::io::{self, BufReader, Read};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::atomic::Ordering;

/// Octets d'un paquet conservés en hexadécimal comme texte brut.
// shortcut: en-têtes seulement (stockage borné) ; le paquet entier se relit
// dans la preuve au décalage enregistré. Agrandir si l'affichage du contenu est demandé.
const RAW_BYTES: usize = 96;

const PCAPNG_SHB: u32 = 0x0A0D_0D0A;

pub fn detect(head: &[u8]) -> Option<Format> {
    match head.get(..4)? {
        [0xd4, 0xc3, 0xb2, 0xa1]
        | [0xa1, 0xb2, 0xc3, 0xd4]
        | [0x4d, 0x3c, 0xb2, 0xa1]
        | [0xa1, 0xb2, 0x3c, 0x4d] => Some(Format::Pcap),
        [0x0a, 0x0d, 0x0d, 0x0a] => Some(Format::PcapNg),
        _ => None,
    }
}

// ——— Lecture des octets ———

#[derive(Clone, Copy)]
struct Endian {
    big: bool,
}

impl Endian {
    fn u16(self, b: &[u8]) -> u16 {
        let a = [b[0], b[1]];
        if self.big {
            u16::from_be_bytes(a)
        } else {
            u16::from_le_bytes(a)
        }
    }
    fn u32(self, b: &[u8]) -> u32 {
        let a = [b[0], b[1], b[2], b[3]];
        if self.big {
            u32::from_be_bytes(a)
        } else {
            u32::from_le_bytes(a)
        }
    }
}

/// Lecteur qui compte les octets lus et distingue la fin propre du fichier.
struct Input<R> {
    inner: BufReader<R>,
    offset: u64,
}

impl<R: Read> Input<R> {
    /// Remplit `buf` ; `Ok(false)` si le fichier se termine avant le premier octet.
    fn fill(&mut self, buf: &mut [u8]) -> Result<bool, ReadEnd> {
        let mut done = 0;
        while done < buf.len() {
            match self.inner.read(&mut buf[done..]) {
                Ok(0) => break,
                Ok(n) => done += n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(ReadEnd::Io(e)),
            }
        }
        self.offset += done as u64;
        match done {
            0 if !buf.is_empty() => Ok(false),
            n if n < buf.len() => Err(ReadEnd::Truncated),
            _ => Ok(true),
        }
    }

    fn vec(&mut self, len: usize) -> Result<Vec<u8>, ReadEnd> {
        let mut v = vec![0; len];
        self.fill(&mut v).and_then(|ok| {
            if ok || len == 0 {
                Ok(v)
            } else {
                Err(ReadEnd::Truncated)
            }
        })
    }
}

enum ReadEnd {
    Io(io::Error),
    /// Fin du fichier au milieu d'un enregistrement (capture interrompue).
    Truncated,
}

// ——— Horodatage ———

/// Résolution d'horodatage d'une interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Resolution {
    /// 10^-n seconde.
    Decimal(u8),
    /// 2^-n seconde.
    Binary(u8),
}

impl Resolution {
    fn micros(self, ticks: u64) -> Option<i64> {
        let t = u128::from(ticks);
        let us = match self {
            Self::Decimal(n) if n <= 6 => t.checked_mul(10u128.checked_pow(u32::from(6 - n))?)?,
            Self::Decimal(n) => t / 10u128.checked_pow(u32::from(n - 6))?,
            Self::Binary(n) if n < 100 => (t * 1_000_000) >> n,
            Self::Binary(_) => return None,
        };
        i64::try_from(us).ok()
    }

    fn precision(self) -> TimePrecision {
        match self {
            Self::Decimal(n) if n >= 6 => TimePrecision::Microsecond,
            Self::Binary(n) if n >= 20 => TimePrecision::Microsecond,
            Self::Decimal(n) if n >= 3 => TimePrecision::Millisecond,
            Self::Binary(n) if n >= 10 => TimePrecision::Millisecond,
            _ => TimePrecision::Second,
        }
    }
}

fn event_time(micros: Option<i64>, offset_secs: i64, resolution: Resolution) -> Option<EventTime> {
    Some(EventTime {
        utc_micros: micros?.checked_add(offset_secs.checked_mul(1_000_000)?)?,
        precision: resolution.precision(),
        basis: TimeBasis::Explicit,
    })
}

// ——— Décodage des en-têtes ———

#[derive(Default, Debug, PartialEq, Eq)]
struct Decoded {
    protocol: Option<&'static str>,
    src: Option<IpAddr>,
    dst: Option<IpAddr>,
    sport: Option<u16>,
    dport: Option<u16>,
    attrs: Vec<(&'static str, String)>,
}

fn mac(b: &[u8]) -> String {
    b.iter()
        .map(|x| format!("{x:02x}"))
        .collect::<Vec<_>>()
        .join(":")
}

fn ip4(b: &[u8]) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(b[0], b[1], b[2], b[3]))
}

fn ip6(b: &[u8]) -> IpAddr {
    let mut a = [0u8; 16];
    a.copy_from_slice(&b[..16]);
    IpAddr::V6(Ipv6Addr::from(a))
}

fn link_name(link: u16) -> &'static str {
    match link {
        0 => "boucle locale BSD",
        1 => "Ethernet",
        101 | 228 | 229 => "IP brut",
        113 => "Linux SLL",
        276 => "Linux SLL2",
        _ => "inconnu",
    }
}

fn decode(link: u16, p: &[u8]) -> Decoded {
    let mut d = Decoded::default();
    match link {
        1 => ethernet(p, &mut d),
        0 if p.len() >= 4 => {
            // Famille d'adresse dans l'ordre d'octets de la machine de capture.
            let fam = u32::from_le_bytes([p[0], p[1], p[2], p[3]])
                .min(u32::from_be_bytes([p[0], p[1], p[2], p[3]]));
            match fam {
                2 => ipv4(&p[4..], &mut d),
                24 | 28 | 30 => ipv6(&p[4..], &mut d),
                _ => {}
            }
        }
        101 => match p.first().map(|b| b >> 4) {
            Some(4) => ipv4(p, &mut d),
            Some(6) => ipv6(p, &mut d),
            _ => {}
        },
        228 => ipv4(p, &mut d),
        229 => ipv6(p, &mut d),
        113 if p.len() >= 16 => ether_type(u16::from_be_bytes([p[14], p[15]]), &p[16..], &mut d),
        276 if p.len() >= 20 => ether_type(u16::from_be_bytes([p[0], p[1]]), &p[20..], &mut d),
        _ => {}
    }
    d
}

fn ethernet(p: &[u8], d: &mut Decoded) {
    if p.len() < 14 {
        return;
    }
    d.attrs.push(("eth.dst", mac(&p[0..6])));
    d.attrs.push(("eth.src", mac(&p[6..12])));
    let (mut ty, mut rest) = (u16::from_be_bytes([p[12], p[13]]), &p[14..]);
    for _ in 0..2 {
        if !matches!(ty, 0x8100 | 0x88a8) || rest.len() < 4 {
            break;
        }
        d.attrs.push((
            "vlan.id",
            (u16::from_be_bytes([rest[0], rest[1]]) & 0x0fff).to_string(),
        ));
        ty = u16::from_be_bytes([rest[2], rest[3]]);
        rest = &rest[4..];
    }
    ether_type(ty, rest, d);
}

fn ether_type(ty: u16, p: &[u8], d: &mut Decoded) {
    match ty {
        0x0800 => ipv4(p, d),
        0x86dd => ipv6(p, d),
        0x0806 => arp(p, d),
        other => d.attrs.push(("eth.type", format!("0x{other:04x}"))),
    }
}

fn arp(p: &[u8], d: &mut Decoded) {
    d.protocol = Some("ARP");
    // Ethernet + IPv4 uniquement (hlen 6, plen 4).
    if p.len() < 28 || p[4] != 6 || p[5] != 4 {
        return;
    }
    let op = u16::from_be_bytes([p[6], p[7]]);
    d.attrs.push((
        "arp.op",
        match op {
            1 => "request".into(),
            2 => "reply".into(),
            o => o.to_string(),
        },
    ));
    d.attrs.push(("arp.sender_mac", mac(&p[8..14])));
    d.src = Some(ip4(&p[14..18]));
    d.dst = Some(ip4(&p[24..28]));
}

fn ipv4(p: &[u8], d: &mut Decoded) {
    if p.len() < 20 || p[0] >> 4 != 4 {
        return;
    }
    let ihl = usize::from(p[0] & 0x0f) * 4;
    d.src = Some(ip4(&p[12..16]));
    d.dst = Some(ip4(&p[16..20]));
    d.attrs.push(("ip.ttl", p[8].to_string()));
    let fragment_offset = u16::from_be_bytes([p[6], p[7]]) & 0x1fff;
    if ihl < 20 || p.len() < ihl {
        return;
    }
    if fragment_offset != 0 {
        d.attrs.push(("ip.fragment", "suite".into()));
        d.protocol = proto_name(p[9]);
        return;
    }
    transport(p[9], &p[ihl..], d, false);
}

fn ipv6(p: &[u8], d: &mut Decoded) {
    if p.len() < 40 || p[0] >> 4 != 6 {
        return;
    }
    d.src = Some(ip6(&p[8..24]));
    d.dst = Some(ip6(&p[24..40]));
    d.attrs.push(("ip.ttl", p[7].to_string()));
    let (mut next, mut rest) = (p[6], &p[40..]);
    // En-têtes d'extension, en nombre borné.
    for _ in 0..8 {
        let len = match next {
            0 | 43 | 60 if rest.len() >= 2 => (usize::from(rest[1]) + 1) * 8,
            51 if rest.len() >= 2 => (usize::from(rest[1]) + 2) * 4,
            44 if rest.len() >= 8 => {
                if u16::from_be_bytes([rest[2], rest[3]]) >> 3 != 0 {
                    d.attrs.push(("ip.fragment", "suite".into()));
                    d.protocol = proto_name(rest[0]);
                    return;
                }
                8
            }
            _ => break,
        };
        if rest.len() < len {
            return;
        }
        next = rest[0];
        rest = &rest[len..];
    }
    transport(next, rest, d, true);
}

fn proto_name(proto: u8) -> Option<&'static str> {
    Some(match proto {
        1 => "ICMP",
        6 => "TCP",
        17 => "UDP",
        58 => "ICMPv6",
        2 => "IGMP",
        47 => "GRE",
        50 => "ESP",
        132 => "SCTP",
        _ => return None,
    })
}

fn transport(proto: u8, p: &[u8], d: &mut Decoded, v6: bool) {
    d.protocol = proto_name(proto);
    if d.protocol.is_none() {
        d.attrs.push(("ip.protocol", proto.to_string()));
        return;
    }
    match proto {
        6 | 17 | 132 if p.len() >= 4 => {
            d.sport = Some(u16::from_be_bytes([p[0], p[1]]));
            d.dport = Some(u16::from_be_bytes([p[2], p[3]]));
            if proto == 6 && p.len() >= 14 {
                d.attrs.push(("tcp.flags", tcp_flags(p[13])));
            }
        }
        1 | 58 if p.len() >= 2 && (proto == 58) == v6 => {
            d.attrs.push(("icmp.type", p[0].to_string()));
            d.attrs.push(("icmp.code", p[1].to_string()));
        }
        _ => {}
    }
}

fn tcp_flags(f: u8) -> String {
    const NAMES: [&str; 8] = ["FIN", "SYN", "RST", "PSH", "ACK", "URG", "ECE", "CWR"];
    let v: Vec<&str> = (0..8)
        .filter(|i| f & (1 << i) != 0)
        .map(|i| NAMES[i])
        .collect();
    if v.is_empty() {
        "aucun".into()
    } else {
        v.join(",")
    }
}

fn endpoint(ip: IpAddr, port: Option<u16>) -> String {
    match (ip, port) {
        (IpAddr::V6(a), Some(p)) => format!("[{a}]:{p}"),
        (a, Some(p)) => format!("{a}:{p}"),
        (a, None) => a.to_string(),
    }
}

/// Paquet lu, avant conversion.
struct Packet<'a> {
    link: u16,
    data: &'a [u8],
    orig_len: u32,
    time: Option<EventTime>,
    interface: Option<u32>,
}

fn to_event(pkt: Packet<'_>, source: SourceRef) -> NewEvent {
    let d = decode(pkt.link, pkt.data);
    let size = format!("{} o", pkt.orig_len);
    let message = match (d.protocol, d.src, d.dst) {
        (proto, Some(s), Some(t)) => {
            let flags = d
                .attrs
                .iter()
                .find(|(k, _)| *k == "tcp.flags")
                .map(|(_, v)| format!(" [{v}]"))
                .unwrap_or_default();
            format!(
                "{} {} → {}{flags} · {size}",
                proto.unwrap_or("IP"),
                endpoint(s, d.sport),
                endpoint(t, d.dport)
            )
        }
        _ => format!("Paquet {} · {size}", link_name(pkt.link)),
    };
    let mut raw = String::with_capacity(RAW_BYTES * 2);
    for b in pkt.data.iter().take(RAW_BYTES) {
        let _ = write!(raw, "{b:02x}");
    }
    let mut ev = NewEvent::new(source, EventKind::Network, message, &raw);
    ev.raw_truncated = pkt.data.len() > RAW_BYTES;
    ev.time = pkt.time;
    ev.source_ip = d.src;
    ev.destination_ip = d.dst;
    ev.source_port = d.sport;
    ev.destination_port = d.dport;
    ev.action = d.protocol.map(str::to_owned);
    let a = &mut ev.attributes;
    a.insert("frame.len".into(), pkt.orig_len.to_string());
    a.insert("frame.captured".into(), pkt.data.len().to_string());
    a.insert(
        "link.type".into(),
        format!("{} ({})", pkt.link, link_name(pkt.link)),
    );
    if let Some(i) = pkt.interface {
        a.insert("frame.interface".into(), i.to_string());
    }
    if let (Some(proto), Some(s), Some(t)) = (d.protocol, d.src, d.dst) {
        // Clé de flux identique dans les deux sens.
        let (x, y) = (endpoint(s, d.sport), endpoint(t, d.dport));
        let (lo, hi) = if (s, d.sport) <= (t, d.dport) {
            (x, y)
        } else {
            (y, x)
        };
        a.insert("flow".into(), format!("{proto} {lo} ↔ {hi}"));
    }
    for (k, v) in d.attrs {
        a.insert(k.into(), v);
    }
    ev
}

// ——— Lecture des fichiers ———

pub fn parse(
    format: Format,
    reader: impl Read,
    mut ctx: ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<ParseReport, ParseError> {
    let ctx = &mut ctx;
    let mut input = Input {
        inner: BufReader::with_capacity(1 << 16, reader),
        offset: 0,
    };
    let mut report = ParseReport::new(format);
    let result = if format == Format::PcapNg {
        parse_ng(&mut input, ctx, &mut report, sink)
    } else {
        parse_classic(&mut input, ctx, &mut report, sink)
    };
    report.bytes = input.offset;
    (ctx.on_progress)(input.offset);
    result.map(|()| report)
}

fn malformed(offset: u64, record: u64, message: impl Into<String>) -> ParseError {
    ParseError::Malformed {
        line: record,
        byte_offset: offset,
        message: message.into(),
    }
}

/// Fin de fichier au milieu d'un enregistrement : signalée, la lecture s'arrête
/// sans perdre les paquets précédents.
fn on_end(e: ReadEnd, report: &mut ParseReport, record: u64, start: u64) -> Result<(), ParseError> {
    match e {
        ReadEnd::Io(e) => Err(ParseError::Io(e)),
        ReadEnd::Truncated => {
            report.records += 1;
            report.error(
                record,
                start,
                "paquet tronqué : le fichier se termine avant la fin de l'enregistrement",
            );
            Ok(())
        }
    }
}

fn emit(
    report: &mut ParseReport,
    ctx: &ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
    start: u64,
    pkt: Packet<'_>,
) -> Result<(), ParseError> {
    report.records += 1;
    let source = SourceRef {
        evidence_id: ctx.evidence_id,
        byte_offset: start,
        line: None,
        record: Some(report.records),
    };
    let event = to_event(pkt, source);
    if let Err(e) = event.validate() {
        report.error(report.records, start, e.to_string());
        return Ok(());
    }
    sink(event).map_err(ParseError::Sink)?;
    report.events += 1;
    Ok(())
}

fn tick(report: &ParseReport, ctx: &mut ParseContext<'_>, offset: u64) -> Result<(), ParseError> {
    if ctx.cancel.load(Ordering::Relaxed) {
        return Err(ParseError::Cancelled);
    }
    if report.records.is_multiple_of(1024) {
        (ctx.on_progress)(offset);
    }
    Ok(())
}

fn parse_classic<R: Read>(
    input: &mut Input<R>,
    ctx: &mut ParseContext<'_>,
    report: &mut ParseReport,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<(), ParseError> {
    let mut h = [0u8; 24];
    match input.fill(&mut h) {
        Ok(true) => {}
        Ok(false) | Err(ReadEnd::Truncated) => {
            return Err(malformed(0, 0, "en-tête PCAP incomplet"))
        }
        Err(ReadEnd::Io(e)) => return Err(ParseError::Io(e)),
    }
    let (big, nanos) = match h[..4] {
        [0xd4, 0xc3, 0xb2, 0xa1] => (false, false),
        [0xa1, 0xb2, 0xc3, 0xd4] => (true, false),
        [0x4d, 0x3c, 0xb2, 0xa1] => (false, true),
        [0xa1, 0xb2, 0x3c, 0x4d] => (true, true),
        _ => return Err(malformed(0, 0, "signature PCAP inconnue")),
    };
    let en = Endian { big };
    // Les 16 bits de poids fort peuvent porter la FCS : seul le type de lien compte.
    let link = (en.u32(&h[20..24]) & 0xffff) as u16;
    let resolution = if nanos {
        Resolution::Decimal(9)
    } else {
        Resolution::Decimal(6)
    };
    report.layout = Some(format!(
        "PCAP {}.{} · lien {} · {}",
        en.u16(&h[4..6]),
        en.u16(&h[6..8]),
        link_name(link),
        if nanos {
            "nanosecondes"
        } else {
            "microsecondes"
        }
    ));
    let mut rec = [0u8; 16];
    loop {
        tick(report, ctx, input.offset)?;
        let start = input.offset;
        let number = report.records + 1;
        match input.fill(&mut rec) {
            Ok(true) => {}
            Ok(false) => return Ok(()),
            Err(e) => return on_end(e, report, number, start),
        }
        let caplen = en.u32(&rec[8..12]) as usize;
        if caplen > MAX_RECORD_BYTES {
            return Err(malformed(
                start,
                number,
                format!("paquet {number} : taille capturée invalide ({caplen} octets)"),
            ));
        }
        let data = match input.vec(caplen) {
            Ok(v) => v,
            Err(e) => return on_end(e, report, number, start),
        };
        let (secs, frac) = (u64::from(en.u32(&rec[0..4])), u64::from(en.u32(&rec[4..8])));
        let per_sec = if nanos { 1_000_000_000 } else { 1_000_000 };
        let time = event_time(resolution.micros(secs * per_sec + frac), 0, resolution);
        let pkt = Packet {
            link,
            data: &data,
            orig_len: en.u32(&rec[12..16]),
            time,
            interface: None,
        };
        emit(report, ctx, sink, start, pkt)?;
    }
}

struct Interface {
    link: u16,
    resolution: Resolution,
    offset_secs: i64,
}

/// Options PCAPNG (code, valeur) d'un corps de bloc ; s'arrête à la fin ou à une option mal formée.
fn options(en: Endian, mut b: &[u8]) -> Vec<(u16, &[u8])> {
    let mut out = Vec::new();
    while b.len() >= 4 {
        let (code, len) = (en.u16(&b[0..2]), usize::from(en.u16(&b[2..4])));
        if code == 0 || b.len() < 4 + len {
            break;
        }
        out.push((code, &b[4..4 + len]));
        b = &b[(4 + len.div_ceil(4) * 4).min(b.len())..];
    }
    out
}

fn interface(en: Endian, body: &[u8]) -> Option<Interface> {
    if body.len() < 8 {
        return None;
    }
    let mut i = Interface {
        link: en.u16(&body[0..2]),
        resolution: Resolution::Decimal(6),
        offset_secs: 0,
    };
    for (code, v) in options(en, &body[8..]) {
        match (code, v) {
            (9, [r]) => {
                i.resolution = if r & 0x80 == 0 {
                    Resolution::Decimal(*r)
                } else {
                    Resolution::Binary(r & 0x7f)
                }
            }
            (14, v) => {
                if let Ok(b) = <[u8; 8]>::try_from(v) {
                    i.offset_secs = if en.big {
                        i64::from_be_bytes(b)
                    } else {
                        i64::from_le_bytes(b)
                    };
                }
            }
            _ => {}
        }
    }
    Some(i)
}

fn parse_ng<R: Read>(
    input: &mut Input<R>,
    ctx: &mut ParseContext<'_>,
    report: &mut ParseReport,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<(), ParseError> {
    let mut en = Endian { big: false };
    let mut interfaces: Vec<Interface> = Vec::new();
    let (mut sections, mut total_interfaces) = (0u32, 0usize);
    let mut head = [0u8; 8];
    loop {
        tick(report, ctx, input.offset)?;
        let start = input.offset;
        let number = report.records + 1;
        match input.fill(&mut head) {
            Ok(true) => {}
            Ok(false) => break,
            Err(e) => {
                on_end(e, report, number, start)?;
                break;
            }
        }
        let raw_type = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
        if raw_type == PCAPNG_SHB {
            // L'ordre des octets de la section est donné par la signature qui suit.
            let mut magic = [0u8; 4];
            if let Err(e) = input.fill(&mut magic) {
                return Err(match e {
                    ReadEnd::Io(e) => ParseError::Io(e),
                    ReadEnd::Truncated => {
                        malformed(start, number, "en-tête de section PCAPNG incomplet")
                    }
                });
            }
            en = match magic {
                [0x4d, 0x3c, 0x2b, 0x1a] => Endian { big: false },
                [0x1a, 0x2b, 0x3c, 0x4d] => Endian { big: true },
                _ => {
                    return Err(malformed(
                        start,
                        number,
                        "signature d'ordre des octets PCAPNG inconnue",
                    ))
                }
            };
            sections += 1;
            interfaces.clear();
        } else if sections == 0 {
            return Err(malformed(
                start,
                number,
                "bloc PCAPNG avant l'en-tête de section",
            ));
        }
        let block_type = en.u32(&head[0..4]);
        let len = en.u32(&head[4..8]) as usize;
        let already = if block_type == PCAPNG_SHB { 12 } else { 8 };
        if len < 12 || !len.is_multiple_of(4) || len > MAX_RECORD_BYTES + 64 {
            return Err(malformed(
                start,
                number,
                format!("bloc PCAPNG de longueur invalide ({len} octets)"),
            ));
        }
        let rest = match input.vec(len - already) {
            Ok(v) => v,
            Err(e) => {
                if block_type == 6 || block_type == 3 || block_type == 2 {
                    on_end(e, report, number, start)?;
                } else if let ReadEnd::Io(e) = e {
                    return Err(ParseError::Io(e));
                }
                break;
            }
        };
        let (body, trailer) = rest.split_at(rest.len() - 4);
        if en.u32(trailer) as usize != len {
            return Err(malformed(
                start,
                number,
                "bloc PCAPNG incohérent (longueurs de début et de fin différentes)",
            ));
        }
        match block_type {
            1 => match interface(en, body) {
                Some(i) => {
                    interfaces.push(i);
                    total_interfaces += 1;
                }
                None => {
                    return Err(malformed(
                        start,
                        number,
                        "description d'interface PCAPNG incomplète",
                    ))
                }
            },
            // Paquet amélioré (6) et ancien bloc de paquet (2).
            6 | 2 if body.len() >= 20 => {
                let (iface, ts_at) = if block_type == 6 {
                    (en.u32(&body[0..4]), 4)
                } else {
                    (u32::from(en.u16(&body[0..2])), 4)
                };
                let caplen = en.u32(&body[12..16]) as usize;
                let Some(data) = body.get(20..20 + caplen) else {
                    report.records += 1;
                    report.error(
                        number,
                        start,
                        "paquet incohérent : taille capturée supérieure au bloc",
                    );
                    continue;
                };
                let Some(i) = interfaces.get(iface as usize) else {
                    report.records += 1;
                    report.error(
                        number,
                        start,
                        format!("paquet sur une interface non décrite ({iface})"),
                    );
                    continue;
                };
                let ticks = u64::from(en.u32(&body[ts_at..ts_at + 4])) << 32
                    | u64::from(en.u32(&body[ts_at + 4..ts_at + 8]));
                let time = event_time(i.resolution.micros(ticks), i.offset_secs, i.resolution);
                let pkt = Packet {
                    link: i.link,
                    data,
                    orig_len: en.u32(&body[16..20]),
                    time,
                    interface: Some(iface),
                };
                emit(report, ctx, sink, start, pkt)?;
            }
            // Paquet simple : sans heure, interface 0.
            3 if body.len() >= 4 => {
                let Some(i) = interfaces.first() else {
                    report.records += 1;
                    report.error(number, start, "paquet simple sans interface décrite");
                    continue;
                };
                let orig_len = en.u32(&body[0..4]);
                let data = &body[4..body.len().min(4 + orig_len as usize)];
                let pkt = Packet {
                    link: i.link,
                    data,
                    orig_len,
                    time: None,
                    interface: Some(0),
                };
                emit(report, ctx, sink, start, pkt)?;
            }
            6 | 2 | 3 => {
                report.records += 1;
                report.error(number, start, "bloc de paquet trop court");
            }
            _ => {} // statistiques, résolution de noms, journaux de déchiffrement… ignorés
        }
    }
    if sections == 0 {
        return Err(malformed(0, 0, "en-tête de section PCAPNG absent"));
    }
    report.layout = Some(format!(
        "PCAPNG · {sections} section{} · {total_interfaces} interface{}",
        if sections > 1 { "s" } else { "" },
        if total_interfaces > 1 { "s" } else { "" }
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use evidentyx_core::{EvidenceId, Outcome};
    use std::sync::atomic::AtomicBool;

    fn run(input: &[u8]) -> Result<(ParseReport, Vec<NewEvent>), ParseError> {
        let format = detect(input).expect("format reconnu");
        let cancel = AtomicBool::new(false);
        let mut events = Vec::new();
        let r = crate::parse(
            format,
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
        )?;
        Ok((r, events))
    }

    /// Ethernet + IPv4 + TCP SYN 10.0.0.1:51000 → 10.0.0.2:443.
    fn tcp_syn() -> Vec<u8> {
        let mut p = vec![0x02, 0, 0, 0, 0, 2, 0x02, 0, 0, 0, 0, 1, 0x08, 0x00];
        p.extend([
            0x45, 0, 0, 40, 0, 1, 0x40, 0, 64, 6, 0, 0, 10, 0, 0, 1, 10, 0, 0, 2,
        ]);
        p.extend([
            0xc7, 0x38, 0x01, 0xbb, 0, 0, 0, 1, 0, 0, 0, 0, 0x50, 0x02, 0xff, 0xff, 0, 0, 0, 0,
        ]);
        p
    }

    /// Ethernet + VLAN 10 + IPv6 + UDP [2001:db8::1]:53 → [2001:db8::2]:5353.
    fn udp6_vlan() -> Vec<u8> {
        let mut p = vec![
            0x02, 0, 0, 0, 0, 2, 0x02, 0, 0, 0, 0, 1, 0x81, 0x00, 0x00, 0x0a, 0x86, 0xdd,
        ];
        p.extend([0x60, 0, 0, 0, 0, 8, 17, 64]);
        p.extend([0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        p.extend([0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]);
        p.extend([0, 53, 0x14, 0xe9, 0, 8, 0, 0]);
        p
    }

    fn pcap(big: bool, nanos: bool, link: u32, packets: &[(u32, u32, &[u8])]) -> Vec<u8> {
        let w32 = |v: u32| {
            if big {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            }
        };
        let w16 = |v: u16| {
            if big {
                v.to_be_bytes()
            } else {
                v.to_le_bytes()
            }
        };
        let mut f = w32(if nanos { 0xa1b2_3c4d } else { 0xa1b2_c3d4 }).to_vec();
        f.extend(w16(2));
        f.extend(w16(4));
        f.extend([0; 8]);
        f.extend(w32(65535));
        f.extend(w32(link));
        for (s, frac, data) in packets {
            f.extend(w32(*s));
            f.extend(w32(*frac));
            f.extend(w32(data.len() as u32));
            f.extend(w32(data.len() as u32 + 10));
            f.extend(*data);
        }
        f
    }

    #[test]
    fn pcap_classique_deux_ordres_d_octets() {
        let (syn, udp) = (tcp_syn(), udp6_vlan());
        for big in [false, true] {
            let f = pcap(
                big,
                false,
                1,
                &[(1_763_251_200, 123_456, &syn), (1_763_251_201, 0, &udp)],
            );
            let (r, ev) = run(&f).unwrap();
            assert_eq!(
                (r.format, r.records, r.events, r.errors),
                (Format::Pcap, 2, 2, 0)
            );
            assert_eq!(
                r.layout.as_deref(),
                Some("PCAP 2.4 · lien Ethernet · microsecondes")
            );
            let a = &ev[0];
            assert_eq!(a.source.byte_offset, 24);
            assert_eq!(a.source.record, Some(1));
            let t = a.time.unwrap();
            assert_eq!(
                (t.utc_micros, t.precision, t.basis),
                (
                    1_763_251_200_123_456,
                    TimePrecision::Microsecond,
                    TimeBasis::Explicit
                )
            );
            assert_eq!(a.message, "TCP 10.0.0.1:51000 → 10.0.0.2:443 [SYN] · 64 o");
            assert_eq!(
                (a.source_port, a.destination_port, a.action.as_deref()),
                (Some(51000), Some(443), Some("TCP"))
            );
            assert_eq!(a.attributes["flow"], "TCP 10.0.0.1:51000 ↔ 10.0.0.2:443");
            assert_eq!(a.attributes["eth.src"], "02:00:00:00:00:01");
            assert_eq!(a.attributes["frame.captured"], "54");
            assert_eq!(a.outcome, None::<Outcome>);
            assert!(a.raw.starts_with("020000000002"));
            let b = &ev[1];
            assert_eq!(b.source.byte_offset, 24 + 16 + 54);
            assert_eq!(
                b.message,
                "UDP [2001:db8::1]:53 → [2001:db8::2]:5353 · 76 o"
            );
            assert_eq!(b.attributes["vlan.id"], "10");
        }
    }

    #[test]
    fn nanosecondes_et_ip_brut() {
        let syn = tcp_syn();
        let f = pcap(false, true, 101, &[(10, 999_999_999, &syn[14..])]);
        let (r, ev) = run(&f).unwrap();
        assert!(r.layout.unwrap().ends_with("nanosecondes"));
        assert_eq!(ev[0].time.unwrap().utc_micros, 10_999_999);
        assert_eq!(ev[0].destination_port, Some(443));
    }

    #[test]
    fn flux_identique_dans_les_deux_sens() {
        let mut back = tcp_syn();
        back[26..30].copy_from_slice(&[10, 0, 0, 2]);
        back[30..34].copy_from_slice(&[10, 0, 0, 1]);
        back[34..38].copy_from_slice(&[0x01, 0xbb, 0xc7, 0x38]);
        back[47] = 0x12; // SYN,ACK
        let syn = tcp_syn();
        let (_, ev) = run(&pcap(false, false, 1, &[(0, 0, &syn), (0, 1, &back)])).unwrap();
        assert_eq!(ev[0].attributes["flow"], ev[1].attributes["flow"]);
        assert_eq!(ev[1].attributes["tcp.flags"], "SYN,ACK");
    }

    #[test]
    fn capture_interrompue_et_taille_invalide() {
        let syn = tcp_syn();
        let mut f = pcap(false, false, 1, &[(0, 0, &syn), (0, 1, &syn)]);
        f.truncate(f.len() - 5);
        let (r, ev) = run(&f).unwrap();
        assert_eq!((r.records, r.events, r.errors), (2, 1, 1));
        assert_eq!(ev.len(), 1);
        assert_eq!(r.first_errors[0].line, 2);

        let mut f = pcap(false, false, 1, &[(0, 0, &syn)]);
        f[32..36].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(run(&f), Err(ParseError::Malformed { .. })));
        assert!(matches!(run(&f[..10]), Err(ParseError::Malformed { .. })));
    }

    fn block(ty: u32, body: &[u8]) -> Vec<u8> {
        let len = (12 + body.len().div_ceil(4) * 4) as u32;
        let mut b = ty.to_le_bytes().to_vec();
        b.extend(len.to_le_bytes());
        b.extend(body);
        b.resize(len as usize - 4, 0);
        b.extend(len.to_le_bytes());
        b
    }

    fn shb() -> Vec<u8> {
        let mut body = 0x1a2b_3c4du32.to_le_bytes().to_vec();
        body.extend([1, 0, 0, 0]);
        body.extend(u64::MAX.to_le_bytes());
        block(PCAPNG_SHB, &body)
    }

    fn idb(link: u16, tsresol: Option<u8>) -> Vec<u8> {
        let mut body = link.to_le_bytes().to_vec();
        body.extend([0, 0, 0, 0, 1, 0]);
        if let Some(r) = tsresol {
            body.extend([9, 0, 1, 0, r, 0, 0, 0]);
        }
        body.extend([0, 0, 0, 0]);
        block(1, &body)
    }

    fn epb(iface: u32, ticks: u64, data: &[u8]) -> Vec<u8> {
        let mut body = iface.to_le_bytes().to_vec();
        body.extend(((ticks >> 32) as u32).to_le_bytes());
        body.extend((ticks as u32).to_le_bytes());
        body.extend((data.len() as u32).to_le_bytes());
        body.extend((data.len() as u32).to_le_bytes());
        body.extend(data);
        block(6, &body)
    }

    #[test]
    fn pcapng_interfaces_resolutions_et_blocs_ignores() {
        let syn = tcp_syn();
        let mut f = shb();
        f.extend(idb(1, None));
        f.extend(idb(1, Some(9)));
        f.extend(block(5, &[0; 8])); // statistiques : ignorées
        f.extend(epb(0, 1_000_000, &syn));
        f.extend(epb(1, 2_000_000_123, &syn));
        f.extend(epb(7, 0, &syn)); // interface non décrite
        let mut spb = (syn.len() as u32).to_le_bytes().to_vec();
        spb.extend(&syn);
        f.extend(block(3, &spb));
        let (r, ev) = run(&f).unwrap();
        assert_eq!(
            (r.format, r.records, r.events, r.errors),
            (Format::PcapNg, 4, 3, 1)
        );
        assert_eq!(
            r.layout.as_deref(),
            Some("PCAPNG · 1 section · 2 interfaces")
        );
        assert_eq!(ev[0].time.unwrap().utc_micros, 1_000_000);
        assert_eq!(ev[1].time.unwrap().utc_micros, 2_000_000);
        assert_eq!(ev[1].attributes["frame.interface"], "1");
        assert_eq!(ev[2].time, None);
        assert_eq!(
            ev[0].source.byte_offset as usize,
            shb().len() + idb(1, None).len() + idb(1, Some(9)).len() + 20
        );
    }

    #[test]
    fn pcapng_bloc_incoherent_arrete_la_lecture() {
        let mut f = shb();
        f.extend(idb(1, None));
        let mut bad = epb(0, 0, &tcp_syn());
        let n = bad.len();
        bad[n - 4] ^= 0xff;
        f.extend(bad);
        assert!(matches!(run(&f), Err(ParseError::Malformed { .. })));
    }

    #[test]
    fn resolutions() {
        assert_eq!(Resolution::Decimal(3).micros(1_500), Some(1_500_000));
        assert_eq!(Resolution::Binary(10).micros(1024), Some(1_000_000));
        assert_eq!(Resolution::Decimal(200).micros(1), None);
        assert_eq!(Resolution::Decimal(0).precision(), TimePrecision::Second);
        assert_eq!(
            Resolution::Binary(30).precision(),
            TimePrecision::Microsecond
        );
    }

    #[test]
    fn decodage_robuste_aux_octets_quelconques() {
        // Toutes les longueurs de paquets tronqués et des octets pseudo-aléatoires :
        // aucune panique, événement toujours valide.
        let base = [tcp_syn(), udp6_vlan()];
        let mut x: u32 = 0x1234_5678;
        for link in [0u16, 1, 101, 113, 228, 229, 276, 999] {
            for p in &base {
                for cut in 0..=p.len() {
                    let mut data = p[..cut].to_vec();
                    for b in data.iter_mut().skip(12) {
                        x ^= x << 13;
                        x ^= x >> 17;
                        x ^= x << 5;
                        if x.is_multiple_of(7) {
                            *b = x as u8;
                        }
                    }
                    let src = SourceRef {
                        evidence_id: EvidenceId::new(1),
                        byte_offset: 0,
                        line: None,
                        record: Some(1),
                    };
                    let pkt = Packet {
                        link,
                        data: &data,
                        orig_len: 0,
                        time: None,
                        interface: None,
                    };
                    assert!(to_event(pkt, src).validate().is_ok());
                }
            }
        }
    }

    #[test]
    fn detection() {
        assert_eq!(
            crate::detect("a.bin", &[0xd4, 0xc3, 0xb2, 0xa1, 2, 0]),
            Some(Format::Pcap)
        );
        assert_eq!(
            crate::detect("a.bin", &[0x0a, 0x0d, 0x0d, 0x0a]),
            Some(Format::PcapNg)
        );
        assert_eq!(crate::detect("a.pcap", b"texte"), None);
    }
}
