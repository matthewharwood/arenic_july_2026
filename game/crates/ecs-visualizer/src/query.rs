use std::collections::BTreeMap;

use quote::ToTokens;
use serde::Serialize;
use syn::{
    FnArg, GenericArgument, ItemFn, ItemType, Pat, PathArguments, PathSegment, Type,
    visit::{self, Visit},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AccessMode {
    Read,
    Write,
    With,
    Without,
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Rule {
    pub(crate) component: String,
    pub(crate) mode: AccessMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Query {
    pub(crate) system: String,
    pub(crate) parameter: String,
    pub(crate) kind: String,
    pub(crate) signature: String,
    pub(crate) rules: Vec<Rule>,
}

pub(crate) fn extract(source: &str) -> syn::Result<Vec<Query>> {
    let file = syn::parse_file(source)?;
    let mut aliases = AliasCollector::default();
    aliases.visit_file(&file);

    let mut collector = QueryCollector {
        aliases: &aliases.types,
        queries: Vec::new(),
        error: None,
    };
    collector.visit_file(&file);
    if let Some(error) = collector.error {
        return Err(error);
    }
    if collector.queries.is_empty() {
        return Err(syn::Error::new_spanned(
            &file,
            "no Query or Single system parameters found",
        ));
    }
    Ok(collector.queries)
}

#[derive(Default)]
struct AliasCollector {
    types: BTreeMap<String, Type>,
}

impl<'ast> Visit<'ast> for AliasCollector {
    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.types
            .insert(item.ident.to_string(), item.ty.as_ref().clone());
        visit::visit_item_type(self, item);
    }
}

struct QueryCollector<'a> {
    aliases: &'a BTreeMap<String, Type>,
    queries: Vec<Query>,
    error: Option<syn::Error>,
}

impl<'ast> Visit<'ast> for QueryCollector<'_> {
    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        for argument in &function.sig.inputs {
            match parse_argument(function, argument, self.aliases) {
                Some(Ok(query)) => self.queries.push(query),
                Some(Err(error)) if self.error.is_none() => self.error = Some(error),
                _ => {}
            }
        }
        visit::visit_item_fn(self, function);
    }
}

fn parse_argument(
    function: &ItemFn,
    argument: &FnArg,
    aliases: &BTreeMap<String, Type>,
) -> Option<syn::Result<Query>> {
    let FnArg::Typed(argument) = argument else {
        return None;
    };
    let Type::Path(query_type) = argument.ty.as_ref() else {
        return None;
    };
    let segment = query_type.path.segments.last()?;
    let kind = match segment.ident.to_string().as_str() {
        "Query" => "Query",
        "Single" => "Single",
        _ => return None,
    };

    Some((|| {
        let Pat::Ident(parameter) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "query parameters must use an identifier pattern",
            ));
        };
        let mut types = type_arguments(segment)?;
        let data = types
            .next()
            .ok_or_else(|| syn::Error::new_spanned(segment, "query data type is missing"))?;
        let mut rules = Vec::new();
        collect_rules(data, AccessMode::Read, aliases, &mut rules)?;
        if let Some(filter) = types.next() {
            collect_rules(filter, AccessMode::With, aliases, &mut rules)?;
        }

        Ok(Query {
            system: function.sig.ident.to_string(),
            parameter: parameter.ident.to_string(),
            kind: kind.to_owned(),
            signature: compact_type(argument.ty.as_ref()),
            rules,
        })
    })())
}

fn collect_rules(
    ty: &Type,
    mode: AccessMode,
    aliases: &BTreeMap<String, Type>,
    rules: &mut Vec<Rule>,
) -> syn::Result<()> {
    match ty {
        Type::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_rules(element, mode, aliases, rules)?;
            }
        }
        Type::Reference(reference) => {
            let mode = if reference.mutability.is_some() {
                AccessMode::Write
            } else {
                mode
            };
            collect_rules(reference.elem.as_ref(), mode, aliases, rules)?;
        }
        Type::Paren(paren) => collect_rules(paren.elem.as_ref(), mode, aliases, rules)?,
        Type::Group(group) => collect_rules(group.elem.as_ref(), mode, aliases, rules)?,
        Type::Path(path) if path.qself.is_none() => {
            let segment = path
                .path
                .segments
                .last()
                .ok_or_else(|| syn::Error::new_spanned(path, "component path is empty"))?;
            let name = segment.ident.to_string();
            if let Some(alias) = aliases.get(&name)
                && matches!(segment.arguments, PathArguments::None)
            {
                collect_rules(alias, mode, aliases, rules)?;
            } else if let Some(wrapper_mode) = wrapper_mode(&name) {
                let inner = type_arguments(segment)?.next().ok_or_else(|| {
                    syn::Error::new_spanned(segment, format!("{name} requires a type"))
                })?;
                collect_rules(inner, wrapper_mode, aliases, rules)?;
            } else {
                let rule = Rule {
                    component: name,
                    mode,
                };
                if !rules.contains(&rule) {
                    rules.push(rule);
                }
            }
        }
        _ => {
            return Err(syn::Error::new_spanned(
                ty,
                "unsupported ECS query type expression",
            ));
        }
    }
    Ok(())
}

fn type_arguments(segment: &PathSegment) -> syn::Result<impl Iterator<Item = &Type>> {
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            segment,
            "expected generic type arguments",
        ));
    };
    Ok(arguments.args.iter().filter_map(|argument| match argument {
        GenericArgument::Type(ty) => Some(ty),
        _ => None,
    }))
}

fn wrapper_mode(name: &str) -> Option<AccessMode> {
    match name {
        "With" => Some(AccessMode::With),
        "Without" => Some(AccessMode::Without),
        "Option" => Some(AccessMode::Optional),
        _ => None,
    }
}

fn compact_type(ty: &Type) -> String {
    ty.to_token_stream()
        .to_string()
        .replace("& mut ", "&mut ")
        .replace("& ", "&")
        .replace(" < ", "<")
        .replace(" >", ">")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" ,", ",")
        .replace(",)", ")")
        .replace(",>", ">")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAMERA_SOURCE: &str = include_str!("../../game/src/camera.rs");

    #[test]
    fn extracts_each_camera_query_parameter() {
        let queries = extract(CAMERA_SOURCE).expect("camera queries should parse");
        let attach_queries = queries
            .iter()
            .filter(|query| query.system == "attach_over_shoulder_camera")
            .count();
        let toggle_queries = queries
            .iter()
            .filter(|query| query.system == "toggle_camera_view")
            .count();

        assert_eq!(queries.len(), 5);
        assert_eq!(attach_queries, 2);
        assert_eq!(toggle_queries, 3);
    }

    #[test]
    fn expands_aliases_and_preserves_access_modes() {
        let queries = extract(CAMERA_SOURCE).expect("camera queries should parse");
        let active_hero = queries
            .iter()
            .find(|query| query.parameter == "active_selected_hero")
            .expect("active hero query should exist");
        let top_down_camera = queries
            .iter()
            .find(|query| query.parameter == "top_down_camera")
            .expect("top-down camera query should exist");

        assert_eq!(
            active_hero.rules,
            [
                rule("Entity", AccessMode::Read),
                rule("Hero", AccessMode::With),
                rule("Selected", AccessMode::With),
                rule("Active", AccessMode::With),
            ]
        );
        assert!(
            top_down_camera
                .rules
                .contains(&rule("Camera", AccessMode::Write))
        );
        assert!(
            top_down_camera
                .rules
                .contains(&rule("OverShoulderCamera", AccessMode::Without))
        );
    }

    fn rule(component: &str, mode: AccessMode) -> Rule {
        Rule {
            component: component.to_owned(),
            mode,
        }
    }
}
