use ens_normalize_rs::Processor;
use lazy_static::lazy_static;
use rayon::prelude::*;
use rstest::rstest;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Entry {
    VersionInfo {
        name: String,
        validated: String,
        built: String,
        cldr: String,
        derived: String,
        ens_hash_base64: String,
        nf_hash_base64: String,
        spec_hash: String,
        unicode: String,
        version: String,
    },
    TestCase(TestCase),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TestCase {
    name: String,
    comment: Option<String>,
    #[serde(default)]
    error: bool,
    norm: Option<String>,
}

pub type IndexedTestCase<'a> = (usize, &'a TestCase);

lazy_static! {
    pub static ref ENS_TESTS: Vec<Entry> =
        serde_json::from_str(include_str!("ens_cases.json")).unwrap();
}

#[rstest]
fn ens_tests() {
    test_cases_parallel(&only_cases(&ENS_TESTS))
}

fn test_cases_parallel(cases: &[IndexedTestCase]) {
    let processor = Processor::default();
    let results = cases
        .par_iter() // Parallel iterator from Rayon
        .map(|(i, test_case)| (i, process_test_case(&processor, test_case)))
        .filter_map(|(i, r)| r.err().map(|e| (i, e)))
        .collect::<Vec<_>>();

    if !results.is_empty() {
        let info = results
            .iter()
            .map(|(i, e)| format!("{}: {}", i, e))
            .collect::<Vec<_>>()
            .join("\n");
        panic!("{} cases failed:\n{}", results.len(), info);
    }
}

fn process_test_case(processor: &Processor, case: &TestCase) -> Result<(), anyhow::Error> {
    let test_name = match (case.comment.as_ref(), case.name.as_str()) {
        (Some(comment), name) if name.len() < 64 => format!("{comment} (`{name}`)"),
        (Some(comment), _) => comment.clone(),
        (None, name) => name.to_string(),
    };
    let result = processor.process(&case.name);

    match result {
        Err(_e) if case.error => (),
        Ok(processed) if !case.error => {
            let actual = processed.normalize();
            if let Some(expected) = &case.norm {
                assert_eq!(
                    actual,
                    expected.to_string(),
                    "in test case '{test_name}': expected '{expected}', got '{actual}'"
                );
            } else {
                assert_eq!(
                    actual, case.name,
                    "in test case '{test_name}': expected '{}', got '{actual}'",
                    case.name
                );
            }
        }
        Err(e) => anyhow::bail!("in test case '{test_name}': expected no error, got {e}"),
        Ok(_) => anyhow::bail!("in test case '{test_name}': expected error, got success"),
    }

    Ok(())
}

fn only_cases(entries: &[Entry]) -> Vec<IndexedTestCase> {
    entries
        .iter()
        .filter_map(|e| match e {
            Entry::TestCase(t) => Some(t),
            _ => None,
        })
        .enumerate()
        .collect()
}
