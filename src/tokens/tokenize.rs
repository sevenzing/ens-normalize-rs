use crate::{
    tokens::{
        CollapsedEnsNameToken, EnsNameToken, TokenDisallowed, TokenEmoji, TokenIgnored,
        TokenMapped, TokenNfc, TokenStop, TokenValid,
    },
    utils, CodePoint, CodePointsSpecs,
};

/// Represents a full ENS name, including the original input and the sequence of tokens
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedName {
    pub input: String,
    pub tokens: Vec<EnsNameToken>,
}

/// Represents a tokenized ENS label (part of a name separated by periods), including sequence of tokens
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedLabel<'a> {
    pub tokens: &'a [EnsNameToken],
}

impl TokenizedName {
    pub fn empty() -> Self {
        Self {
            input: "".to_string(),
            tokens: vec![],
        }
    }

    /// Tokenizes an input string, applying NFC normalization if requested.
    pub fn from_input(input: impl AsRef<str>, specs: &CodePointsSpecs, apply_nfc: bool) -> Self {
        tokenize_name(input, specs, apply_nfc)
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Returns an iterator over all tokens in the tokenized name.
    pub fn iter_tokens(&self) -> impl Iterator<Item = &EnsNameToken> {
        self.tokens.iter()
    }

    /// Returns an iterator over all labels in the tokenized name.
    /// Basically, it splits the tokenized name by stop tokens.
    pub fn iter_labels(&self) -> impl Iterator<Item = TokenizedLabel<'_>> {
        self.tokens
            .split(|t| matches!(t, EnsNameToken::Stop(_)))
            .map(TokenizedLabel::from)
    }

    pub fn labels(&self) -> Vec<TokenizedLabel<'_>> {
        self.iter_labels().collect()
    }
}

impl TokenizedLabel<'_> {
    /// Returns true if all tokens in the label are emoji tokens
    pub fn is_fully_emoji(&self) -> bool {
        self.tokens
            .iter()
            .all(|t| matches!(t, EnsNameToken::Emoji(_)))
    }

    /// Returns true if all codepoints in all tokens are ASCII characters
    pub fn is_fully_ascii(&self) -> bool {
        self.tokens
            .iter()
            .all(|token| token.cps().into_iter().all(utils::is_ascii))
    }

    /// Returns an iterator over all codepoints in all tokens.
    pub fn iter_cps(&self) -> impl DoubleEndedIterator<Item = CodePoint> + '_ {
        self.tokens.iter().flat_map(|token| token.cps())
    }

    /// Collapses consecutive text tokens into single text tokens, keeping emoji tokens separate.
    /// Returns a vector of either Text or Emoji tokens.
    pub fn collapse_into_text_or_emoji(&self) -> Vec<CollapsedEnsNameToken> {
        let mut current_text_cps = vec![];
        let mut collapsed = vec![];
        for token in self.tokens.iter() {
            match token {
                EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) | EnsNameToken::Nfc(_) => {
                    current_text_cps.extend(token.cps().iter());
                }
                EnsNameToken::Emoji(token) => {
                    if !current_text_cps.is_empty() {
                        collapsed.push(CollapsedEnsNameToken::Text(TokenValid {
                            cps: current_text_cps,
                        }));
                        current_text_cps = vec![];
                    }
                    collapsed.push(CollapsedEnsNameToken::Emoji(token.clone()));
                }
                EnsNameToken::Ignored(_) | EnsNameToken::Disallowed(_) | EnsNameToken::Stop(_) => {}
            }
        }
        if !current_text_cps.is_empty() {
            collapsed.push(CollapsedEnsNameToken::Text(TokenValid {
                cps: current_text_cps,
            }));
        }
        collapsed
    }

    /// Returns a vector of codepoints from all text tokens, excluding emoji and ignored tokens
    pub fn get_cps_of_not_ignored_text(&self) -> Vec<CodePoint> {
        self.collapse_into_text_or_emoji()
            .into_iter()
            .filter_map(|token| {
                if let CollapsedEnsNameToken::Text(token) = token {
                    Some(token.cps)
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

impl<'a, T> From<&'a T> for TokenizedLabel<'a>
where
    T: AsRef<[EnsNameToken]> + ?Sized,
{
    fn from(tokens: &'a T) -> Self {
        TokenizedLabel {
            tokens: tokens.as_ref(),
        }
    }
}

fn tokenize_name(name: impl AsRef<str>, specs: &CodePointsSpecs, apply_nfc: bool) -> TokenizedName {
    let name = name.as_ref();
    if name.is_empty() {
        return TokenizedName::empty();
    }
    let tokens = tokenize_input(name, specs, apply_nfc);
    TokenizedName {
        input: name.to_string(),
        tokens,
    }
}

fn tokenize_input(
    input: impl AsRef<str>,
    specs: &CodePointsSpecs,
    apply_nfc: bool,
) -> Vec<EnsNameToken> {
    let input = input.as_ref();
    let emojis = specs.finditer_emoji(input).collect::<Vec<_>>();

    let mut tokens = Vec::new();
    let mut input_cur = 0;

    while input_cur < input.len() {
        if let Some(emoji) = maybe_starts_with_emoji(input_cur, input, &emojis, specs) {
            let cursor_offset = emoji.input.len();
            tokens.push(EnsNameToken::Emoji(emoji));
            input_cur += cursor_offset;
        } else {
            let char = input[input_cur..]
                .chars()
                .next()
                .expect("input_cur is in bounds");
            let cursor_offset = char.len_utf8();
            let cp = char as CodePoint;
            let token = process_one_cp(cp, specs);
            tokens.push(token);
            input_cur += cursor_offset;
        }
    }

    if apply_nfc {
        perform_nfc_transform(&mut tokens, specs);
    }
    collapse_valid_tokens(&mut tokens);
    tokens
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
fn maybe_starts_with_emoji(
    i: usize,
    label: &str,
    emojis: &[regex::Match],
    specs: &CodePointsSpecs,
) -> Option<TokenEmoji> {
    emojis.iter().find_map(|emoji| {
        let start = emoji.start();
        if start == i {
            let end = emoji.end();
            let input_cps = utils::str2cps(&label[start..end]);
            let cps_no_fe0f = utils::filter_fe0f(&input_cps);
            let emoji = specs
                .cps_emoji_no_fe0f_to_pretty(&cps_no_fe0f)
                .expect("emoji should be found")
                .clone();
            Some(TokenEmoji {
                input: label[start..end].to_string(),
                cps_input: input_cps,
                emoji,
                cps_no_fe0f,
            })
        } else {
            None
        }
    })
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
        "xyz👨🏻/",
        true,
        vec![
            EnsNameToken::Valid(TokenValid { cps: vec![120, 121, 122] }),
            EnsNameToken::Emoji(TokenEmoji { input: "👨🏻".to_string(), cps_input: vec![128104, 127995], emoji: vec![128104, 127995], cps_no_fe0f: vec![128104, 127995] }),
            EnsNameToken::Disallowed(TokenDisallowed { cp: 47 }),
        ]
    )]
    #[case::a_poop_b(
        "A💩︎︎b",
        true,
        vec![
            EnsNameToken::Mapped(TokenMapped { cp: 65, cps: vec![97] }),
            EnsNameToken::Emoji(TokenEmoji { input: "💩".to_string(), cps_input: vec![128169], emoji: vec![128169, 65039], cps_no_fe0f: vec![128169] }),
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
            EnsNameToken::Emoji(TokenEmoji { input: "💩️".to_string(), cps_input: vec![128169, 65039], emoji: vec![128169, 65039], cps_no_fe0f: vec![128169] }),
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
            EnsNameToken::Emoji(TokenEmoji { input: "💩️".to_string(), cps_input: vec![128169, 65039], emoji: vec![128169, 65039], cps_no_fe0f: vec![128169] }),
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
            EnsNameToken::Emoji(TokenEmoji { input: "🚴\u{200d}♂\u{fe0f}".to_string(), cps_input: vec![128692, 8205, 9794, 65039], emoji: vec![128692, 8205, 9794, 65039], cps_no_fe0f: vec![128692, 8205, 9794] }),
            EnsNameToken::Stop(TokenStop { cp: 46 }),
            EnsNameToken::Valid(TokenValid { cps: vec![101] }),
            EnsNameToken::Mapped(TokenMapped { cp: 84, cps: vec![116] }),
            EnsNameToken::Valid(TokenValid { cps: vec![104] }),
        ]
    )]
    #[case::emojis(
        "⛹️‍♀",
        true,
        vec![
            EnsNameToken::Emoji(TokenEmoji { input: "⛹️‍♀".to_string(), cps_input: vec![9977, 65039, 8205, 9792], emoji: vec![9977, 65039, 8205, 9792, 65039], cps_no_fe0f: vec![9977, 8205, 9792] }),
        ]
    )]
    fn test_ens_tokenize(
        #[case] input: &str,
        #[case] apply_nfc: bool,
        #[case] expected: Vec<EnsNameToken>,
        specs: &CodePointsSpecs,
    ) {
        let tokens = tokenize_input(input, specs, apply_nfc);
        assert_eq!(tokens, expected);
    }

    #[rstest]
    #[case::leading_cm(
        "󠅑𑆻👱🏿‍♀️xyz",
        vec![
            CollapsedEnsNameToken::Text(TokenValid { cps: vec![70075] }),
            CollapsedEnsNameToken::Emoji(TokenEmoji { input: "👱🏿‍♀️".to_string(), cps_input: vec![128113, 127999, 8205, 9792, 65039], emoji: vec![128113, 127999, 8205, 9792, 65039], cps_no_fe0f: vec![128113, 127999, 8205, 9792] }),
            CollapsedEnsNameToken::Text(TokenValid { cps: vec![120, 121, 122] }),
        ]
    )]
    #[case::atm(
        "a™️",
        vec![
            CollapsedEnsNameToken::Text(TokenValid { cps: vec![97, 116, 109] }),
        ]
    )]
    fn test_collapse(
        #[case] input: &str,
        #[case] expected: Vec<CollapsedEnsNameToken>,
        specs: &CodePointsSpecs,
    ) {
        let tokens = tokenize_input(input, specs, true);
        let label = TokenizedLabel::from(&tokens);
        let result = label.collapse_into_text_or_emoji();
        assert_eq!(result, expected);
    }
}
