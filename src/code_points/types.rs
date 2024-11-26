use crate::static_data::spec_json;
use std::collections::{HashMap, HashSet};

pub type CodePoint = u32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedGroup {
    pub name: spec_json::GroupName,
    pub primary: HashSet<CodePoint>,
    pub secondary: HashSet<CodePoint>,
    pub primary_plus_secondary: HashSet<CodePoint>,
    pub cm_absent: bool,
}

impl From<spec_json::Group> for ParsedGroup {
    fn from(g: spec_json::Group) -> Self {
        Self {
            name: g.name,
            primary: g.primary.clone().into_iter().collect(),
            secondary: g.secondary.clone().into_iter().collect(),
            primary_plus_secondary: g
                .primary
                .clone()
                .into_iter()
                .chain(g.secondary.clone())
                .collect(),
            cm_absent: g.cm.is_empty(),
        }
    }
}

impl ParsedGroup {
    pub fn contains_cp(&self, cp: CodePoint) -> bool {
        self.primary_plus_secondary.contains(&cp)
    }

    pub fn contains_all_cps(&self, cps: &[CodePoint]) -> bool {
        cps.iter().all(|cp| self.contains_cp(*cp))
    }
}

pub type ParsedWholeMap = HashMap<CodePoint, ParsedWholeValue>;

pub enum ParsedWholeValue {
    Number(u32),
    WholeObject(ParsedWholeObject),
}

impl TryFrom<spec_json::WholeValue> for ParsedWholeValue {
    type Error = anyhow::Error;
    fn try_from(value: spec_json::WholeValue) -> Result<Self, Self::Error> {
        match value {
            spec_json::WholeValue::Number(number) => Ok(ParsedWholeValue::Number(number)),
            spec_json::WholeValue::WholeObject(object) => {
                Ok(ParsedWholeValue::WholeObject(object.try_into()?))
            }
        }
    }
}

pub struct ParsedWholeObject {
    pub v: HashSet<CodePoint>,
    pub m: HashMap<CodePoint, HashSet<String>>,
}

impl TryFrom<spec_json::WholeObject> for ParsedWholeObject {
    type Error = anyhow::Error;

    fn try_from(value: spec_json::WholeObject) -> Result<Self, Self::Error> {
        let v = value.v.into_iter().collect();
        let m = value
            .m
            .into_iter()
            .map(|(k, v)| {
                let k = k.parse::<CodePoint>()?;
                let v = v.into_iter().collect();
                Ok((k, v))
            })
            .collect::<Result<HashMap<CodePoint, HashSet<String>>, anyhow::Error>>()?;
        Ok(Self { v, m })
    }
}
