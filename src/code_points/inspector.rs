use super::constants;
use crate::{
    static_data::{Group, NfJson, SpecJson},
    utils, CodePoint,
};
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

pub struct CodePointsInspector {
    pub spec_json: SpecJson,
    pub nf_json: NfJson,
    pub cm: HashSet<CodePoint>,
    pub emoji: HashSet<Vec<CodePoint>>,
    pub ignored: HashSet<CodePoint>,
    pub mapped: HashMap<CodePoint, Vec<CodePoint>>,
    pub nfc_check: HashSet<CodePoint>,
    pub fenced: HashMap<CodePoint, String>,
    pub groups: Vec<Group>,
    pub valid: HashSet<CodePoint>,
    pub nsm: HashSet<CodePoint>,
    pub nsm_max: u32,
    pub emoji_no_fe0f_to_pretty: HashMap<Vec<CodePoint>, Vec<CodePoint>>,
    pub decomp: HashMap<CodePoint, Vec<CodePoint>>,
}

impl CodePointsInspector {
    pub fn new(spec: SpecJson, nf: NfJson) -> Self {
        let spec_json = spec.clone();
        let nf_json = nf.clone();
        let emoji: HashSet<Vec<CodePoint>> = spec.emoji.into_iter().collect();
        let emoji_no_fe0f_to_pretty = emoji
            .iter()
            .map(|e| (utils::filter_fe0f(e), e.clone()))
            .collect();
        let decomp = nf
            .decomp
            .into_iter()
            .map(|item| (item.number, item.nested_numbers))
            .collect();
        let valid = compute_valid(&spec.groups, &decomp);
        Self {
            spec_json,
            nf_json,
            cm: spec.cm.into_iter().collect(),
            emoji,
            emoji_no_fe0f_to_pretty,
            ignored: spec.ignored.into_iter().collect(),
            mapped: spec.mapped.into_iter().map(|m| (m.from, m.to)).collect(),
            nfc_check: spec.nfc_check.into_iter().collect(),
            fenced: spec.fenced.into_iter().map(|f| (f.from, f.to)).collect(),
            valid,
            groups: spec.groups,
            nsm: spec.nsm.into_iter().collect(),
            nsm_max: spec.nsm_max,
            decomp,
        }
    }
}

impl Default for CodePointsInspector {
    fn default() -> Self {
        let spec = SpecJson::default();
        let nf = NfJson::default();
        Self::new(spec, nf)
    }
}

impl CodePointsInspector {
    pub fn get_mapping(&self, cp: CodePoint) -> Option<&Vec<CodePoint>> {
        self.mapped.get(&cp)
    }

    pub fn cps_is_emoji(&self, cps: &[CodePoint]) -> bool {
        self.emoji.contains(cps) || self.emoji_no_fe0f_to_pretty.contains_key(cps)
    }

    pub fn cps_requires_check(&self, cps: &[CodePoint]) -> bool {
        cps.iter().any(|cp| self.nfc_check.contains(cp))
    }

    pub fn cps_emoji_no_fe0f_to_pretty(&self, cps: &[CodePoint]) -> Vec<CodePoint> {
        self.emoji_no_fe0f_to_pretty
            .get(cps)
            .cloned()
            .unwrap_or_default()
    }

    pub fn maybe_normalize(&self, cp: CodePoint) -> Option<&Vec<CodePoint>> {
        self.mapped.get(&cp)
    }

    pub fn is_valid(&self, cp: CodePoint) -> bool {
        self.valid.contains(&cp)
    }

    pub fn is_ignored(&self, cp: CodePoint) -> bool {
        self.ignored.contains(&cp)
    }

    pub fn is_stop(&self, cp: CodePoint) -> bool {
        cp == constants::CP_STOP
    }

    pub fn nfc(&self, str: &str) -> String {
        str.nfc().collect()
    }
}

fn compute_valid(
    groups: &[Group],
    decomp: &HashMap<CodePoint, Vec<CodePoint>>,
) -> HashSet<CodePoint> {
    let mut valid = HashSet::new();
    for g in groups {
        valid.extend(g.primary.iter().chain(g.secondary.iter()));
    }

    let ndf: Vec<CodePoint> = valid
        .iter()
        .flat_map(|cp| decomp.get(cp).cloned().unwrap_or_default())
        .collect();
    valid.extend(ndf);
    valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    fn inspector() -> CodePointsInspector {
        CodePointsInspector::default()
    }

    #[rstest]
    #[case::letter_a('A', "a")]
    #[case::roman_numeral_vi('Ⅵ', "vi")]
    fn test_mapped(#[case] input: char, #[case] output: &str, inspector: &CodePointsInspector) {
        let mapped = inspector.get_mapping(input as u32);
        let expected = output.chars().map(|c| c as u32).collect::<Vec<_>>();
        assert_eq!(mapped, Some(&expected));
    }

    #[rstest]
    #[case::slash("⁄")]
    fn test_fenced(#[case] fence: &str, inspector: &CodePointsInspector) {
        assert!(
            inspector
                .fenced
                .contains_key(&(fence.chars().next().unwrap() as u32)),
            "Fence {fence} not found"
        );
    }

    #[rstest]
    #[case::man_technologist("👨‍💻")]
    fn test_emoji(#[case] emoji: &str, inspector: &CodePointsInspector) {
        let cps = emoji.chars().map(|c| c as u32).collect::<Vec<_>>();
        assert!(inspector.cps_is_emoji(&cps), "Emoji {emoji} not found");
    }

    #[rstest]
    #[case::small(&[36, 45, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 95, 97])]
    #[case::big(&[205743, 205742, 205741, 205740, 205739, 205738, 205737, 205736])]
    fn test_valid(#[case] cps: &[CodePoint], inspector: &CodePointsInspector) {
        for cp in cps {
            assert!(
                inspector.is_valid(*cp),
                "Codepoint {cp} is not valid, but should be"
            );
        }
    }

    #[rstest]
    #[case(&[82])]
    fn test_not_valid(#[case] cps: &[CodePoint], inspector: &CodePointsInspector) {
        for cp in cps {
            assert!(
                !inspector.is_valid(*cp),
                "Codepoint {cp} is valid, but should not be"
            );
        }
    }
}
