use crate::{
    beautify::beautify_labels, join::join_labels, validate::validate_name, CodePointsSpecs,
    ProcessError, TokenizedName, ValidatedLabel,
};

/// Main struct to handle ENS name normalization including
/// tokenization, validation, beautification and normalization
#[derive(Default)]
pub struct EnsNameNormalizer {
    specs: CodePointsSpecs,
}

/// Result of processing an ENS name.
/// Contains tokenized name as intermediate processing result and validated labels.
/// Validated labels can be normalized and beautified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedName {
    pub labels: Vec<ValidatedLabel>,
    pub tokenized: TokenizedName,
}

impl EnsNameNormalizer {
    pub fn new(specs: CodePointsSpecs) -> Self {
        Self { specs }
    }

    /// Tokenize the input string, return a `TokenizedName` object with `Vec<EnsNameToken>` inside
    pub fn tokenize(&self, input: impl AsRef<str>) -> TokenizedName {
        TokenizedName::from_input(input.as_ref(), &self.specs, true)
    }

    /// Process the input string, return a `ProcessedName` object with `Vec<ValidatedLabel>` inside
    /// This function will tokenize and validate the name. Processed name can be normalized and beautified.
    pub fn process(&self, input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
        let input = input.as_ref();
        let tokenized = self.tokenize(input);
        let labels = validate_name(&tokenized, &self.specs)?;
        Ok(ProcessedName { tokenized, labels })
    }

    /// Normalize the input string, return a normalized version of ENS name
    pub fn normalize(&self, input: impl AsRef<str>) -> Result<String, ProcessError> {
        self.process(input).map(|processed| processed.normalize())
    }

    /// Beautify the input string, return a beautified version of ENS name/// Beautify the input string, return a beautified version of ENS name
    pub fn beautify(&self, input: impl AsRef<str>) -> Result<String, ProcessError> {
        self.process(input).map(|processed| processed.beautify())
    }
}

impl ProcessedName {
    pub fn normalize(&self) -> String {
        join_labels(&self.labels)
    }

    pub fn beautify(&self) -> String {
        beautify_labels(&self.labels)
    }
}

/// `no-cache` version of [`EnsNameNormalizer::tokenize`]
pub fn tokenize(input: impl AsRef<str>) -> TokenizedName {
    EnsNameNormalizer::default().tokenize(input)
}

/// `no-cache` version of [`EnsNameNormalizer::process`]
pub fn process(input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
    EnsNameNormalizer::default().process(input)
}

/// `no-cache` version of [`EnsNameNormalizer::normalize`]
pub fn normalize(input: impl AsRef<str>) -> Result<String, ProcessError> {
    EnsNameNormalizer::default().normalize(input)
}

/// `no-cache` version of [`EnsNameNormalizer::beautify`]
pub fn beautify(input: impl AsRef<str>) -> Result<String, ProcessError> {
    EnsNameNormalizer::default().beautify(input)
}
