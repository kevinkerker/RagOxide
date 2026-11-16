//! Rules of the library scope in `Settings` (ADR 0025): registering folders and
//! excluding files and subfolders.

use std::path::{Path, PathBuf};

use kb_core::*;

fn p(path: &str) -> PathBuf {
    PathBuf::from(path)
}

fn watching(folders: &[&str]) -> Settings {
    let mut settings = Settings::default();
    for folder in folders {
        settings.add_folder(Path::new(folder)).unwrap();
    }
    settings
}

#[cfg(windows)]
#[test]
fn folders_are_stored_canonical_and_once() {
    let mut settings = Settings::default();
    settings.add_folder(Path::new("c:/Ablage/")).unwrap();
    settings.add_folder(Path::new(r"C:\ABLAGE")).unwrap();
    assert_eq!(settings.watched_folders, [p(r"C:\Ablage")]);
    assert!(matches!(
        settings.add_folder(Path::new("Ablage")),
        Err(Error::InvalidInput(_))
    ));
    settings.remove_folder(Path::new(r"c:\ablage")).unwrap();
    assert!(settings.watched_folders.is_empty());
    settings
        .remove_folder(Path::new(r"C:\Gibt\Es\Nicht"))
        .unwrap();
}

#[cfg(windows)]
#[test]
fn scope_is_folders_minus_exclusions() {
    let mut settings = watching(&[r"C:\Ablage"]);
    settings.exclude(Path::new(r"C:\Ablage\Privat")).unwrap();
    settings
        .exclude(Path::new(r"C:\Ablage\budget.xlsx"))
        .unwrap();
    assert!(settings.in_scope(Path::new(r"c:\ablage\faq.html")));
    assert!(
        settings.in_scope(Path::new(r"C:\Ablage\Privatsache.md")),
        "whole components"
    );
    assert!(!settings.in_scope(Path::new(r"C:\Ablage\Privat\Steuer\a.pdf")));
    assert!(!settings.in_scope(Path::new(r"C:\Ablage\BUDGET.xlsx")));
    assert!(!settings.in_scope(Path::new(r"D:\Anderswo\a.pdf")));
}

#[cfg(windows)]
#[test]
fn exclusions_must_lie_strictly_inside_a_watched_folder() {
    let mut settings = watching(&[r"C:\Ablage"]);
    let outside = settings.exclude(Path::new(r"D:\Anderswo\a.pdf"));
    assert!(matches!(outside, Err(Error::InvalidInput(_))));
    let folder_itself = settings.exclude(Path::new(r"C:\Ablage"));
    assert!(matches!(folder_itself, Err(Error::InvalidInput(_))));
    assert!(settings.excluded.is_empty());
}

#[cfg(windows)]
#[test]
fn the_exclusion_list_stays_minimal() {
    let mut settings = watching(&[r"C:\Ablage"]);
    settings
        .exclude(Path::new(r"C:\Ablage\Privat\a.pdf"))
        .unwrap();
    settings
        .exclude(Path::new(r"C:\Ablage\Privat\b.pdf"))
        .unwrap();
    // A subfolder covers the entries below it.
    settings.exclude(Path::new(r"C:\Ablage\Privat")).unwrap();
    assert_eq!(settings.excluded, [p(r"C:\Ablage\Privat")]);
    // Already covered: no new entry.
    settings
        .exclude(Path::new(r"c:\ablage\privat\c.pdf"))
        .unwrap();
    assert_eq!(settings.excluded, [p(r"C:\Ablage\Privat")]);
    // Taking an entry off the list.
    settings.include(Path::new(r"C:\ABLAGE\PRIVAT")).unwrap();
    assert!(settings.excluded.is_empty());
    settings.include(Path::new(r"C:\Ablage\nie.pdf")).unwrap();
}

#[cfg(windows)]
#[test]
fn deregistering_a_folder_drops_its_exclusions_only() {
    let mut settings = watching(&[r"C:\Ablage", r"C:\Ablage\Archiv", r"D:\Projekte"]);
    settings.exclude(Path::new(r"C:\Ablage\Privat")).unwrap();
    settings
        .exclude(Path::new(r"C:\Ablage\Archiv\alt.pdf"))
        .unwrap();
    settings.exclude(Path::new(r"D:\Projekte\tmp")).unwrap();
    settings.remove_folder(Path::new(r"C:\Ablage")).unwrap();
    // C:\Ablage\Archiv is still watched, so its exclusion stays.
    assert_eq!(
        settings.excluded,
        [p(r"C:\Ablage\Archiv\alt.pdf"), p(r"D:\Projekte\tmp")]
    );
}

#[cfg(windows)]
#[test]
fn registering_an_excluded_subfolder_brings_it_back_into_scope() {
    let mut settings = watching(&[r"C:\Ablage"]);
    settings.exclude(Path::new(r"C:\Ablage\Archiv")).unwrap();
    settings.add_folder(Path::new(r"C:\Ablage\Archiv")).unwrap();
    assert!(settings.excluded.is_empty());
    assert!(settings.in_scope(Path::new(r"C:\Ablage\Archiv\a.pdf")));
}

#[cfg(windows)]
#[test]
fn a_folder_below_an_exclusion_is_rejected_until_the_exclusion_is_taken_off() {
    let mut settings = watching(&[r"C:\Ablage"]);
    settings.exclude(Path::new(r"C:\Ablage\Privat")).unwrap();
    let below = settings.add_folder(Path::new(r"C:\Ablage\Privat\Steuer"));
    assert!(matches!(below, Err(Error::InvalidInput(_))));
    assert_eq!(
        settings.excluded,
        [p(r"C:\Ablage\Privat")],
        "siblings stay excluded"
    );
    assert_eq!(settings.watched_folders, [p(r"C:\Ablage")]);
    settings.include(Path::new(r"C:\Ablage\Privat")).unwrap();
    settings
        .add_folder(Path::new(r"C:\Ablage\Privat\Steuer"))
        .unwrap();
    assert_eq!(settings.watched_folders.len(), 2);
}

#[cfg(windows)]
#[test]
fn a_watched_folder_itself_is_not_in_scope() {
    let settings = watching(&[r"C:\Ablageaq.html"]);
    assert!(!settings.in_scope(Path::new(r"C:\Ablageaq.html")));
    assert!(settings.in_scope(Path::new(r"C:\Ablageaq.html\inner.md")));
}
