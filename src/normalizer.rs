use crate::{
    beautify::beautify_labels, join::join_labels, validate::validate_name, CodePointsSpecs,
    ProcessError, TokenizedName, ValidatedLabel,
};

#[derive(Default)]
pub struct EnsNameNormalizer {
    specs: CodePointsSpecs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedName {
    pub labels: Vec<ValidatedLabel>,
    pub tokenized: TokenizedName,
}

impl EnsNameNormalizer {
    pub fn new(specs: CodePointsSpecs) -> Self {
        Self { specs }
    }

    pub fn tokenize(&self, input: impl AsRef<str>) -> Result<TokenizedName, ProcessError> {
        TokenizedName::from_input(input.as_ref(), &self.specs, true)
    }

    pub fn process(&self, input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
        let input = input.as_ref();
        let tokenized = self.tokenize(input)?;
        let labels = validate_name(&tokenized, &self.specs)?;
        Ok(ProcessedName { tokenized, labels })
    }

    pub fn normalize(&self, input: impl AsRef<str>) -> Result<String, ProcessError> {
        self.process(input).map(|processed| processed.normalize())
    }

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

pub fn tokenize(input: impl AsRef<str>) -> Result<TokenizedName, ProcessError> {
    EnsNameNormalizer::default().tokenize(input)
}

pub fn process(input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
    EnsNameNormalizer::default().process(input)
}

pub fn normalize(input: impl AsRef<str>) -> Result<String, ProcessError> {
    EnsNameNormalizer::default().normalize(input)
}

pub fn beautify(input: impl AsRef<str>) -> Result<String, ProcessError> {
    EnsNameNormalizer::default().beautify(input)
}
