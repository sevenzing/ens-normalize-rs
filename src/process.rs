use crate::{
    beautify::beautify_labels, join::join_labels, tokenize_name, validate::validate_label,
    CodePointsSpecs, ProcessError, TokenizedName, ValidatedLabel,
};

#[derive(Default)]
pub struct Processor {
    specs: CodePointsSpecs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedName {
    pub input: String,
    pub labels: Vec<ValidatedLabel>,
}

impl Processor {
    pub fn new(specs: CodePointsSpecs) -> Self {
        Self { specs }
    }

    pub fn tokenize(&self, input: impl AsRef<str>) -> Result<TokenizedName, ProcessError> {
        tokenize_name(input.as_ref(), &self.specs, true)
    }

    pub fn process(&self, input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
        let input = input.as_ref();
        let tokenized = tokenize_name(input, &self.specs, true)?;
        let validated = tokenized
            .labels
            .into_iter()
            .map(|label| validate_label(label, &self.specs))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ProcessedName {
            input: input.to_string(),
            labels: validated,
        })
    }

    pub fn normalize(&self, input: impl AsRef<str>) -> Result<String, ProcessError> {
        self.process(input).map(|processed| processed.normalize())
    }

    pub fn beautify(&self, input: impl AsRef<str>) -> Result<String, ProcessError> {
        self.process(input).map(|processed| processed.beautify())
    }
}

impl ProcessedName {
    pub fn beautify(&self) -> String {
        beautify_labels(self.labels.clone())
    }

    pub fn normalize(&self) -> String {
        join_labels(self.labels.clone())
    }
}

pub fn tokenize(input: impl AsRef<str>) -> Result<TokenizedName, ProcessError> {
    let processor = Processor::new(CodePointsSpecs::default());
    processor.tokenize(input)
}

pub fn process(input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
    Processor::new(CodePointsSpecs::default()).process(input)
}

pub fn normalize(input: impl AsRef<str>) -> Result<String, ProcessError> {
    Processor::new(CodePointsSpecs::default()).normalize(input)
}

pub fn beautify(input: impl AsRef<str>) -> Result<String, ProcessError> {
    Processor::new(CodePointsSpecs::default()).beautify(input)
}
