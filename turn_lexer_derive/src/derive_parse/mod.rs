extern crate proc_macro;

use std::collections::BTreeMap;
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Error, Fields, Ident, Lit, Meta, MetaNameValue, Variant,
};

pub enum VariantRegex {
    Token(String),
    Regex(String),
}

pub struct TerminalEnum {
    pub name: Ident,
    pub skip_regex: String,
    pub variants: BTreeMap<Ident, Vec<VariantRegex>>,
}

pub fn parse(input: DeriveInput) -> Result<TerminalEnum, syn::Error> {
    // get plain enum data
    let data = check_plain_enum(&input)?;
    // get the skip regex
    let skip_regex = get_skip_regex(&input.attrs)?.unwrap_or("<whitespace>*".to_owned());
    // get regex and tokens for all enum items
    let variants = get_variants(data)?;
    Ok(TerminalEnum {
        name: input.ident,
        skip_regex,
        variants,
    })
}

/// Checks that the input represents an enum where all options have no data fields
fn check_plain_enum(input: &DeriveInput) -> Result<&DataEnum, Error> {
    // check that the input is an enum
    let ref data = match &input.data {
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
                format!("Enum item {} must not contain data fields.", item.ident),
            ))
        }
    }
    // check that it does not define an explicit discriminant
    if let Some(_) = item.discriminant {
        return Err(Error::new(
            item.ident.span(),
            format!(
                "Enum item {} must not define an explicit discriminant.",
                item.ident
            ),
        ));
    };
    Ok(())
}

fn get_skip_regex(attrs: &Vec<Attribute>) -> Result<Option<String>, Error> {
    let mut skip_regex = None;
    for ref attr in attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "token" {
                return Err(Error::new(
                    ident.span(),
                    "#[token = \"...\"] not allowed at enum scope.",
                ));
            } else if ident == "regex" {
                return Err(Error::new(
                    ident.span(),
                    "#[regex = \"...\"] not allowed at enum scope.",
                ));
            } else if ident == "skip" {
                match attr.parse_meta()? {
                    Meta::NameValue(ref value) => {
                        if let Some(_) = skip_regex {
                            return Err(Error::new(
                                ident.span(),
                                "Multiple definitions of #[skip = \"...\"].",
                            ));
                        } else {
                            skip_regex = Some(retreive_str(ident, value)?);
                        }
                    }
                    _ => {
                        return Err(Error::new(
                            ident.span(),
                            "The skip regex must be defined as #[skip = \"...\"]",
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

fn get_variants(data: &DataEnum) -> Result<BTreeMap<Ident, Vec<VariantRegex>>, Error> {
    let mut result = BTreeMap::new();
    for ref variant in &data.variants {
        let (key, value) = get_variant(variant)?;
        result.insert(key, value);
    }
    Ok(result)
}

fn get_variant(variant: &Variant) -> Result<(Ident, Vec<VariantRegex>), Error> {
    let mut regex = vec![];
    for attr in &variant.attrs {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "token" {
                match attr.parse_meta()? {
                    Meta::NameValue(ref value) => {
                        regex.push(VariantRegex::Token(retreive_str(ident, value)?));
                    }
                    _ => {
                        return Err(Error::new(
                            variant.ident.span(),
                            "Token specification must be in the format #[token = \"...\"].",
                        ))
                    }
                }
            } else if ident == "regex" {
                match attr.parse_meta()? {
                    Meta::NameValue(ref value) => {
                        regex.push(VariantRegex::Regex(retreive_str(ident, value)?));
                    }
                    _ => {
                        return Err(Error::new(
                            variant.ident.span(),
                            "Regex specification must be in the format #[regex = \"...\"].",
                        ))
                    }
                }
            } else if ident == "skip" {
                return Err(Error::new(
                    variant.ident.span(),
                    "#[skip = \"...\"] must be specified at enum level.",
                ));
            }
        } else {
            continue;
        }
    }
    if regex.is_empty() {
        return Err(Error::new(
            variant.ident.span(),
            "At least one #[token = \"...\"] or #[regex = \"...\"] 
            must be specified for each variant.",
        ));
    }
    Ok((variant.ident.clone(), regex))
}

fn retreive_str(attr: &Ident, value: &MetaNameValue) -> Result<String, Error> {
    match value.lit {
        Lit::Str(ref lit) => Ok(lit.value()),
        Lit::Char(ref lit) => Ok(lit.value().to_string()),
        _ => Err(Error::new(
            value.path.get_ident().unwrap().span(),
            format!("Attribute {} must be a string or char literal.", attr),
        )),
    }
}
