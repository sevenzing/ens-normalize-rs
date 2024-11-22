use crate::{CodePoint, CodePoints};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnsNameToken {
    Valid(TokenValid),
    Mapped(TokenMapped),
    Ignored(TokenIgnored),
    Disallowed(TokenDisallowed),
    Stop(TokenStop),
    Nfc(TokenNfc),
    Emoji(TokenEmoji),
}

impl EnsNameToken {
    pub fn cps(&self) -> CodePoints {
        match self {
            EnsNameToken::Valid(t) => t.cps.clone(),
            EnsNameToken::Mapped(t) => t.cps.clone(),
            EnsNameToken::Nfc(t) => t.cps.clone(),
            EnsNameToken::Emoji(t) => t.cps.clone(),
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenValid {
    pub cps: CodePoints,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMapped {
    pub cps: CodePoints,
    pub cp: CodePoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenIgnored {
    pub cp: CodePoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenDisallowed {
    pub cp: CodePoint,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStop {
    pub cp: CodePoint,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenNfc {
    pub cps: CodePoints,
    pub input: CodePoints,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenEmoji {
    pub input: CodePoints,
    pub emoji: CodePoints,
    pub cps: CodePoints,
}
