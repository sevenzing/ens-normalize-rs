use crate::CodePoint;

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum ProcessError {
    #[error("contains visually confusing characters from multiple scripts: {0}")]
    Confused(String),
    #[error("contains visually confusing characters from {group1} and {group2} scripts")]
    ConfusedGroups { group1: String, group2: String },
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
}

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
}
