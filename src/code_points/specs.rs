use super::types::*;
use crate::{
    constants,
    static_data::{
        nf_json,
        spec_json::{self, GroupName},
    },
    utils, CodePoint,
};
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;

struct Extent {
    group_indices: HashSet<usize>,
    codepoints: HashSet<CodePoint>,
}

struct EmojiNode {
    children: HashMap<CodePoint, usize>,
    emoji: Option<Vec<CodePoint>>,
}

struct EmojiTrie {
    nodes: Vec<EmojiNode>,
}

impl EmojiTrie {
    fn new() -> Self {
        Self {
            nodes: vec![EmojiNode {
                children: HashMap::new(),
                emoji: None,
            }],
        }
    }

    fn insert(&mut self, emoji: &[CodePoint]) {
        let mut frontier = vec![0];
        for &cp in emoji {
            let mut next = Vec::new();
            for &idx in &frontier {
                let child_idx = if let Some(&child) = self.nodes[idx].children.get(&cp) {
                    child
                } else {
                    self.nodes.push(EmojiNode {
                        children: HashMap::new(),
                        emoji: None,
                    });
                    let child = self.nodes.len() - 1;
                    self.nodes[idx].children.insert(cp, child);
                    child
                };
                next.push(child_idx);
            }
            if cp == constants::CP_FE0F {
                frontier.extend(next);
            } else {
                frontier = next;
            }
        }
        for idx in frontier {
            self.nodes[idx].emoji = Some(emoji.to_vec());
        }
    }

    fn longest_match(&self, cps: &[CodePoint], start: usize) -> Option<(usize, Vec<CodePoint>)> {
        let mut frontier = vec![0];
        let mut matched: Option<Vec<CodePoint>> = None;
        let mut end = start;

        for (i, &cp) in cps.iter().enumerate().skip(start) {
            let mut next = Vec::new();
            for &idx in &frontier {
                if let Some(&child) = self.nodes[idx].children.get(&cp) {
                    next.push(child);
                }
            }
            if cp == constants::CP_FE0F {
                frontier.extend(&next);
            } else {
                frontier = next;
            }
            for &idx in &frontier {
                if let Some(emoji) = &self.nodes[idx].emoji {
                    matched = Some(emoji.clone());
                    end = i + 1;
                }
            }
        }

        matched.map(|emoji| (end, emoji))
    }
}

/// This struct contains logic for validating and normalizing code points.
pub struct CodePointsSpecs {
    cm: HashSet<CodePoint>,
    ignored: HashSet<CodePoint>,
    mapped: HashMap<CodePoint, Vec<CodePoint>>,
    nfc_check: HashSet<CodePoint>,
    wholes: Vec<ParsedWhole>,
    confusables: HashMap<CodePoint, usize>,
    unique_non_confusables: HashSet<CodePoint>,
    fenced: HashMap<CodePoint, String>,
    groups: Vec<ParsedGroup>,
    group_name_to_index: HashMap<spec_json::GroupName, usize>,
    valid: HashSet<CodePoint>,
    nsm: HashSet<CodePoint>,
    nsm_max: u32,
    emoji_no_fe0f_to_pretty: HashMap<Vec<CodePoint>, Vec<CodePoint>>,
    decomp: HashMap<CodePoint, Vec<CodePoint>>,
    emoji_trie: EmojiTrie,
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
        let valid = compute_valid(&groups);
        let (wholes, confusables) = decode_wholes(spec.wholes, &groups);
        let unique_non_confusables = compute_unique_non_confusables(&groups, &confusables);

        let mut emoji_trie = EmojiTrie::new();
        for e in &emoji {
            emoji_trie.insert(e);
        }

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
            wholes,
            confusables,
            unique_non_confusables,
            group_name_to_index,
            emoji_trie,
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
        self.emoji_trie
            .longest_match(cps, 0)
            .map(|(end, _)| end == cps.len())
            .unwrap_or(false)
    }

    pub fn longest_emoji_at(
        &self,
        input: &str,
        byte_offset: usize,
    ) -> Option<(usize, Vec<CodePoint>)> {
        let cps = utils::str2cps(&input[byte_offset..]);
        let (end, emoji) = self.emoji_trie.longest_match(&cps, 0)?;
        let mut byte_end = byte_offset;
        for cp in &cps[..end] {
            byte_end += utils::cp2str(*cp).len();
        }
        Some((byte_end, emoji))
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

    pub fn whole_for_confusable(&self, cp: CodePoint) -> Option<&ParsedWhole> {
        self.confusables.get(&cp).map(|&idx| &self.wholes[idx])
    }

    pub fn is_unique_non_confusable(&self, cp: CodePoint) -> bool {
        self.unique_non_confusables.contains(&cp)
    }

    pub fn group_at(&self, index: usize) -> &ParsedGroup {
        &self.groups[index]
    }

    pub fn group_by_name(&self, name: impl Into<GroupName>) -> Option<&ParsedGroup> {
        self.group_name_to_index
            .get(&name.into())
            .and_then(|i| self.groups.get(*i))
    }
}

fn compute_valid(groups: &[ParsedGroup]) -> HashSet<CodePoint> {
    let mut valid = HashSet::new();
    for g in groups {
        valid.extend(g.primary_plus_secondary.iter().copied());
    }

    let seeds: Vec<CodePoint> = valid.iter().copied().collect();
    for cp in seeds {
        for c in utils::cp2str(cp).nfd() {
            valid.insert(c as CodePoint);
        }
    }
    valid
}

fn decode_wholes(
    wholes: Vec<spec_json::Whole>,
    groups: &[ParsedGroup],
) -> (Vec<ParsedWhole>, HashMap<CodePoint, usize>) {
    let mut parsed_wholes = Vec::with_capacity(wholes.len());
    let mut confusables = HashMap::new();

    for whole in wholes {
        let valid: HashSet<CodePoint> = whole.valid.into_iter().collect();
        let confused: HashSet<CodePoint> = whole.confused.into_iter().collect();
        let mut parsed = ParsedWhole {
            valid: valid.clone(),
            confused: confused.clone(),
            complements: HashMap::new(),
        };

        let whole_index = parsed_wholes.len();
        for cp in &confused {
            confusables.insert(*cp, whole_index);
        }

        let mut cover = HashSet::new();
        let mut extents = Vec::new();

        for cp in valid.iter().chain(confused.iter()) {
            let cp_groups: HashSet<usize> = groups
                .iter()
                .enumerate()
                .filter(|(_, group)| group.contains_cp(*cp))
                .map(|(index, _)| index)
                .collect();

            let extent = extents.iter_mut().find(|extent: &&mut Extent| {
                cp_groups
                    .iter()
                    .any(|group_index| extent.group_indices.contains(group_index))
            });

            let extent = match extent {
                Some(extent) => extent,
                None => {
                    extents.push(Extent {
                        group_indices: HashSet::new(),
                        codepoints: HashSet::new(),
                    });
                    extents.last_mut().unwrap()
                }
            };

            for group_index in cp_groups {
                extent.group_indices.insert(group_index);
                cover.insert(group_index);
            }
            extent.codepoints.insert(*cp);
        }

        for extent in extents {
            let mut complements: Vec<usize> = cover
                .iter()
                .filter(|group_index| !extent.group_indices.contains(group_index))
                .copied()
                .collect();
            complements.sort_unstable();
            for cp in extent.codepoints {
                parsed.complements.insert(cp, complements.clone());
            }
        }

        parsed_wholes.push(parsed);
    }

    (parsed_wholes, confusables)
}

fn compute_unique_non_confusables(
    groups: &[ParsedGroup],
    confusables: &HashMap<CodePoint, usize>,
) -> HashSet<CodePoint> {
    let mut counts: HashMap<CodePoint, u32> = HashMap::new();

    for group in groups {
        for cp in &group.primary_plus_secondary {
            *counts.entry(*cp).or_default() += 1;
        }
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(cp, _)| cp)
        .filter(|cp| !confusables.contains_key(cp))
        .collect()
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
        let mut pos = 0;
        let mut matches = Vec::new();
        while pos < emoji.len() {
            if let Some((end, _)) = specs.longest_emoji_at(emoji, pos) {
                matches.push((pos, end));
                pos = end;
            } else {
                pos += emoji[pos..].chars().next().unwrap().len_utf8();
            }
        }
        assert_eq!(matches.len(), expected.len());
        for (i, (expected_emoji, start, end)) in expected.into_iter().enumerate() {
            assert_eq!(&emoji[start..end], expected_emoji);
            assert_eq!(matches[i].0, start);
            assert_eq!(matches[i].1, end);
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
