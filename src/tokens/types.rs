use crate::{constants, utils, CodePoint};

/// Represents a token in an ENS name.
/// see https://docs.ens.domains/ensip/15#tokenize for more details.
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
            EnsNameToken::Emoji(t) => t.cps_no_fe0f.clone(),
            EnsNameToken::Disallowed(t) => vec![t.cp],
            EnsNameToken::Stop(t) => vec![t.cp],
            EnsNameToken::Ignored(t) => vec![t.cp],
        }
    }

    pub fn input_size(&self) -> usize {
        match self {
            EnsNameToken::Valid(t) => t.cps.len(),
            EnsNameToken::Nfc(t) => t.input.len(),
            EnsNameToken::Emoji(t) => t.cps_input.len(),
            EnsNameToken::Mapped(_) => 1,
            EnsNameToken::Disallowed(_) => 1,
            EnsNameToken::Ignored(_) => 1,
            EnsNameToken::Stop(_) => 1,
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(
            self,
            EnsNameToken::Valid(_) | EnsNameToken::Mapped(_) | EnsNameToken::Nfc(_)
        )
    }

    pub fn is_emoji(&self) -> bool {
        matches!(self, EnsNameToken::Emoji(_))
    }

    pub fn is_ignored(&self) -> bool {
        matches!(self, EnsNameToken::Ignored(_))
    }

    pub fn is_disallowed(&self) -> bool {
        matches!(self, EnsNameToken::Disallowed(_))
    }

    pub fn is_stop(&self) -> bool {
        matches!(self, EnsNameToken::Stop(_))
    }

    pub fn stop() -> Self {
        Self::Stop(TokenStop {
            cp: constants::CP_STOP,
        })
    }

    pub fn as_string(&self) -> String {
        utils::cps2str(&self.cps())
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
    pub input: String,
    pub emoji: Vec<CodePoint>,
    pub cps_input: Vec<CodePoint>,
    pub cps_no_fe0f: Vec<CodePoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollapsedEnsNameToken {
    Text(TokenValid),
    Emoji(TokenEmoji),
}

impl CollapsedEnsNameToken {
    pub fn input_size(&self) -> usize {
        match self {
            CollapsedEnsNameToken::Text(t) => t.cps.len(),
            CollapsedEnsNameToken::Emoji(t) => t.cps_input.len(),
        }
    }
}
