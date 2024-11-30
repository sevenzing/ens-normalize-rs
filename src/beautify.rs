use crate::{constants, join::join_cps, CodePoint, EnsNameToken, LabelType, ValidatedLabel};

/// Beautifies a list of validated labels by
/// - replacing Greek code points with their pretty variants
/// - using pretty variants of emojis
pub fn beautify_labels(labels: &[ValidatedLabel]) -> String {
    let labels_cps = labels.iter().map(|label| {
        label
            .tokens
            .iter()
            .filter_map(|token| match token {
                EnsNameToken::Emoji(emoji) => Some(emoji.emoji.clone()),
                EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) | EnsNameToken::Nfc(_) => {
                    Some(cps_replaced_greek(token.cps(), &label.label_type))
                }
                EnsNameToken::Ignored(_) | EnsNameToken::Disallowed(_) | EnsNameToken::Stop(_) => {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>()
    });
    join_cps(labels_cps)
}

fn cps_replaced_greek(mut cps: Vec<CodePoint>, label_type: &LabelType) -> Vec<CodePoint> {
    if !label_type.is_greek() {
        cps.iter_mut().for_each(|cp| {
            if *cp == constants::CP_XI_SMALL {
                *cp = constants::CP_XI_CAPITAL;
            }
        });
    }

    cps
}
