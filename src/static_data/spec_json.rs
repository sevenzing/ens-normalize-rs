#![allow(unused)]

use crate::{utils::filter_fe0f, CodePoint};
use anyhow::Context;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize};
use serde_plain::{derive_display_from_serialize, derive_fromstr_from_deserialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
    str::FromStr,
};
use unicode_normalization::UnicodeNormalization;

const SPEC_CONTENT: &str = include_str!("spec.json");

lazy_static! {
    pub static ref DEFAULT_SPEC: Spec = serde_json::from_str(SPEC_CONTENT).unwrap();
}

#[derive(Debug, Clone, Deserialize)]
#[serde_as]
pub struct Spec {
    pub created: String,
    pub unicode: String,
    pub cldr: String,
    pub emoji: Vec<Vec<CodePoint>>,
    pub ignored: Vec<CodePoint>,
    #[serde(deserialize_with = "deserialize_mapped")]
    pub mapped: Vec<Mapped>,
    #[serde(deserialize_with = "deserialize_fenced")]
    pub fenced: Vec<Fenced>,
    pub cm: Vec<CodePoint>,
    pub nsm: Vec<CodePoint>,
    pub nsm_max: u32,
    pub escape: Vec<CodePoint>,
    pub groups: Vec<Group>,
    pub nfc_check: Vec<CodePoint>,
    pub wholes: Vec<Whole>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Mapped {
    pub from: CodePoint,
    pub to: Vec<CodePoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Fenced {
    pub from: CodePoint,
    pub to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Whole {
    /// Hex codepoint id(s); metadata only in Unicode 17 `spec.json`.
    #[serde(rename = "target")]
    _target: String,
    pub valid: Vec<CodePoint>,
    pub confused: Vec<CodePoint>,
}

fn deserialize_mapped<'de, D>(deserializer: D) -> Result<Vec<Mapped>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Vec<(CodePoint, Vec<CodePoint>)> = Vec::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(from, to)| Mapped { from, to })
        .collect())
}

fn deserialize_fenced<'de, D>(deserializer: D) -> Result<Vec<Fenced>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Vec<(CodePoint, String)> = Vec::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|(from, to)| Fenced { from, to })
        .collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct Group {
    pub name: GroupName,
    pub primary: Vec<CodePoint>,
    pub secondary: Vec<CodePoint>,
    #[serde(default)]
    pub cm: Vec<CodePoint>,
    #[serde(default)]
    pub restricted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub enum GroupName {
    Emoji,
    #[serde(rename = "ASCII")]
    Ascii,
    Greek,
    #[serde(untagged)]
    Other(String),
}
derive_fromstr_from_deserialize!(GroupName);
derive_display_from_serialize!(GroupName);

impl From<String> for GroupName {
    fn from(s: String) -> Self {
        s.parse::<Self>().unwrap_or(Self::Other(s))
    }
}

impl GroupName {
    pub fn is_greek(&self) -> bool {
        matches!(self, GroupName::Greek)
    }
}

impl Default for Spec {
    fn default() -> Self {
        DEFAULT_SPEC.clone()
    }
}
