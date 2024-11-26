use crate::CodePoint;

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
    pub fn cps(&self) -> Vec<CodePoint> {
        match self {
            EnsNameToken::Valid(t) => t.cps.clone(),
            EnsNameToken::Mapped(t) => t.cps.clone(),
            EnsNameToken::Nfc(t) => t.cps.clone(),
            EnsNameToken::Emoji(t) => t.cps.clone(),
            _ => vec![],
        }
    }

    pub fn size(&self) -> usize {
        match self {
            EnsNameToken::Valid(t) => t.cps.len(),
            EnsNameToken::Mapped(t) => t.cps.len(),
            EnsNameToken::Nfc(t) => t.cps.len(),
            EnsNameToken::Emoji(t) => t.cps.len(),
            _ => 0,
        }
    }

    pub fn is_text(&self) -> bool {
        !self.is_emoji()
    }

    pub fn is_emoji(&self) -> bool {
        matches!(self, EnsNameToken::Emoji(_))
    }

    pub fn ignored(&self) -> bool {
        matches!(self, EnsNameToken::Ignored(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenValid {
    pub cps: Vec<CodePoint>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenMapped {
    pub cps: Vec<CodePoint>,
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
    pub cps: Vec<CodePoint>,
    pub input: Vec<CodePoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenEmoji {
    pub input: Vec<CodePoint>,
    pub emoji: Vec<CodePoint>,
    pub cps: Vec<CodePoint>,
}
