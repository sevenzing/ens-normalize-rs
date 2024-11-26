use super::TokenDisallowed;
use crate::{
    code_points::constants,
    tokens::{
        EnsNameToken, TokenEmoji, TokenIgnored, TokenMapped, TokenNfc, TokenStop, TokenValid,
    },
    utils, CodePoint, CodePointsSpecs, DisallowedSequence, ProcessError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedName {
    pub input: String,
    pub labels: Vec<TokenizedLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedLabel {
    pub input: String,
    pub tokens: Vec<EnsNameToken>,
    pub cps: Vec<CodePoint>,
}

pub fn tokenize_name(
    name: impl AsRef<str>,
    specs: &CodePointsSpecs,
    apply_nfc: bool,
) -> Result<TokenizedName, ProcessError> {
    let name = name.as_ref();
    if name.is_empty() {
        return Ok(TokenizedName {
            input: name.to_string(),
            labels: vec![],
        });
    }
    let labels = name
        .split(constants::CP_STOP as u8 as char)
        .map(|label| tokenize_label(label, specs, apply_nfc))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TokenizedName {
        input: name.to_string(),
        labels,
    })
}

pub fn tokenize_label(
    label: impl AsRef<str>,
    specs: &CodePointsSpecs,
    apply_nfc: bool,
) -> Result<TokenizedLabel, ProcessError> {
    let label = label.as_ref();
    let chars = label.chars().collect::<Vec<_>>();

    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if let Some(emoji) = maybe_starts_with_emoji(&chars[i..], specs) {
            let cps_taken = emoji.input.len();
            tokens.push(EnsNameToken::Emoji(emoji));
            i += cps_taken;
        } else {
            let token = process_one_cp(chars[i] as CodePoint, specs);
            if let EnsNameToken::Disallowed(t) = token {
                return Err(ProcessError::DisallowedSequence(
                    DisallowedSequence::Invalid(utils::cp2str(t.cp)),
                ));
            }
            tokens.push(token);
            i += 1;
        }
    }

    if apply_nfc {
        perform_nfc_transform(&mut tokens, specs);
    }
    collapse_valid_tokens(&mut tokens);
    let cps = tokens
        .iter()
        .flat_map(|token| token.cps())
        .collect::<Vec<_>>();
    Ok(TokenizedLabel {
        input: label.to_string(),
        tokens,
        cps,
    })
}

fn perform_nfc_transform(tokens: &mut Vec<EnsNameToken>, specs: &CodePointsSpecs) {
    let mut i = 0;
    let mut start = -1i32;

    while i < tokens.len() {
        let token = &tokens[i];
        match token {
            EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) => {
                let cps = token.cps();
                if specs.cps_requires_check(&cps) {
                    let mut end = i + 1;
                    for (pos, token) in tokens.iter().enumerate().skip(end) {
                        match token {
                            EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) => {
                                if !specs.cps_requires_check(&cps) {
                                    break;
                                }
                                end = pos + 1;
                            }
                            EnsNameToken::Ignored(_) => {}
                            _ => break,
                        }
                    }

                    if start < 0 {
                        start = i as i32;
                    }

                    let slice = &tokens[start as usize..end];
                    let mut cps = Vec::new();
                    for tok in slice {
                        match tok {
                            EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) => {
                                cps.extend(&tok.cps());
                            }
                            _ => {}
                        }
                    }

                    let str0 = utils::cps2str(&cps);
                    let str = utils::nfc(&str0);

                    if str0 == str {
                        i = end - 1;
                    } else {
                        let new_token = EnsNameToken::Nfc(TokenNfc {
                            input: cps,
                            cps: utils::str2cps(&str),
                        });
                        tokens.splice(start as usize..end, vec![new_token]);
                        i = start as usize;
                    }
                    start = -1;
                } else {
                    start = i as i32;
                }
            }
            EnsNameToken::Ignored(_) => {}
            _ => {
                start = -1;
            }
        }
        i += 1;
    }
}

// given array of codepoints
// returns the longest valid emoji sequence (or undefined if no match)
fn maybe_starts_with_emoji(chars: &[char], specs: &CodePointsSpecs) -> Option<TokenEmoji> {
    let mut longest_match = None;
    let cps: Vec<CodePoint> = chars.iter().map(|c| *c as u32).collect();

    for i in 1..=chars.len() {
        let candidate = &cps[0..i];
        if specs.cps_is_emoji(candidate) {
            let input = candidate.to_vec();
            let cps_no_fe0f = utils::filter_fe0f(&input);
            let emoji = specs.cps_emoji_no_fe0f_to_pretty(&cps_no_fe0f);

            longest_match = Some(TokenEmoji {
                input,
                emoji,
                cps: cps_no_fe0f,
            });
        }
    }
    longest_match
}

fn process_one_cp(cp: CodePoint, specs: &CodePointsSpecs) -> EnsNameToken {
    if specs.is_stop(cp) {
        EnsNameToken::Stop(TokenStop { cp })
    } else if specs.is_valid(cp) {
        EnsNameToken::Valid(TokenValid { cps: vec![cp] })
    } else if specs.is_ignored(cp) {
        EnsNameToken::Ignored(TokenIgnored { cp })
    } else if let Some(normalized) = specs.maybe_normalize(cp) {
        EnsNameToken::Mapped(TokenMapped {
            cp,
            cps: normalized.clone(),
        })
    } else {
        EnsNameToken::Disallowed(TokenDisallowed { cp })
    }
}

impl TokenizedLabel {
    pub fn is_fully_emoji(&self) -> bool {
        self.tokens
            .iter()
            .all(|t| matches!(t, EnsNameToken::Emoji(_)))
    }

    pub fn is_fully_ascii(&self) -> bool {
        self.tokens
            .iter()
            .all(|token| token.cps().into_iter().all(utils::is_ascii))
    }

    pub fn get_only_text_cps(&self) -> Vec<CodePoint> {
        self.tokens
            .iter()
            .filter_map(|token| {
                if token.is_text() {
                    Some(token.cps())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

fn collapse_valid_tokens(tokens: &mut Vec<EnsNameToken>) {
    let mut i = 0;
    while i < tokens.len() {
        if let EnsNameToken::Valid(token) = &tokens[i] {
            let mut j = i + 1;
            let mut cps = token.cps.clone();
            while j < tokens.len() {
                if let EnsNameToken::Valid(next_token) = &tokens[j] {
                    cps.extend(next_token.cps.iter());
                    j += 1;
                } else {
                    break;
                }
            }
            let new_token = EnsNameToken::Valid(TokenValid { cps });
            tokens.splice(i..j, vec![new_token].into_iter());
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    fn specs() -> CodePointsSpecs {
        CodePointsSpecs::default()
    }

    #[rstest]
    #[case::empty(vec![], vec![])]
    #[case::single(
        vec![EnsNameToken::Valid(TokenValid { cps: vec![1, 2, 3] })],
        vec![EnsNameToken::Valid(TokenValid { cps: vec![1, 2, 3] })],
    )]
    #[case::two(
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![1, 2, 3] }),
            EnsNameToken::Valid(TokenValid { cps: vec![4, 5, 6] }),
        ],
        vec![EnsNameToken::Valid(TokenValid { cps: vec![1, 2, 3, 4, 5, 6] })],
    )]
    #[case::full(
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![1, 2, 3] }),
            EnsNameToken::Disallowed(TokenDisallowed { cp: 0 }),
            EnsNameToken::Valid(TokenValid { cps: vec![4, 5, 6] }),
            EnsNameToken::Valid(TokenValid { cps: vec![7, 8, 9] }),
            EnsNameToken::Valid(TokenValid { cps: vec![10, 11, 12] }),
            EnsNameToken::Disallowed(TokenDisallowed { cp: 10 }),
            EnsNameToken::Stop(TokenStop { cp: 11 }),
            EnsNameToken::Valid(TokenValid { cps: vec![12] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 13 }),
        ],
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![1, 2, 3] }),
            EnsNameToken::Disallowed(TokenDisallowed { cp: 0 }),
            EnsNameToken::Valid(TokenValid { cps: vec![4, 5, 6, 7, 8, 9, 10, 11, 12] }),
            EnsNameToken::Disallowed(TokenDisallowed { cp: 10 }),
            EnsNameToken::Stop(TokenStop { cp: 11 }),
            EnsNameToken::Valid(TokenValid { cps: vec![12] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 13 }),
        ],
    )]
    fn test_collapse_valid_tokens(
        #[case] input: Vec<EnsNameToken>,
        #[case] expected: Vec<EnsNameToken>,
    ) {
        let mut tokens = input;
        collapse_valid_tokens(&mut tokens);
        assert_eq!(tokens, expected);
    }

    #[rstest]
    #[case::xyz(
        "xyz👨🏻",
        true,
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![120, 121, 122] }),
            EnsNameToken::Emoji(TokenEmoji { input: vec![128104, 127995], emoji: vec![128104, 127995], cps: vec![128104, 127995] }),
        ]
    )]
    #[case::a_poop_b(
        "A💩︎︎b",
        true,
        vec![
            EnsNameToken::Mapped(TokenMapped { cp: 65, cps: vec![97] }),
            EnsNameToken::Emoji(TokenEmoji { input: vec![128169], emoji: vec![128169, 65039], cps: vec![128169] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 65038 }),
            EnsNameToken::Ignored(TokenIgnored { cp: 65038 }),
            EnsNameToken::Valid(TokenValid { cps: vec![98] }),
        ]
    )]
    #[case::atm(
        "a™️",
        true,
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![97] }),
            EnsNameToken::Mapped(TokenMapped { cp: 8482, cps: vec![116, 109] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 65039 }),
        ]
    )]
    #[case::no_nfc(
        "_R💩\u{FE0F}a\u{FE0F}\u{304}\u{AD}.",
        false,
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![95] }),
            EnsNameToken::Mapped(TokenMapped { cp: 82, cps: vec![114] }),
            EnsNameToken::Emoji(TokenEmoji { input: vec![128169, 65039], emoji: vec![128169, 65039], cps: vec![128169] }),
            EnsNameToken::Valid(TokenValid { cps: vec![97] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 65039 }),
            EnsNameToken::Valid(TokenValid { cps: vec![772] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 173 }),
            EnsNameToken::Stop(TokenStop { cp: 46 }),
        ]
    )]
    #[case::with_nfc(
        "_R💩\u{FE0F}a\u{FE0F}\u{304}\u{AD}.",
        true,
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![95] }),
            EnsNameToken::Mapped(TokenMapped { cp: 82, cps: vec![114] }),
            EnsNameToken::Emoji(TokenEmoji { input: vec![128169, 65039], emoji: vec![128169, 65039], cps: vec![128169] }),
            EnsNameToken::Nfc(TokenNfc { input: vec![97, 772], cps: vec![257] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 173 }),
            EnsNameToken::Stop(TokenStop { cp: 46 }),
        ]
    )]
    #[case::raffy(
        "RaFFY🚴‍♂️.eTh",
        true,
        vec![
            EnsNameToken::Mapped(TokenMapped { cp: 82, cps: vec![114] }),
            EnsNameToken::Valid(TokenValid { cps: vec![97] }),
            EnsNameToken::Mapped(TokenMapped { cp: 70, cps: vec![102] }),
            EnsNameToken::Mapped(TokenMapped { cp: 70, cps: vec![102] }),
            EnsNameToken::Mapped(TokenMapped { cp: 89, cps: vec![121] }),
            EnsNameToken::Emoji(TokenEmoji { input: vec![128692, 8205, 9794, 65039], emoji: vec![128692, 8205, 9794, 65039], cps: vec![128692, 8205, 9794] }),
            EnsNameToken::Stop(TokenStop { cp: 46 }),
            EnsNameToken::Valid(TokenValid { cps: vec![101] }),
            EnsNameToken::Mapped(TokenMapped { cp: 84, cps: vec![116] }),
            EnsNameToken::Valid(TokenValid { cps: vec![104] }),
        ]
    )]
    fn test_ens_tokenize(
        #[case] input: &str,
        #[case] normalize: bool,
        #[case] expected: Vec<EnsNameToken>,
        specs: &CodePointsSpecs,
    ) {
        let result = tokenize_label(input, specs, normalize).expect("tokenize");
        assert_eq!(result.tokens, expected);
    }

    #[rstest]
    #[case::disallowed("/", false)]
    fn test_ens_tokenize_disallowed(
        #[case] input: &str,
        #[case] normalize: bool,
        specs: &CodePointsSpecs,
    ) {
        let result = tokenize_label(input, specs, normalize);
        result.expect_err("should be disallowed");
    }
}
