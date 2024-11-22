use crate::CodePoint;

const FE0F: CodePoint = 0xfe0f;

pub fn filter_fe0f(cps: &[CodePoint]) -> Vec<CodePoint> {
    cps.iter().filter(|cp| **cp != FE0F).cloned().collect()
}

pub fn cps2str(cps: &[CodePoint]) -> String {
    String::from_utf16_lossy(
        cps.iter()
            .map(|cp| *cp as u16)
            .collect::<Vec<_>>()
            .as_slice(),
    )
}

pub fn str2cps(str: &str) -> Vec<CodePoint> {
    str.chars().map(|c| c as CodePoint).collect()
}
