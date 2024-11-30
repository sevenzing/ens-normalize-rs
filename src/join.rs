use crate::{constants, utils, CodePoint, EnsNameToken, ValidatedLabel};

/// Joins validated labels into a string
pub fn join_labels(labels: &[ValidatedLabel]) -> String {
    let labels_cps = labels.iter().map(|label| {
        label
            .tokens
            .iter()
            .filter_map(|token| match token {
                EnsNameToken::Disallowed(_) | EnsNameToken::Ignored(_) | EnsNameToken::Stop(_) => {
                    None
                }
                EnsNameToken::Valid(token) => Some(&token.cps),
                EnsNameToken::Mapped(token) => Some(&token.cps),
                EnsNameToken::Nfc(token) => Some(&token.cps),
                EnsNameToken::Emoji(token) => Some(&token.cps_no_fe0f),
            })
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
    });

    join_cps(labels_cps)
}

/// Joins code points into a string
pub fn join_cps(cps: impl Iterator<Item = Vec<CodePoint>>) -> String {
    let cps_flatten = itertools::intersperse(cps, vec![constants::CP_STOP])
        .flatten()
        .collect::<Vec<_>>();

    utils::cps2str(&cps_flatten)
}
