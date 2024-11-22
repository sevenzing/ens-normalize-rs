#![allow(unused)]

use crate::{utils::filter_fe0f, CodePoint};
use anyhow::Context;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    ops::Index,
};
use unicode_normalization::UnicodeNormalization;

const SPEC_CONTENT: &str = include_str!("spec.json");

lazy_static! {
    pub static ref DEFAULT_SPEC: SpecJson = serde_json::from_str(SPEC_CONTENT).unwrap();
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpecJson {
    pub created: String,
    pub unicode: String,
    pub cldr: String,
    pub emoji: Vec<Vec<CodePoint>>,
    pub ignored: Vec<CodePoint>,
    pub mapped: Vec<Mapped>,
    pub fenced: Vec<Fenced>,
    pub wholes: Vec<Whole>,
    pub cm: Vec<CodePoint>,
    pub nsm: Vec<CodePoint>,
    pub nsm_max: u32,
    pub escape: Vec<CodePoint>,
    pub groups: Vec<Group>,
    pub nfc_check: Vec<CodePoint>,
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
    pub target: String,
    pub valid: Vec<CodePoint>,
    pub confused: Vec<CodePoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Group {
    pub name: String,
    pub primary: Vec<CodePoint>,
    pub secondary: Vec<CodePoint>,
    #[serde(default)]
    pub cm: Vec<CodePoint>,
    #[serde(default)]
    pub restricted: bool,
}

impl Default for SpecJson {
    fn default() -> Self {
        DEFAULT_SPEC.clone()
    }
}
