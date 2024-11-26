use crate::{
    code_points::constants, static_data::spec_json, utils, CodePoint, CodePointsSpecs,
    CurrableError, DisallowedSequence, ParsedGroup, ParsedWholeValue, ProcessError, TokenizedLabel,
};
use std::collections::HashSet;

pub type LabelType = spec_json::GroupName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedLabel {
    pub tokenized: TokenizedLabel,
    pub label_type: LabelType,
}

// https://docs.ens.domains/ensip/15#validate
pub fn validate_label(
    label: TokenizedLabel,
    specs: &CodePointsSpecs,
) -> Result<ValidatedLabel, ProcessError> {
    non_empty(&label)?;
    if label.is_fully_emoji() {
        return Ok(ValidatedLabel {
            tokenized: label.clone(),
            label_type: LabelType::Emoji,
        });
    };
    underscore_only_at_beginning(&label)?;
    if label.is_fully_ascii() {
        no_hyphen_at_second_and_third(&label)?;
        return Ok(ValidatedLabel {
            tokenized: label.clone(),
            label_type: LabelType::Ascii,
        });
    }
    check_fenced(&label, specs)?;
    check_cm_leading_emoji(&label, specs)?;
    let group = check_and_get_group(&label, specs)?;
    Ok(ValidatedLabel {
        tokenized: label,
        label_type: group.name,
    })
}

fn non_empty(label: &TokenizedLabel) -> Result<(), ProcessError> {
    let non_ignored_token_exists = label.tokens.iter().any(|label| !label.ignored());
    if !non_ignored_token_exists {
        return Err(ProcessError::DisallowedSequence(
            DisallowedSequence::EmptyLabel,
        ));
    }
    Ok(())
}

fn underscore_only_at_beginning(label: &TokenizedLabel) -> Result<(), ProcessError> {
    let leading_underscores = label
        .cps
        .iter()
        .take_while(|cp| **cp == constants::CP_UNDERSCORE)
        .count();
    let underscore_in_middle = label
        .cps
        .iter()
        .enumerate()
        .skip(leading_underscores)
        .find(|(_, cp)| **cp == constants::CP_UNDERSCORE);
    if let Some((index, _)) = underscore_in_middle {
        return Err(ProcessError::CurrableError {
            inner: CurrableError::UnderscoreInMiddle,
            index,
            sequence: utils::cps2str(&[constants::CP_UNDERSCORE]),
            maybe_suggest: Some("".to_string()),
        });
    }
    Ok(())
}

// The 3rd and 4th characters must not both be 2D (-) HYPHEN-MINUS.
// Must not match /^..--/
// Examples: "ab-c" and "---a"are valid, "xn--" and ---- are invalid.
fn no_hyphen_at_second_and_third(label: &TokenizedLabel) -> Result<(), ProcessError> {
    if label.cps.get(2) == Some(&constants::CP_HYPHEN)
        && label.cps.get(3) == Some(&constants::CP_HYPHEN)
    {
        return Err(ProcessError::CurrableError {
            inner: CurrableError::HyphenAtSecondAndThird,
            index: 2,
            sequence: utils::cps2str(&[constants::CP_HYPHEN, constants::CP_HYPHEN]),
            maybe_suggest: Some("".to_string()),
        });
    }
    Ok(())
}

fn check_fenced(label: &TokenizedLabel, specs: &CodePointsSpecs) -> Result<(), ProcessError> {
    if let Some(first_cp) = label.cps.first() {
        if specs.is_fenced(*first_cp) {
            return Err(ProcessError::CurrableError {
                inner: CurrableError::FencedLeading,
                index: 0,
                sequence: utils::cps2str(&[*first_cp]),
                maybe_suggest: Some("".to_string()),
            });
        }
    }
    if let Some(last_cp) = label.cps.last() {
        if specs.is_fenced(*last_cp) {
            return Err(ProcessError::CurrableError {
                inner: CurrableError::FencedTrailing,
                index: label.cps.len() - 1,
                sequence: utils::cps2str(&[*last_cp]),
                maybe_suggest: Some("".to_string()),
            });
        }
    }

    for (i, window) in label.cps.windows(2).enumerate() {
        let (one, two) = (window[0], window[1]);
        if specs.is_fenced(one) && specs.is_fenced(two) {
            return Err(ProcessError::CurrableError {
                inner: CurrableError::FencedConsecutive,
                index: i,
                sequence: utils::cps2str(&[one, two]),
                maybe_suggest: Some("".to_string()),
            });
        }
    }
    Ok(())
}

fn check_cm_leading_emoji(
    label: &TokenizedLabel,
    specs: &CodePointsSpecs,
) -> Result<(), ProcessError> {
    let mut index = 0;

    for (i, token) in label.tokens.iter().enumerate() {
        if token.is_text() {
            if let Some(cp) = token.cps().first() {
                if specs.is_cm(*cp) {
                    if i == 0 {
                        return Err(ProcessError::CurrableError {
                            inner: CurrableError::CmStart,
                            index,
                            sequence: utils::cps2str(&[*cp]),
                            maybe_suggest: Some("".to_string()),
                        });
                    } else if label.tokens[i - 1].is_emoji() {
                        return Err(ProcessError::CurrableError {
                            inner: CurrableError::CmAfterEmoji,
                            index,
                            sequence: utils::cps2str(&[*cp]),
                            maybe_suggest: Some("".to_string()),
                        });
                    }
                }
            }
        }
        index += token.size();
    }

    Ok(())
}

fn check_and_get_group(
    label: &TokenizedLabel,
    specs: &CodePointsSpecs,
) -> Result<ParsedGroup, ProcessError> {
    let cps = label.get_only_text_cps();
    let unique_cps = cps
        .clone()
        .into_iter()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let group = determine_group(&unique_cps, specs).cloned()?;
    check_group(&group, &cps, specs)?;
    check_whole(&group, &unique_cps, specs)?;
    Ok(group)
}

fn check_group(
    group: &ParsedGroup,
    cps: &[CodePoint],
    specs: &CodePointsSpecs,
) -> Result<(), ProcessError> {
    for cp in cps.iter() {
        if !group.contains_cp(*cp) {
            return Err(ProcessError::Confused(format!(
                "symbol {} not present in group {}",
                utils::cp2str(*cp),
                group.name
            )));
        }
    }
    if group.cm_absent {
        let decomposed = utils::nfd_cps(cps, specs);
        let mut i = 1;
        let e = decomposed.len();
        while i < e {
            if specs.is_nsm(decomposed[i]) {
                let mut j = i + 1;
                while j < e && specs.is_nsm(decomposed[j]) {
                    if j - i + 1 > specs.nsm_max() as usize {
                        return Err(ProcessError::DisallowedSequence(
                            DisallowedSequence::NsmTooMany,
                        ));
                    }
                    for k in i..j {
                        if decomposed[k] == decomposed[j] {
                            return Err(ProcessError::DisallowedSequence(
                                DisallowedSequence::NsmRepeated,
                            ));
                        }
                    }
                    j += 1;
                }
                i = j;
            }
            i += 1;
        }
    }
    Ok(())
}

fn check_whole(
    group: &ParsedGroup,
    unique_cps: &[CodePoint],
    specs: &CodePointsSpecs,
) -> Result<(), ProcessError> {
    let (maker, shared) = get_groups_candidates_and_shared_cps(unique_cps, specs);
    for group_name in maker {
        let confused_group_candidate = specs.group_by_name(group_name).expect("group must exist");
        if confused_group_candidate.contains_all_cps(&shared) {
            return Err(ProcessError::ConfusedGroups {
                group1: group.name.to_string(),
                group2: confused_group_candidate.name.to_string(),
            });
        }
    }
    Ok(())
}

fn get_groups_candidates_and_shared_cps(
    unique_cps: &[CodePoint],
    specs: &CodePointsSpecs,
) -> (Vec<String>, Vec<CodePoint>) {
    let mut maybe_groups: Option<Vec<String>> = None;
    let mut shared: Vec<CodePoint> = Vec::new();

    for cp in unique_cps {
        match specs.whole_map(*cp) {
            Some(ParsedWholeValue::Number(_)) => {
                return (vec![], vec![]);
            }
            Some(ParsedWholeValue::WholeObject(whole)) => {
                let confused_groups_names = whole
                    .m
                    .get(cp)
                    .expect("since we got `whole` from cp, `M` must have a value for `cp`");

                match maybe_groups.as_mut() {
                    Some(groups) => {
                        groups.retain(|g| confused_groups_names.contains(g));
                    }
                    None => {
                        maybe_groups = Some(confused_groups_names.iter().cloned().collect());
                    }
                }
            }
            None => {
                shared.push(*cp);
            }
        };
    }

    (maybe_groups.unwrap_or_default(), shared)
}

fn determine_group<'a>(
    unique_cps: &'a [CodePoint],
    specs: &'a CodePointsSpecs,
) -> Result<&'a ParsedGroup, ProcessError> {
    specs
        .groups_for_cps(unique_cps)
        .next()
        .ok_or(ProcessError::Confused(format!(
            "no group found for {:?}",
            unique_cps
        )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize_label;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    fn specs() -> CodePointsSpecs {
        CodePointsSpecs::default()
    }

    #[rstest]
    // success
    #[case::hello("hello", Ok(LabelType::Ascii))]
    #[case::latin("E︎̃", Ok(LabelType::Other("Latin".to_string())))]
    #[case::cyrillic("всем-привет", Ok(LabelType::Other("Cyrillic".to_string())))]
    #[case::with_fenced_in_middle("a・a’s", Ok(LabelType::Other("Han".to_string())))]
    #[case::ascii_with_hyphen("ab-c", Ok(LabelType::Ascii))]
    // errors
    #[case::hyphen_at_second_and_third("ab--", Err(ProcessError::CurrableError {
        inner: CurrableError::HyphenAtSecondAndThird,
        index: 2,
        sequence: "--".to_string(),
        maybe_suggest: Some("".to_string())
    }))]
    #[case::fenced_leading("’85", Err(ProcessError::CurrableError {
        inner: CurrableError::FencedLeading,
        index: 0,
        sequence: "’".to_string(),
        maybe_suggest: Some("".to_string())
    }))]
    #[case::fenced_contiguous("a・・a", Err(ProcessError::CurrableError {
        inner: CurrableError::FencedConsecutive,
        index: 1,
        sequence: "・・".to_string(),
        maybe_suggest: Some("".to_string())
    }))]
    #[case::cm_after_emoji("😎😎😎😎😎😎😎😎\u{300}hello", Err(ProcessError::CurrableError {
        inner: CurrableError::CmAfterEmoji,
        index: 8,
        sequence: "\u{300}".to_string(),
        maybe_suggest: Some("".to_string())
    }))]
    #[case::cm_leading("\u{300}hello", Err(ProcessError::CurrableError {
        inner: CurrableError::CmStart,
        index: 0,
        sequence: "\u{300}".to_string(),
        maybe_suggest: Some("".to_string())
    }))]
    fn test_validate_and_get_type(
        #[case] input: &str,
        #[case] expected: Result<LabelType, ProcessError>,
        specs: &CodePointsSpecs,
    ) {
        let label = tokenize_label(input, specs, true).unwrap();
        let result = validate_label(label, specs);
        assert_eq!(
            result.clone().map(|v| v.label_type),
            expected,
            "{:?}",
            result
        );
    }

    #[rstest]
    #[case::emoji("\"Emoji\"", LabelType::Emoji)]
    #[case::ascii("\"ASCII\"", LabelType::Ascii)]
    #[case::greek("\"Greek\"", LabelType::Greek)]
    #[case::other("\"FooBar\"", LabelType::Other("FooBar".to_string()))]
    fn test_deserialize_label_type(#[case] input: &str, #[case] expected: LabelType) {
        let result: LabelType = serde_json::from_str(input).unwrap();
        assert_eq!(result, expected);
    }
}
