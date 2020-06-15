extern crate proc_macro;

use proc_macro2::Span;
use std::collections::BTreeMap;
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Error, Fields, Ident, Lit, Meta, MetaNameValue, Variant,
};

pub struct RegexValue {
    pub span: Span,
    pub regex: String,
}

pub enum Regex {
    Token(RegexValue),
    Regex(RegexValue),
}

pub struct InputTokenRegexes {
    pub enum_name: Ident,
    pub skip_regex: RegexValue,
    pub variants: BTreeMap<Ident, Vec<Regex>>,
}

pub fn parse(input: DeriveInput) -> Result<InputTokenRegexes, syn::Error> {
    let default_skip = RegexValue {
        span: Span::call_site(),
        regex: "<whitespace>*".to_owned(),
    };
    // get plain enum data
    let data = check_plain_enum(&input)?;
    // get the skip regex
    let skip_regex = get_skip_regex(&input.attrs)?.unwrap_or(default_skip);
    // get regex and tokens for all enum items
    let variants = get_variants(data)?;
    Ok(InputTokenRegexes {
        enum_name: input.ident,
        skip_regex,
        variants,
    })
}

/// Checks that the input represents an enum where all options have no data fields
fn check_plain_enum(input: &DeriveInput) -> Result<&DataEnum, Error> {
    // check that the input is an enum
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(Error::new(
                input.ident.span(),
                "#[derive(Lexer)] is only valid for enums.",
            ));
        }
    };
    // check all variants
    for item in data.variants.iter() {
        check_enum_item(item)?;
    }
    Ok(data)
}

fn check_enum_item(item: &Variant) -> Result<(), Error> {
    // check that the item has no data fields
    match item.fields {
        Fields::Unit => (),
        _ => {
            return Err(Error::new(
                item.ident.span(),
                format!(
                    "Lexer enum variants must be units (try \"{},\").",
                    item.ident
                ),
            ))
        }
    }
    // check that it does not define an explicit discriminant
    if item.discriminant.is_some() {
        return Err(Error::new(
            item.ident.span(),
            "Lexer enum variants must not define an explicit discriminant.",
        ));
    };
    Ok(())
}

fn get_skip_regex(attrs: &[Attribute]) -> Result<Option<RegexValue>, Error> {
    let mut skip_regex = None;
    for attr in attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "token" {
                return Err(Error::new(
                    attr.path.get_ident().unwrap().span(),
                    "#[token = ...] is not allowed at enum scope.",
                ));
            } else if ident == "regex" {
                return Err(Error::new(
                    attr.path.get_ident().unwrap().span(),
                    "#[regex = ...] is not allowed at enum scope.",
                ));
            } else if ident == "skip" {
                match attr.parse_meta()? {
                    Meta::NameValue(ref value) => {
                        if skip_regex.is_some() {
                            return Err(Error::new(
                                attr.path.get_ident().unwrap().span(),
                                "Multiple definitions of #[skip = ...].",
                            ));
                        } else {
                            skip_regex = Some(retreive_str(ident, value)?);
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            attr.path.get_ident().unwrap().span(),
                            "The skip regex must be defined as #[skip = ...]",
                        ))
                    }
                }
            }
        } else {
            continue;
        }
    }
    Ok(skip_regex)
}

fn get_variants(data: &DataEnum) -> Result<BTreeMap<Ident, Vec<Regex>>, Error> {
    let mut result = BTreeMap::new();
    for variant in &data.variants {
        let (key, value) = get_variant(variant)?;
        result.insert(key, value);
    }
    Ok(result)
}

fn get_variant(variant: &Variant) -> Result<(Ident, Vec<Regex>), Error> {
    let mut regex = vec![];
    for attr in &variant.attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "token" {
                match attr.parse_meta()? {
                    Meta::NameValue(ref value) => {
                        regex.push(Regex::Token(retreive_str(ident, value)?));
                    }
                    _ => {
                        return Err(Error::new(
                            attr.path.get_ident().unwrap().span(),
                            "Token specification must be in the format #[token = ...].",
                        ))
                    }
                }
            } else if ident == "regex" {
                match attr.parse_meta()? {
                    Meta::NameValue(ref value) => {
                        regex.push(Regex::Regex(retreive_str(ident, value)?));
                    }
                    _ => {
                        return Err(Error::new(
                            attr.path.get_ident().unwrap().span(),
                            "Regex specification must be in the format #[regex = ...].",
                        ))
                    }
                }
            } else if ident == "skip" {
                return Err(Error::new(
                    attr.path.get_ident().unwrap().span(),
                    "#[skip = ...] must be specified at enum level.",
                ));
            }
        } else {
            continue;
        }
    }
    Ok((variant.ident.clone(), regex))
}

fn retreive_str(attr: &Ident, value: &MetaNameValue) -> Result<RegexValue, Error> {
    match value.lit {
        Lit::Str(ref lit) => Ok(RegexValue {
            span: lit.span(),
            regex: lit.value(),
        }),
        Lit::Char(ref lit) => Ok(RegexValue {
            span: lit.span(),
            regex: lit.value().to_string(),
        }),
        _ => Err(Error::new(
            value.path.get_ident().unwrap().span(),
            format!("Attribute {} must be a string or char literal.", attr),
        )),
    }
}
