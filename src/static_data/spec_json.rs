#![allow(unused)]

use crate::{utils::filter_fe0f, CodePoint};
use anyhow::Context;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
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
    pub mapped: Vec<Mapped>,
    pub fenced: Vec<Fenced>,
    //pub wholes: Vec<Whole>,
    pub cm: Vec<CodePoint>,
    pub nsm: Vec<CodePoint>,
    pub nsm_max: u32,
    pub escape: Vec<CodePoint>,
    pub groups: Vec<Group>,
    pub nfc_check: Vec<CodePoint>,
    pub whole_map: HashMap<String, WholeValue>,
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
    #[serde(deserialize_with = "serde_aux::prelude::deserialize_number_from_string")]
    pub target: CodePoint,
    pub valid: Vec<CodePoint>,
    pub confused: Vec<CodePoint>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WholeValue {
    Number(u32),
    WholeObject(WholeObject),
}

#[derive(Debug, Clone, Deserialize)]
#[serde_as]
pub struct WholeObject {
    #[serde(rename = "V")]
    pub v: Vec<CodePoint>,
    #[serde(rename = "M")]
    pub m: HashMap<String, Vec<String>>,
}
