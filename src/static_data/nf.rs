use crate::CodePoint;
use lazy_static::lazy_static;
use serde::Deserialize;

const NF_CONTENT: &str = include_str!("nf.json");

lazy_static! {
    pub static ref DEFAULT_NF: NfJson = serde_json::from_str(NF_CONTENT).unwrap();
}

#[derive(Debug, Clone, Deserialize)]
pub struct NfJson {
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

impl Default for NfJson {
    fn default() -> Self {
        DEFAULT_NF.clone()
    }
}
