// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::str::FromStr;
use qsc_ast::{
    ast::{Attr, ExprKind, ItemKind, Namespace},
    mut_visit::MutVisitor,
};
use std::rc::Rc;

use super::TargetProfile;

#[derive(PartialEq, Hash, Clone, Debug)]
pub struct TrackedName {
    pub name: Rc<str>,
    pub namespace: Rc<str>,
}

pub(crate) struct Conditional {
    target: TargetProfile,
    dropped_names: Vec<TrackedName>,
    included_names: Vec<TrackedName>,
}

impl Conditional {
    pub(crate) fn new(target: TargetProfile) -> Self {
        Self {
            target,
            dropped_names: Vec::new(),
            included_names: Vec::new(),
        }
    }

    pub(crate) fn into_names(self) -> Vec<TrackedName> {
        self.dropped_names
            .into_iter()
            .filter(|n| !self.included_names.contains(n))
            .collect()
    }
}

impl MutVisitor for Conditional {
    fn visit_namespace(&mut self, namespace: &mut Namespace) {
        namespace.items = namespace
            .items
            .iter()
            .filter_map(|item| {
                if matches_target(&item.attrs, self.target) {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => {
                            self.included_names.push(TrackedName {
                                name: callable.name.name.clone(),
                                namespace: namespace.name.name.clone(),
                            });
                        }
                        ItemKind::Ty(ident, _) => self.included_names.push(TrackedName {
                            name: ident.name.clone(),
                            namespace: namespace.name.name.clone(),
                        }),
                        _ => {}
                    }
                    Some(item.clone())
                } else {
                    match item.kind.as_ref() {
                        ItemKind::Callable(callable) => {
                            self.dropped_names.push(TrackedName {
                                name: callable.name.name.clone(),
                                namespace: namespace.name.name.clone(),
                            });
                        }
                        ItemKind::Ty(ident, _) => self.dropped_names.push(TrackedName {
                            name: ident.name.clone(),
                            namespace: namespace.name.name.clone(),
                        }),
                        _ => {}
                    }
                    None
                }
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
    }
}

fn matches_target(attrs: &[Box<Attr>], target: TargetProfile) -> bool {
    attrs.iter().all(|attr| {
        if attr.name.name.as_ref() == "Config" {
            if let ExprKind::Paren(inner) = attr.arg.kind.as_ref() {
                match inner.kind.as_ref() {
                    ExprKind::Path(path) => {
                        match TargetProfile::from_str(path.name.name.as_ref()) {
                            Ok(t) => t == target,
                            Err(_) => true,
                        }
                    }
                    _ => true,
                }
            } else {
                true
            }
        } else {
            true
        }
    })
}
