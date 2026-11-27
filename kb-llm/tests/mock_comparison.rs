//! F37: the same sequence with `MockGenerator` and with `LlamaGenerator` on the stand-in
//! server. Equal: when `StartingModel` comes, error categories, valid sequences, a crash
//! ending with `Err(Generator)`. The other differences are listed in the README.

mod common;

use common::*;
use kb_core::mock::MockGenerator;
use kb_core::{
    AnswerLevel, CancellationToken, ErrorKind, GenerationEvent, Generator, HardwareTier,
};
use serde_json::json;

const REPLY: &str = "Hallo Welt, das ist eine Antwort.";

/// What the comparison looks at in one generation.
#[derive(Debug, PartialEq)]
struct Outcome {
    starting_model: bool,
    valid: bool,
    error: Option<ErrorKind>,
    text: String,
}

async fn outcome(generator: &dyn Generator, level: AnswerLevel) -> Outcome {
    let (events, error) = run(generator, request(level, "Was steht in den Quellen?")).await;
    outcome_of(&events, error.map(|e| e.kind()))
}

fn outcome_of(events: &[GenerationEvent], error: Option<ErrorKind>) -> Outcome {
    Outcome {
        starting_model: starts_with_starting_model(events),
        valid: error.is_some() || kb_core::mock::generation_order_violation(events).is_none(),
        error,
        text: text(events),
    }
}

/// The sequence of F37 on one generator.
async fn sequence(generator: &dyn Generator) -> Vec<Outcome> {
    let mut outcomes = Vec::new();
    outcomes.push(outcome(generator, AnswerLevel::Fast).await);
    outcomes.push(outcome(generator, AnswerLevel::Fast).await);
    let count = generator
        .count_tokens(AnswerLevel::Balanced, "eins zwei drei")
        .await;
    outcomes.push(Outcome {
        starting_model: false,
        valid: true,
        error: count.as_ref().err().map(|e| e.kind()),
        text: format!("{:?}", count.ok()),
    });
    outcomes.push(outcome(generator, AnswerLevel::Balanced).await);
    outcomes.push(outcome(generator, AnswerLevel::Fast).await);
    // A token cancelled before the stream is read.
    let cancel = CancellationToken::new();
    cancel.cancel();
    let stream = generator
        .generate(request(AnswerLevel::Fast, "x"), cancel)
        .await
        .unwrap();
    let (events, error) = collect(stream).await;
    outcomes.push(outcome_of(&events, error.map(|e| e.kind())));
    outcomes
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn package_and_mock_agree() {
    let setup = Setup::new();
    setup.control(json!({"reply": REPLY}));
    let package = setup.generator(HardwareTier::Supported);
    let mock = MockGenerator::new(REPLY);
    let expected = sequence(&mock).await;
    let actual = sequence(&package).await;
    assert_eq!(actual, expected);
    assert_eq!(
        expected
            .iter()
            .map(|o| o.starting_model)
            .collect::<Vec<_>>(),
        [true, false, false, false, true, false]
    );

    // check_level: a level the tier does not offer, a level without its model file.
    let below = setup.generator(HardwareTier::BelowMinimum);
    let mock_below = MockGenerator::new(REPLY).offering(&[AnswerLevel::Fast]);
    assert_eq!(
        below.check_level(AnswerLevel::Balanced).unwrap_err().kind(),
        mock_below
            .check_level(AnswerLevel::Balanced)
            .unwrap_err()
            .kind()
    );
    let missing = Setup::new();
    std::fs::remove_file(&missing.level_files(HardwareTier::Supported, AnswerLevel::Thorough)[0])
        .unwrap();
    let package_missing = missing.generator(HardwareTier::Supported);
    let mock_missing = MockGenerator::new(REPLY).without_model(AnswerLevel::Thorough);
    for level in AnswerLevel::ALL {
        assert_eq!(
            package_missing.check_level(level).map_err(|e| e.kind()),
            mock_missing.check_level(level).map_err(|e| e.kind()),
            "{level:?}"
        );
    }
}

/// A crash after three words ends with `Err(Generator)` after the same text.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_crash_after_three_words_agrees_with_the_mock() {
    let setup = Setup::new();
    setup.control(json!({"reply": REPLY, "crash_after": 3}));
    let package = setup.generator(HardwareTier::Supported);
    let mock = MockGenerator::failing_after(REPLY, 3);
    let expected = outcome(&mock, AnswerLevel::Fast).await;
    let actual = outcome(&package, AnswerLevel::Fast).await;
    assert_eq!(actual, expected);
    assert_eq!(expected.error, Some(ErrorKind::Generator));
    assert_eq!(expected.text, "Hallo Welt, das ");
}
