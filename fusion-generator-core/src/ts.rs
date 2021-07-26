use std::rc::Rc;

use concat_string::concat_string;

pub trait AST: Sized {
  fn kind(&self) -> ASTKind;
}

pub trait File {
  fn codegen(&self) -> String;
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
}

#[derive(Clone)]
pub struct EnumVariant(Ident);

impl EnumVariant {
  fn new(name: Ident) -> Self {
    Self(name)
  }
}

#[derive(Clone)]
pub struct Enum {
  variants: Option<Rc<Vec<EnumVariant>>>,
  name: Ident,
}

impl Enum {
  fn new(name: Ident, variants: Option<Vec<EnumVariant>>) -> Self {
    Self {
      variants: variants.map(|val| Rc::new(val)),
      name,
    }
  }
}

#[derive(Clone)]
pub struct EnumModule {
  content: Option<Enum>,
  name: Ident,
}

impl EnumModule {
  fn new(name: Ident, content: Option<Enum>) -> Self {
    Self { content, name }
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

enum CodegenOpts<'a> {
  TypeWithoutOptional(bool),
  ModuleHeader(&'a str),
  MethodEndpointName(&'a Ident),
  None,
}

#[inline]
fn codegen_imports(imports: &Option<Rc<Vec<Import>>>) -> Option<String> {
  imports.as_ref().map(|list| {
    list
      .iter()
      .map(|i| codegen(i, CodegenOpts::None))
      .collect::<Vec<String>>()
      .join("\n")
  })
}

#[inline]
fn codegen<T: AST>(ast: &T, opts: CodegenOpts) -> String {
  match ast.kind() {
    ASTKind::Ident(node) => (*node.0).clone(),
    ASTKind::Import(node) => {
      format!(
        "import type {} from \"{}\";",
        codegen(&node.symbol, CodegenOpts::None),
        node.source
      )
    }
    ASTKind::Type(node) => {
      let without_optional = match opts {
        CodegenOpts::TypeWithoutOptional(val) => val,
        _ => false,
      };

      let name = codegen(&node.name, CodegenOpts::None);

      let tail = if !without_optional && node.optional {
        " | undefined"
      } else {
        ""
      };

      let inner = node
        .inner
        .as_ref()
        .map(|val| {
          let types = val
            .iter()
            .map(|t| codegen(t, CodegenOpts::None))
            .collect::<Vec<String>>()
            .join(", ");

          format!("<{}>", types)
        })
        .unwrap_or_default();

      concat_string!(name, inner, tail)
    }
    ASTKind::Field(node) => {
      let tail = if node.r#type.is_optional() { "?" } else { "" };

      format!(
        "  {}{}: {};",
        codegen(&node.name, CodegenOpts::None),
        tail,
        codegen(&node.r#type, CodegenOpts::TypeWithoutOptional(true))
      )
    }
    ASTKind::Struct(node) => {
      let fields: Option<Vec<String>> = node
        .fields
        .as_ref()
        .map(|list| list.iter().map(|f| codegen(f, CodegenOpts::None)).collect());

      format!(
        "export default interface {} {{\n{}\n}}",
        codegen(&node.name, CodegenOpts::None),
        fields.map(|list| list.join("\n")).unwrap_or_default()
      )
    }
    ASTKind::StructModule(node) => {
      let header = match opts {
        CodegenOpts::ModuleHeader(val) => Some(val),
        _ => None,
      }
      .map(|val| {
        let sep = "\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

      let imports = codegen_imports(&node.imports)
        .map(|i| {
          let sep = "\n\n";
          concat_string!(i, sep)
        })
        .unwrap_or_default();

      let content = node
        .content
        .as_ref()
        .map(|d| codegen(d, CodegenOpts::None))
        .unwrap_or_default();

      concat_string!(header, imports, content)
    }
    ASTKind::Parameter(node) => {
      format!(
        "{}: {}",
        codegen(&node.name, CodegenOpts::None),
        codegen(&node.r#type, CodegenOpts::None)
      )
    }
    ASTKind::Method(node) => {
      let endpoint_name = match opts {
        CodegenOpts::MethodEndpointName(name) => name,
        _ => panic!("No value for CodegenOpts::MethodEndpointName"),
      };

      let name = codegen(&node.name, CodegenOpts::None);
      let parameters: Option<Vec<String>> = node.parameters.as_ref().map(|parameters| {
        parameters
          .iter()
          .map(|p| codegen(p, CodegenOpts::None))
          .collect()
      });

      let parameter_names: Option<Vec<String>> = node.parameters.as_ref().map(|parameters| {
        parameters
          .iter()
          .map(|p| codegen(&p.name, CodegenOpts::None))
          .collect()
      });

      format!(
        "function _{}({}): Promise<{}> {{\n  client.call(\"{}\", \"{}\"{});\n}}",
        name,
        parameters
          .as_ref()
          .map(|list| list.join(", "))
          .unwrap_or_default(),
        codegen(&node.return_type, CodegenOpts::None),
        codegen(endpoint_name, CodegenOpts::None),
        name,
        parameter_names
          .as_ref()
          .map(|list| format!(", {{{}}}", list.join(", ")))
          .unwrap_or_default()
      )
    }
    ASTKind::EndpointModule(node) => {
      let header = match opts {
        CodegenOpts::ModuleHeader(val) => Some(val),
        _ => None,
      }
      .map(|val| {
        let sep = "\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

      let imports = codegen_imports(&node.imports)
        .map(|val| {
          let sep = "\n\n";
          concat_string!(val, sep)
        })
        .unwrap_or_default();

      let items = node
        .content
        .as_ref()
        .map(|items| {
          let combined: String = items
            .iter()
            .map(|i| codegen(i, CodegenOpts::MethodEndpointName(&node.name)))
            .collect::<Vec<String>>()
            .join("\n\n");

          let sep = "\n\n";

          concat_string!(combined, sep)
        })
        .unwrap_or_default();

      let exports = node
        .content
        .as_ref()
        .map(|items| {
          let combined = items
            .iter()
            .map(|i| {
              let name = codegen(&i.name, CodegenOpts::None);
              format!("  _{n} as {n},", n = name)
            })
            .collect::<Vec<String>>()
            .join("\n");

          format!("export {{\n{}\n}};", combined)
        })
        .unwrap_or_default();

      concat_string!(header, imports, items, exports)
    }
    ASTKind::EnumVariant(node) => {
      let name = codegen(&node.0, CodegenOpts::None);
      format!("{n} = '{n}'", n = name)
    }
    ASTKind::Enum(node) => {
      let variants = node
        .variants
        .as_ref()
        .map(|items| {
          items
            .iter()
            .map(|val| format!("  {},", codegen(val, CodegenOpts::None)))
            .collect::<Vec<String>>()
            .join("\n")
        })
        .unwrap_or_default();

      format!(
        "export default enum {} {{\n{}\n}}",
        codegen(&node.name, CodegenOpts::None),
        variants,
      )
    }
    ASTKind::EnumModule(node) => {
      let header = match opts {
        CodegenOpts::ModuleHeader(val) => Some(val),
        _ => None,
      }
      .map(|val| {
        let sep = "\n";
        concat_string!(val, sep)
      })
      .unwrap_or_default();

      let content = node
        .content
        .as_ref()
        .map(|val| codegen(val, CodegenOpts::None))
        .unwrap_or_default();

      let sep = "\n";
      concat_string!(header, sep, content)
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::ts_a::{
    codegen, CodegenOpts, EndpointModule, Enum, EnumModule, EnumVariant, Field, Ident, Import,
    Method, Parameter, Struct, StructModule, Type,
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

    let code = codegen(
      &struct_mod,
      CodegenOpts::ModuleHeader("/**\n * Some header\n */"),
    );
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

    let code = codegen(
      &endpoint_mod,
      CodegenOpts::ModuleHeader("/**\n * Some header\n */"),
    );
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

    let code = codegen(
      &enum_mod,
      CodegenOpts::ModuleHeader("/**\n * Some header\n */"),
    );

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
