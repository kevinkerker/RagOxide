//! The checked-in TypeScript bindings must match the Rust types.

use std::path::Path;

#[test]
fn bindings_are_up_to_date() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/api/bindings.ts");
    let generated = ragoxide_app::generate_bindings();
    if std::env::var_os("UPDATE_BINDINGS").is_some() {
        std::fs::write(&path, &generated).expect("write ui/api/bindings.ts");
        return;
    }
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        current == generated,
        "ui/api/bindings.ts is outdated; run `pnpm gen:types` in ui/ (or \
         `UPDATE_BINDINGS=1 cargo test -p ragoxide-app --test bindings`) and check in the result"
    );
}

/// `ts-rs` does not generate constants; the copies in `ui/api/backend.ts` must match.
#[test]
fn constants_copied_to_the_user_interface_match() {
    use kb_core::ModelRole;
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/api/backend.ts");
    let source = std::fs::read_to_string(&path).expect("read ui/api/backend.ts");
    let name = |role: ModelRole| serde_json::to_string(&role).unwrap();
    let start = source
        .find("PROVIDE_ORDER: readonly ModelRole[] = [")
        .expect("PROVIDE_ORDER in ui/api/backend.ts");
    let end = start + source[start..].find("];").expect("end of PROVIDE_ORDER");
    let listed: Vec<String> = source[start..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(|role| format!("\"{role}\""))
        .collect();
    let expected: Vec<String> = ModelRole::PROVIDE_ORDER.into_iter().map(name).collect();
    assert_eq!(listed, expected, "PROVIDE_ORDER in ui/api/backend.ts");

    let start = source
        .find("ANSWER_LEVELS: readonly AnswerLevel[] = [")
        .expect("ANSWER_LEVELS in ui/api/backend.ts");
    let end = start + source[start..].find("];").expect("end of ANSWER_LEVELS");
    let listed: Vec<String> = source[start..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(|level| format!("\"{level}\""))
        .collect();
    let expected: Vec<String> = kb_core::AnswerLevel::ALL
        .into_iter()
        .map(|level| serde_json::to_string(&level).unwrap())
        .collect();
    assert_eq!(listed, expected, "ANSWER_LEVELS in ui/api/backend.ts");

    let start = source
        .find("export function isNeededForIngestion")
        .expect("isNeededForIngestion in ui/api/backend.ts");
    let body = &source[start..start + source[start..].find('}').unwrap()];
    for role in ModelRole::PROVIDE_ORDER {
        // The copy names exactly the roles ingestion does not need.
        assert_eq!(
            body.contains(&name(role)),
            !role.is_needed_for_ingestion(),
            "isNeededForIngestion in ui/api/backend.ts for {role:?}"
        );
    }
}
