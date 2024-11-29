use super::types::*;
use crate::{
    constants,
    static_data::{
        nf_json,
        spec_json::{self, GroupName},
    },
    utils, CodePoint,
};
use regex::Regex;
use std::collections::{HashMap, HashSet};

/// This struct contains logic for validating and normalizing code points.
pub struct CodePointsSpecs {
    cm: HashSet<CodePoint>,
    ignored: HashSet<CodePoint>,
    mapped: HashMap<CodePoint, Vec<CodePoint>>,
    nfc_check: HashSet<CodePoint>,
    whole_map: ParsedWholeMap,
    fenced: HashMap<CodePoint, String>,
    groups: Vec<ParsedGroup>,
    group_name_to_index: HashMap<spec_json::GroupName, usize>,
    valid: HashSet<CodePoint>,
    nsm: HashSet<CodePoint>,
    nsm_max: u32,
    emoji_no_fe0f_to_pretty: HashMap<Vec<CodePoint>, Vec<CodePoint>>,
    decomp: HashMap<CodePoint, Vec<CodePoint>>,
    emoji_regex: Regex,
}

impl CodePointsSpecs {
    pub fn new(spec: spec_json::Spec, nf: nf_json::Nf) -> Self {
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
        let groups: Vec<ParsedGroup> = spec.groups.into_iter().map(ParsedGroup::from).collect();
        let group_name_to_index: HashMap<spec_json::GroupName, usize> = groups
            .iter()
            .enumerate()
            .map(|(i, g)| (g.name.clone(), i))
            .collect();
        let valid = compute_valid(&groups, &decomp);
        let whole_map = compute_whole_map(spec.whole_map);

        let emoji_str_list = emoji
            .iter()
            .map(|cps| utils::cps2str(cps))
            .collect::<Vec<_>>();
        let emoji_regex =
            create_emoji_regex_pattern(emoji_str_list).expect("failed to create emoji regex");

        Self {
            cm: spec.cm.into_iter().collect(),
            emoji_no_fe0f_to_pretty,
            ignored: spec.ignored.into_iter().collect(),
            mapped: spec.mapped.into_iter().map(|m| (m.from, m.to)).collect(),
            nfc_check: spec.nfc_check.into_iter().collect(),
            fenced: spec.fenced.into_iter().map(|f| (f.from, f.to)).collect(),
            valid,
            groups,
            nsm: spec.nsm.into_iter().collect(),
            nsm_max: spec.nsm_max,
            decomp,
            whole_map,
            group_name_to_index,
            emoji_regex,
        }
    }
}

impl Default for CodePointsSpecs {
    fn default() -> Self {
        let spec = spec_json::Spec::default();
        let nf = nf_json::Nf::default();
        Self::new(spec, nf)
    }
}

impl CodePointsSpecs {
    pub fn get_mapping(&self, cp: CodePoint) -> Option<&Vec<CodePoint>> {
        self.mapped.get(&cp)
    }

    pub fn cps_is_emoji(&self, cps: &[CodePoint]) -> bool {
        let s = utils::cps2str(cps);
        let maybe_match = self.finditer_emoji(&s).next();
        maybe_match
            .map(|m| m.start() == 0 && m.end() == s.len())
            .unwrap_or(false)
    }

    pub fn finditer_emoji<'a>(&'a self, s: &'a str) -> impl Iterator<Item = regex::Match<'_>> {
        self.emoji_regex.find_iter(s)
    }

    pub fn cps_requires_check(&self, cps: &[CodePoint]) -> bool {
        cps.iter().any(|cp| self.nfc_check.contains(cp))
    }

    pub fn cps_emoji_no_fe0f_to_pretty(&self, cps: &[CodePoint]) -> Option<&Vec<CodePoint>> {
        self.emoji_no_fe0f_to_pretty.get(cps)
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

    pub fn is_fenced(&self, cp: CodePoint) -> bool {
        self.fenced.contains_key(&cp)
    }

    pub fn is_cm(&self, cp: CodePoint) -> bool {
        self.cm.contains(&cp)
    }

    pub fn groups_for_cps<'a>(
        &'a self,
        cps: &'a [CodePoint],
    ) -> impl Iterator<Item = &'a ParsedGroup> {
        self.groups
            .iter()
            .filter(|group| cps.iter().all(|cp| group.contains_cp(*cp)))
    }

    pub fn is_nsm(&self, cp: CodePoint) -> bool {
        self.nsm.contains(&cp)
    }

    pub fn nsm_max(&self) -> u32 {
        self.nsm_max
    }

    pub fn decompose(&self, cp: CodePoint) -> Option<&Vec<CodePoint>> {
        self.decomp.get(&cp)
    }

    pub fn whole_map(&self, cp: CodePoint) -> Option<&ParsedWholeValue> {
        self.whole_map.get(&cp)
    }

    pub fn group_by_name(&self, name: impl Into<GroupName>) -> Option<&ParsedGroup> {
        self.group_name_to_index
            .get(&name.into())
            .and_then(|i| self.groups.get(*i))
    }
}

fn compute_valid(
    groups: &[ParsedGroup],
    decomp: &HashMap<CodePoint, Vec<CodePoint>>,
) -> HashSet<CodePoint> {
    let mut valid = HashSet::new();
    for g in groups {
        valid.extend(g.primary_plus_secondary.iter());
    }

    let ndf: Vec<CodePoint> = valid
        .iter()
        .flat_map(|cp| decomp.get(cp).cloned().unwrap_or_default())
        .collect();
    valid.extend(ndf);
    valid
}

fn compute_whole_map(whole_map: HashMap<String, spec_json::WholeValue>) -> ParsedWholeMap {
    whole_map
        .into_iter()
        .map(|(k, v)| (k.parse::<CodePoint>().unwrap(), v.try_into().unwrap()))
        .collect()
}

fn create_emoji_regex_pattern(emojis: Vec<impl AsRef<str>>) -> Result<Regex, regex::Error> {
    let fe0f = regex::escape(constants::STR_FEOF);

    // Make FE0F optional
    let make_emoji = |emoji: &str| regex::escape(emoji).replace(&fe0f, &format!("{}?", fe0f));

    // Order emojis to match the longest ones first
    let order = |emoji: &str| emoji.replace(constants::STR_FEOF, "").len();

    let mut sorted_emojis = emojis;
    sorted_emojis.sort_by_key(|b| std::cmp::Reverse(order(b.as_ref())));

    let emoji_regex = sorted_emojis
        .into_iter()
        .map(|emoji| make_emoji(emoji.as_ref()))
        .collect::<Vec<_>>()
        .join("|");

    regex::Regex::new(&emoji_regex)
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
    #[case::letter_a('A', "a")]
    #[case::roman_numeral_vi('Ⅵ', "vi")]
    fn test_mapped(#[case] input: char, #[case] output: &str, specs: &CodePointsSpecs) {
        let mapped = specs.get_mapping(input as u32);
        let expected = output.chars().map(|c| c as u32).collect::<Vec<_>>();
        assert_eq!(mapped, Some(&expected));
    }

    #[rstest]
    #[case::slash("⁄")]
    fn test_fenced(#[case] fence: &str, specs: &CodePointsSpecs) {
        assert!(
            specs
                .fenced
                .contains_key(&(fence.chars().next().unwrap() as u32)),
            "Fence {fence} not found"
        );
    }

    #[rstest]
    #[case::string("hello😀", vec![("😀", 5, 9)])]
    #[case::man_technologist("👨‍💻", vec![("👨‍💻", 0, 11)])]
    fn test_emoji(
        #[case] emoji: &str,
        #[case] expected: Vec<(&str, usize, usize)>,
        specs: &CodePointsSpecs,
    ) {
        let matches = specs.finditer_emoji(emoji).collect::<Vec<_>>();
        assert_eq!(matches.len(), expected.len());
        for (i, (emoji, start, end)) in expected.into_iter().enumerate() {
            assert_eq!(matches[i].as_str(), emoji);
            assert_eq!(matches[i].start(), start);
            assert_eq!(matches[i].end(), end);
        }
    }

    #[rstest]
    #[case::small(&[36, 45, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 95, 97])]
    #[case::big(&[205743, 205742, 205741, 205740, 205739, 205738, 205737, 205736])]
    fn test_valid(#[case] cps: &[CodePoint], specs: &CodePointsSpecs) {
        for cp in cps {
            assert!(
                specs.is_valid(*cp),
                "Codepoint {cp} is not valid, but should be"
            );
        }
    }

    #[rstest]
    #[case(&[82])]
    fn test_not_valid(#[case] cps: &[CodePoint], specs: &CodePointsSpecs) {
        for cp in cps {
            assert!(
                !specs.is_valid(*cp),
                "Codepoint {cp} is valid, but should not be"
            );
        }
    }
}
