use std::rc::Rc;

use concat_string::concat_string;

use crate::ast::AST;
use crate::ts::TSConvertable;

#[derive(Clone)]
pub struct Ident {
  name: Rc<String>,
}

impl Ident {
  pub fn new(name: &str) -> Self {
    Self {
      name: Rc::new(String::from(name)),
    }
  }
}

impl AST for Ident {}

impl TSConvertable for Ident {
  fn as_ts(&self) -> String {
    (*self.name).clone()
  }
}

#[derive(Clone)]
pub struct Import {
  source: Rc<String>,
  symbol: Ident,
}

impl Import {
  pub fn new(symbol: Ident, source: &str) -> Self {
    Self {
      source: Rc::new(source.to_string()),
      symbol,
    }
  }
}

impl AST for Import {}

impl TSConvertable for Import {
  fn as_ts(&self) -> String {
    format!(
      "import type {} from \"{}\";",
      self.symbol.as_ts(),
      self.source
    )
  }
}

#[derive(Clone)]
pub struct Type {
  optional: bool,
  inner: Option<Rc<Vec<Type>>>,
  name: Ident,
}

impl Type {
  pub fn new(name: Ident, optional: bool, inner: Option<Vec<Type>>) -> Self {
    Self {
      optional,
      inner: inner.map(|val| Rc::new(val)),
      name,
    }
  }

  pub fn is_optional(&self) -> bool {
    self.optional
  }

  pub(crate) fn without_optional(&self) -> NonOptionalType {
    NonOptionalType { r#type: self }
  }

  #[inline]
  fn inner_as_ts(&self) -> String {
    self
      .inner
      .as_ref()
      .map(|val| {
        let types: Vec<String> = val.iter().map(|t| t.as_ts()).collect();

        format!("<{}>", types.join(", "))
      })
      .unwrap_or_default()
  }
}

impl AST for Type {}

impl TSConvertable for Type {
  fn as_ts(&self) -> String {
    let tail = if self.optional { " | undefined" } else { "" };
    concat_string!(&self.name.as_ts(), &self.inner_as_ts(), tail)
  }
}

pub(crate) struct NonOptionalType<'a> {
  r#type: &'a Type,
}

impl TSConvertable for NonOptionalType<'_> {
  fn as_ts(&self) -> String {
    concat_string!(&self.r#type.name.as_ts(), &self.r#type.inner_as_ts())
  }
}
