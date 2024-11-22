use crate::{
    tokens::{
        EnsNameToken, TokenEmoji, TokenIgnored, TokenMapped, TokenNfc, TokenStop, TokenValid,
    },
    utils, CodePoint, CodePointsInspector,
};

use super::TokenDisallowed;

#[derive(Default)]
pub struct Tokenizer {
    inspector: CodePointsInspector,
}

impl Tokenizer {
    pub fn new(inspector: CodePointsInspector) -> Self {
        Self { inspector }
    }

    pub fn ens_tokenize(&self, label: impl AsRef<str>, normalize: bool) -> Vec<EnsNameToken> {
        let chars = label.as_ref().chars().collect::<Vec<_>>();

        let mut tokens = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            if let Some(emoji) = self.maybe_starts_with_emoji(&chars[i..]) {
                let cps_len = emoji.input.len();
                tokens.push(EnsNameToken::Emoji(emoji));
                i += cps_len;
            } else {
                let token = self.process_one_cp(chars[i] as CodePoint);
                tokens.push(token);
                i += 1;
            }
        }

        if normalize {
            self.normalize(&mut tokens);
        }
        collapse_valid_tokens(&mut tokens);
        tokens
    }

    fn normalize(&self, tokens: &mut Vec<EnsNameToken>) {
        let mut i = 0;
        let mut start = -1i32;

        while i < tokens.len() {
            let token = &tokens[i];
            match token {
                EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) => {
                    let cps = token.cps();
                    if self.inspector.cps_requires_check(&cps) {
                        let mut end = i + 1;
                        for (pos, token) in tokens.iter().enumerate().skip(end) {
                            match token {
                                EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) => {
                                    if !self.inspector.cps_requires_check(&cps) {
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
                        let str = self.inspector.nfc(&str0);

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
    fn maybe_starts_with_emoji(&self, chars: &[char]) -> Option<TokenEmoji> {
        let mut longest_match = None;
        let cps: Vec<CodePoint> = chars.iter().map(|c| *c as u32).collect();

        for i in 1..=chars.len() {
            let candidate = &cps[0..i];
            if self.inspector.cps_is_emoji(candidate) {
                let input = candidate.to_vec();
                let cps_no_fe0f = utils::filter_fe0f(&input);
                let emoji = self.inspector.cps_emoji_no_fe0f_to_pretty(&cps_no_fe0f);

                longest_match = Some(TokenEmoji {
                    input,
                    emoji,
                    cps: cps_no_fe0f,
                });
            }
        }
        longest_match
    }

    fn process_one_cp(&self, cp: CodePoint) -> EnsNameToken {
        if self.inspector.is_stop(cp) {
            EnsNameToken::Stop(TokenStop { cp })
        } else if self.inspector.is_valid(cp) {
            EnsNameToken::Valid(TokenValid { cps: vec![cp] })
        } else if self.inspector.is_ignored(cp) {
            EnsNameToken::Ignored(TokenIgnored { cp })
        } else if let Some(normalized) = self.inspector.maybe_normalize(cp) {
            EnsNameToken::Mapped(TokenMapped {
                cp,
                cps: normalized.clone(),
            })
        } else {
            EnsNameToken::Disallowed(TokenDisallowed { cp })
        }
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
    use rstest::rstest;

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
        "_R💩\u{FE0F}a\u{FE0F}\u{304}\u{AD}./",
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
            EnsNameToken::Disallowed(TokenDisallowed { cp: 47 }),
        ]
    )]
    #[case::with_nfc(
        "_R💩\u{FE0F}a\u{FE0F}\u{304}\u{AD}./",
        true,
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![95] }),
            EnsNameToken::Mapped(TokenMapped { cp: 82, cps: vec![114] }),
            EnsNameToken::Emoji(TokenEmoji { input: vec![128169, 65039], emoji: vec![128169, 65039], cps: vec![128169] }),
            EnsNameToken::Nfc(TokenNfc { input: vec![97, 772], cps: vec![257] }),
            EnsNameToken::Ignored(TokenIgnored { cp: 173 }),
            EnsNameToken::Stop(TokenStop { cp: 46 }),
            EnsNameToken::Disallowed(TokenDisallowed { cp: 47 }),
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
    ) {
        let tokenizer = Tokenizer::default();
        let result = tokenizer.ens_tokenize(input, normalize);
        assert_eq!(result, expected);
    }
}
