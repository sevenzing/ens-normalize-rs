use crate::CodePoint;
use lazy_static::lazy_static;
use serde::Deserialize;

const NF_CONTENT: &str = include_str!("nf.json");

lazy_static! {
    pub static ref DEFAULT_NF: Nf = serde_json::from_str(NF_CONTENT).unwrap();
}

#[derive(Debug, Clone, Deserialize)]
pub struct Nf {
    pub created: String,
    pub unicode: String,
    pub ranks: Vec<Vec<CodePoint>>,
    pub exclusions: Vec<CodePoint>,
    pub decomp: Vec<DecompItem>,
    pub qc: Vec<CodePoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DecompItem {
    pub number: CodePoint,
    pub nested_numbers: Vec<CodePoint>,
}

impl Default for Nf {
    fn default() -> Self {
        DEFAULT_NF.clone()
    }
}

#[cfg(test)]
mod nf_parse_tests {
    use super::*;
    #[test]
    fn decomp_loads() {
        let nf = Nf::default();
        assert_eq!(nf.decomp.len(), 2081);
        assert_eq!(nf.decomp[0].number, 192);
        assert_eq!(nf.decomp[0].nested_numbers, vec![65, 768]);
    }
}
