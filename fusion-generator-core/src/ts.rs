use std::rc::Rc;

use concat_string::concat_string;

pub trait AST: Sized {
  fn kind(&self) -> ASTKind;
}

pub trait Module<'a> {
  fn imports(&'a self) -> &'a Option<Rc<Vec<Import>>>;
  fn imports_codegen(&'a self) -> Option<String> {
    self.imports().as_ref().map(|list| {
      list
        .iter()
        .map(|i| i.codegen())
        .collect::<Vec<_>>()
        .join("\n")
    })
  }
}

macro_rules! impl_ast {
  ($($kind:tt),+) => {
    pub enum ASTKind<'a> {
      $($kind(&'a $kind)),+
    }

    $(
      impl AST for $kind {
        fn kind(&self) -> ASTKind {
          ASTKind::$kind(self)
        }
      }
    )+
  };
}

#[derive(Clone)]
pub struct Ident(Rc<String>);

impl Ident {
  pub fn new(name: &str) -> Self {
    Self(Rc::new(name.to_string()))
  }

  pub fn codegen(&self) -> String {
    (*self.0).clone()
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

  pub fn codegen(&self) -> String {
    format!(
      "import type {} from \"{}\";",
      self.symbol.codegen(),
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

pub struct RequiredType<'a> {
  r#type: &'a Type,
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

  pub fn codegen(&self) -> String {
    let name = self.name.codegen();
    let inner = self.inner_codegen();
    let tail = if self.optional { " | undefined" } else { "" };

    concat_string!(name, inner, tail)
  }

  pub fn required(&self) -> RequiredType {
    RequiredType { r#type: self }
  }

  #[inline]
  fn inner_codegen(&self) -> String {
    self
      .inner
      .as_ref()
      .map(|val| {
        let types = val
          .iter()
          .map(|t| t.codegen())
          .collect::<Vec<String>>()
          .join(", ");

        format!("<{}>", types)
      })
      .unwrap_or_default()
  }
}

impl RequiredType<'_> {
  pub fn codegen(&self) -> String {
    let name = self.r#type.name.codegen();
    let inner = self.r#type.inner_codegen();

    concat_string!(name, inner)
  }
}

#[derive(Clone)]
pub struct Field {
  name: Ident,
  r#type: Type,
}

impl Field {
  pub fn new(name: Ident, r#type: Type) -> Self {
    Self { name, r#type }
  }

  pub fn codegen(&self) -> String {
    let tail = if self.r#type.is_optional() { "?" } else { "" };

    format!(
      "  {}{}: {};",
      self.name.codegen(),
      tail,
      self.r#type.required().codegen()
    )
  }
}

#[derive(Clone)]
pub struct Struct {
  fields: Option<Rc<Vec<Field>>>,
  name: Ident,
}

impl Struct {
  pub fn new(name: Ident, fields: Option<Vec<Field>>) -> Self {
    Self {
      fields: fields.map(|val| Rc::new(val)),
      name,
    }
  }

  pub fn codegen(&self) -> String {
    let fields = self
      .fields
      .as_ref()
      .map(|items| {
        items
          .iter()
          .map(|f| f.codegen())
          .collect::<Vec<String>>()
          .join("\n")
      })
      .unwrap_or_default();

    format!(
      "export default interface {} {{\n{}\n}}",
      self.name.codegen(),
      fields
    )
  }
}

#[derive(Clone)]
pub struct StructModule {
  content: Option<Struct>,
  imports: Option<Rc<Vec<Import>>>,
  name: Ident,
}

impl StructModule {
  pub fn new(name: Ident, imports: Option<Vec<Import>>, content: Option<Struct>) -> Self {
    Self {
      content,
      imports: imports.map(|val| Rc::new(val)),
      name,
    }
  }

  pub fn codegen(&self, header: Option<&str>) -> String {
    let header = header
      .map(|val| {
        let sep = "\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

    let imports = self
      .imports_codegen()
      .map(|i| {
        let sep = "\n\n";
        concat_string!(i, sep)
      })
      .unwrap_or_default();

    let content = self
      .content
      .as_ref()
      .map(|d| d.codegen())
      .unwrap_or_default();

    concat_string!(header, imports, content)
  }
}

impl<'a> Module<'a> for StructModule {
  fn imports(&'a self) -> &'a Option<Rc<Vec<Import>>> {
    &self.imports
  }
}

#[derive(Clone)]
pub struct Parameter {
  name: Ident,
  r#type: Type,
}

impl Parameter {
  pub fn new(name: Ident, r#type: Type) -> Self {
    Self { r#type, name }
  }

  pub fn codegen(&self) -> String {
    format!("{}: {}", self.name.codegen(), self.r#type.codegen())
  }
}

#[derive(Clone)]
pub struct Method {
  name: Ident,
  parameters: Option<Rc<Vec<Parameter>>>,
  return_type: Type,
}

impl Method {
  pub fn new(name: Ident, parameters: Option<Vec<Parameter>>, return_type: Type) -> Self {
    Self {
      name,
      parameters: parameters.map(|val| Rc::new(val)),
      return_type,
    }
  }

  pub fn codegen(&self, endpoint_name: &Ident) -> String {
    let name = self.name.codegen();
    let parameters = self
      .parameters
      .as_ref()
      .map(|parameters| {
        parameters
          .iter()
          .map(|p| p.codegen())
          .collect::<Vec<String>>()
          .join(", ")
      })
      .unwrap_or_default();

    let parameter_names = self
      .parameters
      .as_ref()
      .map(|parameters| {
        let list = parameters
          .iter()
          .map(|p| p.name.codegen())
          .collect::<Vec<String>>()
          .join(", ");

        format!(", {{{}}}", list)
      })
      .unwrap_or_default();

    format!(
      "function _{}({}): Promise<{}> {{\n  client.call(\"{}\", \"{}\"{});\n}}",
      name,
      parameters,
      self.return_type.codegen(),
      endpoint_name.codegen(),
      name,
      parameter_names
    )
  }
}

#[derive(Clone)]
pub struct EndpointModule {
  content: Option<Rc<Vec<Method>>>,
  imports: Option<Rc<Vec<Import>>>,
  name: Ident,
}

impl EndpointModule {
  pub fn new(name: Ident, imports: Option<Vec<Import>>, content: Option<Vec<Method>>) -> Self {
    Self {
      content: content.map(|val| Rc::new(val)),
      imports: imports.map(|val| Rc::new(val)),
      name,
    }
  }

  pub fn codegen(&self, header: Option<&str>) -> String {
    let header = header
      .map(|val| {
        let sep = "\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

    let imports = self
      .imports_codegen()
      .map(|val| {
        let sep = "\n\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

    let items = self
      .content
      .as_ref()
      .map(|items| {
        let combined: String = items
          .iter()
          .map(|i| i.codegen(&self.name))
          .collect::<Vec<String>>()
          .join("\n\n");

        let sep = "\n\n";
        concat_string!(combined, sep)
      })
      .unwrap_or_default();

    let exports = self
      .content
      .as_ref()
      .map(|items| {
        let combined = items
          .iter()
          .map(|i| {
            let name = i.name.codegen();
            format!("  _{n} as {n},", n = name)
          })
          .collect::<Vec<String>>()
          .join("\n");

        format!("export {{\n{}\n}};", combined)
      })
      .unwrap_or_default();

    concat_string!(header, imports, items, exports)
  }
}

impl<'a> Module<'a> for EndpointModule {
  fn imports(&'a self) -> &'a Option<Rc<Vec<Import>>> {
    &self.imports
  }
}

#[derive(Clone)]
pub struct EnumVariant(Ident);

impl EnumVariant {
  pub fn new(name: Ident) -> Self {
    Self(name)
  }

  pub fn codegen(&self) -> String {
    let name = self.0.codegen();
    format!("{n} = '{n}'", n = name)
  }
}

#[derive(Clone)]
pub struct Enum {
  variants: Option<Rc<Vec<EnumVariant>>>,
  name: Ident,
}

impl Enum {
  pub fn new(name: Ident, variants: Option<Vec<EnumVariant>>) -> Self {
    Self {
      variants: variants.map(|val| Rc::new(val)),
      name,
    }
  }

  pub fn codegen(&self) -> String {
    let variants = self
      .variants
      .as_ref()
      .map(|items| {
        items
          .iter()
          .map(|val| format!("  {},", val.codegen()))
          .collect::<Vec<String>>()
          .join("\n")
      })
      .unwrap_or_default();

    format!(
      "export default enum {} {{\n{}\n}}",
      self.name.codegen(),
      variants,
    )
  }
}

#[derive(Clone)]
pub struct EnumModule {
  content: Option<Enum>,
  name: Ident,
}

impl EnumModule {
  pub fn new(name: Ident, content: Option<Enum>) -> Self {
    Self { content, name }
  }

  pub fn codegen(&self, header: Option<&str>) -> String {
    let header = header
      .map(|val| {
        let sep = "\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

    let content = self
      .content
      .as_ref()
      .map(|val| val.codegen())
      .unwrap_or_default();

    let sep = "\n";
    concat_string!(header, sep, content)
  }
}

impl_ast!(
  Ident,
  Import,
  Type,
  Field,
  Struct,
  StructModule,
  Parameter,
  Method,
  EndpointModule,
  EnumVariant,
  Enum,
  EnumModule
);

#[cfg(test)]
mod tests {
  use crate::ts::{
    EndpointModule, Enum, EnumModule, EnumVariant, Field, Ident, Import, Method, Parameter, Struct,
    StructModule, Type,
  };

  #[test]
  fn should_generate_code_for_struct_module() {
    let struct_mod = StructModule::new(
      Ident::new("ChartSeries"),
      None,
      Some(Struct::new(
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

    let code = struct_mod.codegen(Some("/**\n * Some header\n */"));
    assert_eq!(
      code,
      "/**
 * Some header
 */
export default interface ChartSeries {
  data?: Array<number | undefined>;
  name: string;
}"
    )
  }

  #[test]
  fn should_generate_code_for_endpoint_module() {
    let health_grid_item_struct_ident = Ident::new("HealthGridItem");
    let chart_series_struct_ident = Ident::new("ChartsSeries");
    let array_ident = Ident::new("Array");

    let endpoint_mod = EndpointModule::new(
      Ident::new("DashboardEndpoint"),
      Some(vec![
        Import::new(
          chart_series_struct_ident.clone(),
          "./com/example/application/views/dashboard/ChartSeries",
        ),
        Import::new(
          health_grid_item_struct_ident.clone(),
          "./com/example/application/views/dashboard/HealthGridItem",
        ),
      ]),
      Some(vec![
        Method::new(
          Ident::new("healthGridItems"),
          None,
          Type::new(
            array_ident.clone(),
            false,
            Some(vec![Type::new(
              health_grid_item_struct_ident.clone(),
              false,
              None,
            )]),
          ),
        ),
        Method::new(
          Ident::new("monthlyVisitorSeries"),
          Some(vec![
            Parameter::new(
              Ident::new("id"),
              Type::new(Ident::new("number"), true, None),
            ),
            Parameter::new(
              Ident::new("optional"),
              Type::new(Ident::new("boolean"), false, None),
            ),
          ]),
          Type::new(
            array_ident.clone(),
            true,
            Some(vec![Type::new(
              chart_series_struct_ident.clone(),
              true,
              None,
            )]),
          ),
        ),
      ]),
    );

    let code = endpoint_mod.codegen(Some("/**\n * Some header\n */"));
    assert_eq!(
      code,
      "/**
 * Some header
 */
import type ChartsSeries from \"./com/example/application/views/dashboard/ChartSeries\";
import type HealthGridItem from \"./com/example/application/views/dashboard/HealthGridItem\";

function _healthGridItems(): Promise<Array<HealthGridItem>> {
  client.call(\"DashboardEndpoint\", \"healthGridItems\");
}

function _monthlyVisitorSeries(id: number | undefined, optional: boolean): Promise<Array<ChartsSeries | undefined> | undefined> {
  client.call(\"DashboardEndpoint\", \"monthlyVisitorSeries\", {id, optional});
}

export {
  _healthGridItems as healthGridItems,
  _monthlyVisitorSeries as monthlyVisitorSeries,
};"
    )
  }

  #[test]
  fn should_generate_code_for_enum_module() {
    let name = Ident::new("MyEnum");

    let enum_mod = EnumModule::new(
      name.clone(),
      Some(Enum::new(
        name.clone(),
        Some(vec![
          EnumVariant::new(Ident::new("VAL1")),
          EnumVariant::new(Ident::new("VAL2")),
          EnumVariant::new(Ident::new("VAL3")),
        ]),
      )),
    );

    let code = enum_mod.codegen(Some("/**\n * Some header\n */"));

    assert_eq!(
      code,
      "/**
 * Some header
 */

export default enum MyEnum {
  VAL1 = 'VAL1',
  VAL2 = 'VAL2',
  VAL3 = 'VAL3',
}"
    );
  }
}
