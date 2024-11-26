use crate::{
    constants, tokenize_name, utils,
    validate::{validate_label, ValidatedLabel},
    CodePointsSpecs, EnsNameToken, ProcessError, TokenizedName,
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

fn join_labels(labels: Vec<ValidatedLabel>) -> Result<String, ProcessError> {
    let labels_cps = labels.into_iter().map(|label| {
        label
            .tokenized
            .tokens
            .into_iter()
            .filter_map(|token| match token {
                EnsNameToken::Disallowed(_) | EnsNameToken::Ignored(_) | EnsNameToken::Stop(_) => {
                    None
                }
                EnsNameToken::Valid(token) => Some(token.cps),
                EnsNameToken::Mapped(token) => Some(token.cps),
                EnsNameToken::Nfc(token) => Some(token.cps),
                EnsNameToken::Emoji(token) => Some(token.cps),
            })
            .flatten()
            .collect::<Vec<_>>()
    });

    let cps_flatten = itertools::intersperse(labels_cps, vec![constants::CP_STOP])
        .flatten()
        .collect::<Vec<_>>();

    let str = utils::cps2str(&cps_flatten);

    Ok(str)
}

pub fn process(input: impl AsRef<str>) -> Result<ProcessedName, ProcessError> {
    let processor = Processor::new(CodePointsSpecs::default());
    processor.process(input)
}

pub fn normalize(input: impl AsRef<str>) -> Result<String, ProcessError> {
    process(input).map(|processed| processed.normalized)
}
