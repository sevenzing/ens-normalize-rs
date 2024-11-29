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
    pub normalized: String,
}

pub fn process(input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
    let processor = Processor::new(CodePointsSpecs::default());
    processor.process(input)
}

pub fn normalize(input: impl AsRef<str>) -> Result<String, ProcessError> {
    process(input).map(|processed| processed.normalized)
}

pub fn beautify(input: impl AsRef<str>) -> Result<String, ProcessError> {
    process(input).and_then(|processed| processed.beautify())
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
        let normalized = join_labels(validated.clone())?;
        Ok(ProcessedName {
            input: input.to_string(),
            labels: validated,
            normalized,
        })
    }
}

impl ProcessedName {
    pub fn beautify(&self) -> Result<String, ProcessError> {
        beautify_labels(self.labels.clone())
    }
}
