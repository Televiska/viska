use crate::common::factories::common::{uri::Uri, Transport, Version};
use common::libsip::{self, headers::GenValue as LibsipGenValue};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NamedHeader<T: NamedParamTrait> {
    pub display_name: Option<String>,
    pub uri: Uri,
    pub params: NamedParams<T>,
}

pub trait NamedParamTrait: Default + Into<(String, Option<String>)> {}
impl<T: Default + Into<(String, Option<String>)>> NamedParamTrait for T {}

impl<T: NamedParamTrait> Default for NamedHeader<T> {
    fn default() -> Self {
        Self {
            display_name: None,
            uri: Default::default(),
            params: vec![].into(),
        }
    }
}

impl<T: NamedParamTrait> Into<libsip::headers::NamedHeader> for NamedHeader<T> {
    fn into(self) -> libsip::headers::NamedHeader {
        libsip::headers::NamedHeader {
            display_name: self.display_name,
            uri: self.uri.into(),
            parameters: self.params.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamedParams<T: NamedParamTrait>(Vec<T>);

impl<T: NamedParamTrait> Default for NamedParams<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T: NamedParamTrait> Into<HashMap<String, Option<String>>> for NamedParams<T> {
    fn into(self) -> HashMap<String, Option<String>> {
        self.0
            .into_iter()
            .map(Into::<(String, Option<String>)>::into)
            .collect::<_>()
    }
}

impl<T: NamedParamTrait + Into<(String, Option<LibsipGenValue>)>>
    Into<HashMap<String, Option<LibsipGenValue>>> for NamedParams<T>
{
    fn into(self) -> HashMap<String, Option<LibsipGenValue>> {
        self.0
            .into_iter()
            .map(Into::<(String, Option<LibsipGenValue>)>::into)
            .collect::<HashMap<String, Option<LibsipGenValue>>>()
    }
}

impl<T: NamedParamTrait> From<Vec<T>> for NamedParams<T> {
    fn from(from: Vec<T>) -> Self {
        Self(from)
    }
}

impl<T: NamedParamTrait> From<T> for NamedParams<T> {
    fn from(from: T) -> Self {
        Self(vec![from])
    }
}

#[derive(Debug, Clone)]
pub enum NamedParam {
    Tag(Tag),
    Custom(String, Option<String>),
}

impl Default for NamedParam {
    fn default() -> Self {
        Self::Tag(Default::default())
    }
}

impl Into<(String, Option<String>)> for NamedParam {
    fn into(self) -> (String, Option<String>) {
        match self {
            Self::Tag(tag) => ("tag".into(), Some(tag.into())),
            Self::Custom(key, value) => (key, value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tag(String);

//TODO: add randomized builder here
impl Default for Tag {
    fn default() -> Self {
        Self("arandomtag".into())
    }
}

impl Into<String> for Tag {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone)]
pub enum ContactParam {
    Custom(String, Option<GenValue>),
}

impl Default for ContactParam {
    fn default() -> Self {
        ContactParam::Custom("q".into(), Some(GenValue::Value("1.0".into())))
    }
}

impl Into<(String, Option<String>)> for ContactParam {
    fn into(self) -> (String, Option<String>) {
        match self {
            Self::Custom(key, gen_value) => (key, gen_value.map(|v| v.into())),
        }
    }
}

impl Into<(String, Option<LibsipGenValue>)> for ContactParam {
    fn into(self) -> (String, Option<LibsipGenValue>) {
        match self {
            Self::Custom(key, gen_value) => (key, gen_value.map(|v| v.into())),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GenValue {
    Value(String),
    QuotedValue(String),
}

impl Into<String> for GenValue {
    fn into(self) -> String {
        match self {
            Self::Value(value) => value,
            Self::QuotedValue(value) => format!("\"{}\"", value),
        }
    }
}

impl Into<LibsipGenValue> for GenValue {
    fn into(self) -> LibsipGenValue {
        match self {
            Self::Value(value) => LibsipGenValue::Token(value),
            Self::QuotedValue(value) => LibsipGenValue::QuotedString(value),
        }
    }
}
