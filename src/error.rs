use crate::CodePoint;

/// Errors that can occur during processing of an ENS name.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ProcessError {
    #[error("invalid character ('{sequence}') at position {index}: {inner}")]
    CurrableError {
        inner: CurrableError,
        index: usize,
        sequence: String,
        maybe_suggest: Option<String>,
    },
    #[error("disallowed sequence: {0}")]
    DisallowedSequence(#[from] DisallowedSequence),
}

/// Errors that can be cured by the normalizer.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum CurrableError {
    #[error("underscore in middle")]
    UnderscoreInMiddle,
    #[error("hyphen at second and third position")]
    HyphenAtSecondAndThird,
    #[error("combining mark in disallowed position at the start of the label")]
    CmStart,
    #[error("combining mark in disallowed position after an emoji")]
    CmAfterEmoji,
    #[error("fenced character at the start of a label")]
    FencedLeading,
    #[error("fenced character at the end of a label")]
    FencedTrailing,
    #[error("consecutive sequence of fenced characters")]
    FencedConsecutive,
    #[error("contains visually confusing characters from multiple scripts: character with code '{cp}' not in group '{group_name}'")]
    Confused { group_name: String, cp: CodePoint },
}

/// Errors regarding disallowed sequences.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum DisallowedSequence {
    #[error("disallowed character: {0}")]
    Invalid(String),
    #[error("invisible character: {0}")]
    InvisibleCharacter(CodePoint),
    #[error("empty label")]
    EmptyLabel,
    #[error("nsm too many")]
    NsmTooMany,
    #[error("nsm repeated")]
    NsmRepeated,
    #[error("contains visually confusing characters from {group1} and {group2} scripts")]
    ConfusedGroups { group1: String, group2: String },
}
