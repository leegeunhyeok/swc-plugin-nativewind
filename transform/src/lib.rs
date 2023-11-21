use swc_core::ecma::visit::{as_folder, Fold, VisitMut};

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {}

pub fn nativewind() -> impl VisitMut + Fold {
    as_folder(TransformVisitor {})
}
