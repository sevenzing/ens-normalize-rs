use crate::static_data::spec_json;
use std::collections::{HashMap, HashSet};

pub type CodePoint = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedGroup {
    pub name: spec_json::GroupName,
    pub primary: HashSet<CodePoint>,
    pub secondary: HashSet<CodePoint>,
    pub primary_plus_secondary: HashSet<CodePoint>,
    pub cm_absent: bool,
}

impl From<spec_json::Group> for ParsedGroup {
    fn from(g: spec_json::Group) -> Self {
        Self {
            name: g.name,
            primary: g.primary.clone().into_iter().collect(),
            secondary: g.secondary.clone().into_iter().collect(),
            primary_plus_secondary: g
                .primary
                .clone()
                .into_iter()
                .chain(g.secondary.clone())
                .collect(),
            cm_absent: g.cm.is_empty(),
        }
    }
}

impl ParsedGroup {
    pub fn contains_cp(&self, cp: CodePoint) -> bool {
        self.primary_plus_secondary.contains(&cp)
    }

    pub fn contains_all_cps(&self, cps: &[CodePoint]) -> bool {
        cps.iter().all(|cp| self.contains_cp(*cp))
    }
}

#[derive(Debug, Clone)]
pub struct ParsedWhole {
    pub valid: HashSet<CodePoint>,
    pub confused: HashSet<CodePoint>,
    /// Group indices (into `CodePointsSpecs::groups`) not in this codepoint's confusable extent.
    pub complements: HashMap<CodePoint, Vec<usize>>,
}
