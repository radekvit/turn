use crate::matchers::{CharacterCategory, SingleMatcher};
use crate::regex::hir;
use crate::regex::mir;
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};

pub struct CategoryRegistry<'a> {
    builtin: HashMap<&'a str, CharacterCategory>,
    categories: HashMap<&'a str, Vec<mir::SetMember>>,
}
pub struct RegexRegistry<'a> {
    builtin: HashMap<&'a str, CharacterCategory>,
    categories: HashMap<&'a str, Vec<mir::SetMember>>,
    regexes: HashMap<&'a str, mir::MIR<'a>>,
}

pub fn convert_categories<'a>(
    mut categories: HashMap<&'a str, Vec<hir::SetMember>>,
    builtin: HashMap<&'a str, CharacterCategory>,
) -> Result<CategoryRegistry<'a>, ()> {
    let mut registry = CategoryRegistry {
        categories: HashMap::new(),
        builtin,
    };

    let mut dependencies: HashMap<_, HashSet<_>> = categories
        .iter()
        .map(|(k, v)| {
            (
                *k,
                dependencies(v)
                    .into_iter()
                    .filter(|x| registry.builtin.get(x).is_none())
                    .collect(),
            )
        })
        .collect();
    while !categories.is_empty() {
        // find all keys without dependencies, and add them to the registry
        let free_categories: Vec<_> = dependencies
            .iter()
            .filter_map(|(k, v)| if v.is_empty() { Some(*k) } else { None })
            .collect();
        // There are categories that have not been compiled, but still have unresolved dependencies.
        if free_categories.is_empty() {
            // TODO emit nice error
            return Err(());
        }
        free_categories.iter().for_each(|category| {
            let mir_category = create_category(categories.get(category).unwrap(), &registry);
            registry.categories.insert(category, mir_category);
            categories.remove(category);
            dependencies.remove(category);
            dependencies.iter_mut().for_each(|(_, dependency)| {
                dependency.remove(category);
            });
        });
    }

    Ok(registry)
}

fn create_category<'a>(
    category: &Vec<hir::SetMember>,
    registry: &CategoryRegistry<'a>,
) -> Vec<mir::SetMember> {
    unimplemented!()
}

fn dependencies<'a>(category: &Vec<hir::SetMember<'a>>) -> HashSet<&'a str> {
    category
        .iter()
        .filter_map(|x| match x {
            hir::SetMember::Category(c) => Some(*c),
            _ => None,
        })
        .collect()
}
/*
pub fn hir_to_mir<'a, 'b>(
    registry: RegexRegistry<'b>,
    hirs: HashMap<&'b str, hir::HIR<'a>>,
) -> Result<HashMap<&'b str, hir::HIR<'a>>, ()> {

}*/
/*
impl<'a> TryFrom<MIRCatalogue<'a, '_>> for mir::MIR<'a> {
    type Error = hir::HIR<'a>;

    fn try_from(value: MIRCatalogue<'a, '_>) -> Result<Self, Self::Error> {
        let MIRCatalogue {
            hir,
            categories,
            regexes,
        } = value;
        match hir {
            hir::HIR::AnyChar => Ok(mir::MIR::Category(CharacterCategory::Any)),
            hir::HIR::Sequence(sequence) => Ok(mir::MIR::Sequence(sequence)),
            // todo figure out the error
            //hir::HIR::SubRegex(name) => Ok(value.1.get(name).ok_or(value.0)?.clone()),
            hir::HIR::Repetition { regex, min, max } => Ok(mir::MIR::Repetition {
                regex: Box::new(
                    MIRCatalogue {
                        hir: *regex,
                        categories: value.categories,
                        regexes: value.regexes,
                    }
                    .try_into()?,
                ),
                min,
                max,
            }),
            hir::HIR::Alternation(alternatives) => Ok(mir::MIR::Alternation(
                alternatives
                    .into_iter()
                    .map(|hir| {
                        MIRCatalogue {
                            hir,
                            categories: categories,
                            regexes: regexes,
                        }
                        .try_into()
                    })
                    .collect::<Result<_, _>>()?,
            )),
            hir::HIR::Set(members) => todo!(),
            _ => todo!(),
        }
    }
}

fn get_subregex<'a, 'b>(name: &str, catalogue: &MIRCatalogue<'a, 'b>) -> Result<mir::MIR<'a>, ()> {
    if let Some(category) = builtin_categories.get(name) {
        Ok(mir::MIR::Category(*category))
    } else if let Some(members) = catalogue.categories.get(name) {
        Ok(mir::MIR::Set(members.clone()))
    } else if let Some(mir) = catalogue.regexes.get(name) {
        Ok(mir.clone())
    } else {
        Err(())
    }
}

fn get_category<'a, 'b>(
    name: &str,
    catalogue: &MIRCatalogue<'a, 'b>,
) -> Result<Vec<mir::SetMember>, ()> {
    if let Some(category) = builtin_categories.get(name) {
        Ok(vec![mir::SetMember::Category(*category)])
    } else if let Some(members) = catalogue.categories.get(name) {
        Ok(members.clone())
    } else {
        Err(())
    }
}
*/
impl<'a> From<&mir::SetMember> for SingleMatcher {
    fn from(value: &mir::SetMember) -> Self {
        match value {
            mir::SetMember::Character(c) => SingleMatcher::Character(*c),
            mir::SetMember::Category(category) => SingleMatcher::Category(*category),
        }
    }
}
