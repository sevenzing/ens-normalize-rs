use crate::{CodePoint, CodePointsSpecs};
use unicode_normalization::UnicodeNormalization;

const FE0F: CodePoint = 0xfe0f;
const LAST_ASCII_CP: CodePoint = 0x7f;

#[inline]
pub fn filter_fe0f(cps: &[CodePoint]) -> Vec<CodePoint> {
    cps.iter().filter(|cp| **cp != FE0F).cloned().collect()
}

#[inline]
pub fn cps2str(cps: &[CodePoint]) -> String {
    cps.iter()
        .filter_map(|&code_point| char::from_u32(code_point))
        .collect()
}

pub fn cp2str(cp: CodePoint) -> String {
    cps2str(&[cp])
}

#[inline]
pub fn str2cps(str: &str) -> Vec<CodePoint> {
    str.chars().map(|c| c as CodePoint).collect()
}

#[inline]
pub fn is_ascii(cp: CodePoint) -> bool {
    cp <= LAST_ASCII_CP
}

#[inline]
pub fn nfc(str: &str) -> String {
    str.nfc().collect()
}

pub fn nfd_cps(cps: &[CodePoint], specs: &CodePointsSpecs) -> Vec<CodePoint> {
    let mut decomposed = Vec::new();
    for cp in cps {
        if let Some(decomposed_cp) = specs.decompose(*cp) {
            decomposed.extend(decomposed_cp);
        } else {
            decomposed.push(*cp);
        }
    }
    decomposed
}
