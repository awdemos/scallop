//! Minimal host types the `AstNode` derive expects to be in scope, modeled
//! after `core/src/compiler/front/ast/utils.rs`.
use astnode_derive::AstNode;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeLocation;

impl NodeLocation {
  pub fn from_span(_start: usize, _end: usize) -> Self {
    Self
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstNodeWrapper<T> {
  pub _loc: NodeLocation,
  pub _node: T,
}

#[allow(unused)]
pub trait AstWalker {
  fn walk<V>(&self, v: &mut V);
  fn walk_mut<V>(&mut self, v: &mut V);
}

#[allow(unused)]
pub trait NodeVisitor<X> {
  fn visit(&mut self, _: &X) {}
  fn visit_mut(&mut self, _: &mut X) {}
}

impl<U, X> NodeVisitor<X> for U {}

#[derive(Clone, Debug, PartialEq, AstNode)]
#[doc(hidden)]
pub enum _Binding {
  Bound,
  Free,
}

#[test]
fn test_last_unit_variant_with_trailing_comma() {
  let bound = Binding::bound();
  assert!(bound.is_bound());
  assert!(!bound.is_free());

  let free = Binding::free();
  assert!(free.is_free());
  assert!(!free.is_bound());
}

#[derive(Clone, Debug, PartialEq, AstNode)]
#[doc(hidden)]
pub enum _NoComma {
  First,
  Second,
}

#[test]
fn test_last_unit_variant_without_trailing_comma() {
  let first = NoComma::first();
  assert!(first.is_first());
  assert!(!first.is_second());

  // Regression: the last unit variant used to be silently dropped by the
  // derive when the enum had no trailing comma.
  let second = NoComma::second();
  assert!(second.is_second());
  assert!(!second.is_first());
}
