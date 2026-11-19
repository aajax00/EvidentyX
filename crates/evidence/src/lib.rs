//! # evidentyx-evidence
//!
//! Lecture des fichiers de preuve : métadonnées et empreinte SHA-256.
//!
//! Garanties :
//! - les fichiers sont ouverts **en lecture seule** et lus en flux (mémoire
//!   constante, quelle que soit leur taille) ; ils ne sont jamais modifiés,
//!   déplacés ni copiés ;
//! - le calcul peut être suivi (progression) et interrompu (annulation).

use evidentyx_core::{Sha256Digest, Timestamp};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

/// Taille des blocs lus (1 Mio).
const CHUNK: usize = 1 << 20;

/// Empreinte et métadonnées d'un fichier de preuve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    /// Chemin absolu et canonique du fichier.
    pub path: PathBuf,
    /// Nom du fichier.
    pub file_name: String,
    pub size_bytes: u64,
    pub sha256: Sha256Digest,
    /// Dernière modification selon le système de fichiers, si disponible.
    pub modified_at: Option<Timestamp>,
}

/// Erreur de lecture d'une preuve.
#[derive(Debug)]
pub enum EvidenceError {
    /// Fichier introuvable.
    NotFound(PathBuf),
    /// Le chemin désigne autre chose qu'un fichier (dossier, périphérique…).
    NotAFile(PathBuf),
    /// Chemin non représentable en UTF-8 (non pris en charge pour l'instant).
    NonUtf8Path(PathBuf),
    /// Lecture interrompue à la demande de l'utilisateur.
    Cancelled,
    /// Autre erreur d'entrée/sortie (droits, disque…).
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "fichier introuvable : {}", p.display()),
            Self::NotAFile(p) => write!(f, "ce n'est pas un fichier : {}", p.display()),
            Self::NonUtf8Path(p) => write!(
                f,
                "chemin contenant des caractères non pris en charge : {}",
                p.display()
            ),
            Self::Cancelled => write!(f, "lecture annulée"),
            Self::Io { path, source } => {
                write!(f, "lecture impossible de {} : {source}", path.display())
            }
        }
    }
}

impl std::error::Error for EvidenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(path: &Path, source: io::Error) -> EvidenceError {
    if source.kind() == io::ErrorKind::NotFound {
        EvidenceError::NotFound(path.to_path_buf())
    } else {
        EvidenceError::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

/// Calcule l'empreinte SHA-256 d'un fichier, en flux.
///
/// `on_progress(lus, total)` est appelé après chaque bloc ; `cancel` est
/// consulté entre deux blocs : s'il vaut `true`, la lecture s'arrête avec
/// `EvidenceError::Cancelled`.
pub fn fingerprint(
    path: &Path,
    mut on_progress: impl FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<Fingerprint, EvidenceError> {
    let canonical = path.canonicalize().map_err(|e| io_error(path, e))?;
    let metadata = canonical.metadata().map_err(|e| io_error(&canonical, e))?;
    if !metadata.is_file() {
        return Err(EvidenceError::NotAFile(canonical));
    }
    let file_name = canonical
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| EvidenceError::NonUtf8Path(canonical.clone()))?
        .to_owned();
    if canonical.to_str().is_none() {
        return Err(EvidenceError::NonUtf8Path(canonical));
    }

    let total = metadata.len();
    // File::open ouvre en lecture seule.
    let mut file = File::open(&canonical).map_err(|e| io_error(&canonical, e))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK];
    let mut read_total = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(EvidenceError::Cancelled);
        }
        let n = match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(io_error(&canonical, e)),
        };
        hasher.update(&buffer[..n]);
        read_total += n as u64;
        on_progress(read_total, total);
    }

    let digest: [u8; 32] = hasher.finalize().into();
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .map(Timestamp::from_unix_millis);

    Ok(Fingerprint {
        path: canonical,
        file_name,
        // Taille réellement lue : fait foi si le fichier a changé pendant la lecture.
        size_bytes: read_total,
        sha256: Sha256Digest::from_bytes(digest),
        modified_at,
    })
}

/// Recalcule l'empreinte d'un fichier pour une vérification d'intégrité.
/// Renvoie `None` si le fichier est introuvable ou illisible.
pub fn current_digest(
    path: &Path,
    cancel: &AtomicBool,
) -> Result<Option<Sha256Digest>, EvidenceError> {
    match fingerprint(path, |_, _| {}, cancel) {
        Ok(f) => Ok(Some(f.sha256)),
        Err(EvidenceError::Cancelled) => Err(EvidenceError::Cancelled),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn no_cancel() -> AtomicBool {
        AtomicBool::new(false)
    }

    fn file_with(content: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preuve.log");
        File::create(&path).unwrap().write_all(content).unwrap();
        (dir, path)
    }

    #[test]
    fn vecteurs_de_reference_sha256() {
        // Vecteurs du NIST (FIPS 180-2).
        let cases: [(&[u8], &str); 2] = [
            (
                b"",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc",
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
        ];
        for (content, expected) in cases {
            let (_dir, path) = file_with(content);
            let f = fingerprint(&path, |_, _| {}, &no_cancel()).unwrap();
            assert_eq!(f.sha256.as_str(), expected);
            assert_eq!(f.size_bytes, content.len() as u64);
            assert_eq!(f.file_name, "preuve.log");
            assert!(f.path.is_absolute());
            assert!(f.modified_at.is_some());
        }
    }

    #[test]
    fn fichier_plus_grand_qu_un_bloc_et_progression() {
        let content = vec![0x5au8; CHUNK * 2 + 123];
        let (_dir, path) = file_with(&content);
        let mut calls = Vec::new();
        let f = fingerprint(&path, |read, total| calls.push((read, total)), &no_cancel()).unwrap();
        let expected: [u8; 32] = Sha256::digest(&content).into();
        assert_eq!(f.sha256, Sha256Digest::from_bytes(expected));
        assert_eq!(
            calls.last(),
            Some(&(content.len() as u64, content.len() as u64))
        );
        assert!(calls.len() >= 3);
    }

    #[test]
    fn le_fichier_n_est_pas_modifie() {
        let (_dir, path) = file_with(b"contenu original");
        let before = std::fs::read(&path).unwrap();
        let modified_before = path.metadata().unwrap().modified().unwrap();
        fingerprint(&path, |_, _| {}, &no_cancel()).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert_eq!(
            path.metadata().unwrap().modified().unwrap(),
            modified_before
        );
    }

    #[test]
    fn annulation() {
        let (_dir, path) = file_with(b"x");
        let cancel = AtomicBool::new(true);
        assert!(matches!(
            fingerprint(&path, |_, _| {}, &cancel),
            Err(EvidenceError::Cancelled)
        ));
    }

    #[test]
    fn erreurs_explicites() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            fingerprint(&dir.path().join("absent"), |_, _| {}, &no_cancel()),
            Err(EvidenceError::NotFound(_))
        ));
        assert!(matches!(
            fingerprint(dir.path(), |_, _| {}, &no_cancel()),
            Err(EvidenceError::NotAFile(_))
        ));
    }

    #[test]
    fn verification_d_integrite() {
        let (_dir, path) = file_with(b"avant");
        let recorded = fingerprint(&path, |_, _| {}, &no_cancel()).unwrap().sha256;
        assert_eq!(
            current_digest(&path, &no_cancel()).unwrap(),
            Some(recorded.clone())
        );
        std::fs::write(&path, b"apres").unwrap();
        assert_ne!(current_digest(&path, &no_cancel()).unwrap(), Some(recorded));
        std::fs::remove_file(&path).unwrap();
        assert_eq!(current_digest(&path, &no_cancel()).unwrap(), None);
    }
}
