use std::rc::Rc;

use concat_string::concat_string;

use crate::ast::{File, FileWithOptions, Ident, Import, Type, AST};
use crate::ts::TSConvertable;

#[derive(Clone)]
pub struct Field {
  name: Ident,
  r#type: Type,
}

impl Field {
  pub fn new(name: Ident, r#type: Type) -> Self {
    Self { name, r#type }
  }
}

impl AST for Field {}

impl TSConvertable for Field {
  fn as_ts(&self) -> String {
    let tail = if self.r#type.is_optional() { "?" } else { "" };

    format!(
      "  {}{}: {};",
      self.name.as_ts(),
      tail,
      self.r#type.without_optional().as_ts()
    )
  }
}

#[derive(Clone)]
pub struct Declaration {
  fields: Option<Rc<Vec<Field>>>,
  name: Ident,
}

impl Declaration {
  pub fn new(name: Ident, fields: Option<Vec<Field>>) -> Self {
    Self {
      fields: fields.map(|val| Rc::new(val)),
      name,
    }
  }
}

impl AST for Declaration {}

impl TSConvertable for Declaration {
  fn as_ts(&self) -> String {
    let fields: Option<Vec<String>> = self
      .fields
      .as_ref()
      .map(|list| list.iter().map(|f| f.as_ts()).collect());

    format!(
      "export default interface {} {{\n{}\n}}",
      self.name.as_ts(),
      fields.map(|list| list.join("\n")).unwrap_or_default()
    )
  }
}

pub type DeclarationFile = File<Declaration>;
type DeclarationFileWithOptions<'a> = FileWithOptions<'a, Declaration>;

impl DeclarationFile {
  pub fn new(name: Ident, imports: Option<Vec<Import>>, content: Option<Declaration>) -> Self {
    Self {
      content,
      imports: imports.map(|val| Rc::new(val)),
      name,
    }
  }
}

impl TSConvertable for DeclarationFileWithOptions<'_> {
  fn as_ts(&self) -> String {
    concat_string!(
      &self
        .header
        .as_ref()
        .map(|val| concat_string!(&val, "\n"))
        .unwrap_or_default(),
      &self
        .imports_as_ts()
        .map(|i| concat_string!(&i, "\n\n"))
        .unwrap_or_default(),
      &self
        .file
        .content
        .as_ref()
        .map(|d| d.as_ts())
        .unwrap_or_default()
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::declaration::{Declaration, DeclarationFile, Field};
  use crate::ast::{Ident, Type};
  use crate::ts::TSConvertable;

  #[test]
  fn should_generate_code() {
    let ast = DeclarationFile::new(
      Ident::new("ChartSeries"),
      None,
      Some(Declaration::new(
        Ident::new("ChartSeries"),
        Some(vec![
          Field::new(
            Ident::new("data"),
            Type::new(
              Ident::new("Array"),
              true,
              Some(vec![Type::new(Ident::new("number"), true, None)]),
            ),
          ),
          Field::new(
            Ident::new("name"),
            Type::new(Ident::new("string"), false, None),
          ),
        ]),
      )),
    );

    let code = ast.with_options(Some("/**\n * Some header\n */")).as_ts();
    assert_eq!(
      code,
      "\
/**
 * Some header
 */
export default interface ChartSeries {
  data?: Array<number | undefined>;
  name: string;
}"
    )
  }
}
