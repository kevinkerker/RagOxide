//! The word pairs of step 1 as expected results (E58, covering E37, E38, E56, ß/ss,
//! case and the negative probes).

mod common;

use common::*;
use kb_core::{Index, IndexEntry, SearchMode};

const DIM: u32 = 64;

/// `(group, query, text, expected)`: `expected` says whether the text is among the
/// hits of the query.
const PAIRS: &[(&str, &str, &str, bool)] = &[
    ("E37 Numerus", "Krankenhaus", "Krankenhäuser", true),
    ("E37 Numerus", "Krankenhäuser", "Krankenhaus", true),
    ("E37 Numerus", "Vertrag", "Verträge", true),
    ("E37 Numerus", "Verträge", "Vertrag", true),
    ("E37 Genitiv", "Bundestag", "Bundestages", true),
    ("E37 Genitiv", "Bundestages", "Bundestag", true),
    ("E37 Genitiv", "Grundgesetz", "Grundgesetzes", true),
    ("E37 Genitiv", "Grundgesetzes", "Grundgesetz", true),
    ("E37 Adjektiv", "Abgeordnete", "Abgeordneten", true),
    ("E37 Adjektiv", "Abgeordneten", "Abgeordnete", true),
    ("E37 Adjektiv", "zuständige", "zuständigen", true),
    ("E37 Adjektiv", "zuständigen", "zuständige", true),
    ("E38 A", "Kanzler", "Bundeskanzler", true),
    ("E38 A", "Mehrwertsteuer", "Mehrwertsteuersystem", true),
    ("E38 A", "Kohle", "Steinkohle", true),
    ("E38 B", "Bundestagsabgeordnete", "Abgeordneter", true),
    ("E38 B", "Bundestagspräsident", "Präsident", true),
    (
        "E38 B",
        "Krankenhausbetten",
        "Betten in Krankenhäusern",
        true,
    ),
    ("ß/ss", "Giesserei", "Gießerei", true),
    ("ß/ss", "Straße", "Strasse", true),
    ("E56 Partizip", "gewählt", "Wahl", true),
    ("E56 Partizip", "geprüft", "prüfen", true),
    ("E56 Partizip", "gekündigt", "kündigen", true),
    ("Groß/klein", "ÄRZTE", "Ärzte", true),
    ("Groß/klein", "gießereikoks", "Gießereikoks", true),
    // Known false hit: the word list has `Bun-des=tag` as a compound (ADR 0130).
    (
        "Negativprobe, bekannter Fehltreffer",
        "Tag",
        "Bundestag",
        true,
    ),
    ("Negativprobe", "Wahl", "Auswahl", false),
    ("Negativprobe", "Amt", "insgesamt", false),
    ("Negativprobe", "Rat", "Heirat", false),
    ("Negativprobe", "Ende", "Dividende", false),
];

#[tokio::test]
async fn every_word_pair_of_step_one_behaves_as_measured() {
    let (index, dir) = open("e58", DIM).await;
    let entries: Vec<IndexEntry> = PAIRS
        .iter()
        .enumerate()
        .map(|(i, (_, _, text, _))| {
            entry(
                chunk(&format!("pair-{i:02}"), 0, text, Some("de")),
                DIM,
                i as u64,
            )
        })
        .collect();
    index.add(&entries).await.unwrap();
    let mut wrong = Vec::new();
    for (i, (group, query, text, expected)) in PAIRS.iter().enumerate() {
        let hits = index
            .search(&request(query, u32::MAX, SearchMode::FullText), &[])
            .await
            .unwrap();
        let found = ids(&hits).contains(&format!("pair-{i:02}#0"));
        if found != *expected {
            wrong.push(format!(
                "{group}: {query} -> {text}: found {found}, expected {expected}"
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    drop(index);
    remove(dir);
}
