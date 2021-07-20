use std::rc::Rc;

use concat_string::concat_string;

use crate::ast::{File, FileWithOptions, Ident, Import, Type, AST};
use crate::ts::TSConvertable;

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

impl AST for Parameter {}

impl TSConvertable for Parameter {
  fn as_ts(&self) -> String {
    format!("{}: {}", self.name.as_ts(), self.r#type.as_ts())
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

  pub(crate) fn with_options(&self, endpoint_name: Ident) -> MethodWithOptions {
    MethodWithOptions {
      endpoint_name,
      method: self,
    }
  }
}

impl AST for Method {}

pub(crate) struct MethodWithOptions<'a> {
  endpoint_name: Ident,
  method: &'a Method,
}

impl TSConvertable for MethodWithOptions<'_> {
  fn as_ts(&self) -> String {
    let name = self.method.name.as_ts();
    let parameters: Option<Vec<String>> = self
      .method
      .parameters
      .as_ref()
      .map(|parameters| parameters.iter().map(|p| p.as_ts()).collect());

    let parameter_names: Option<Vec<String>> = self
      .method
      .parameters
      .as_ref()
      .map(|parameters| parameters.iter().map(|p| p.name.as_ts()).collect());

    format!(
      "function _{}({}): Promise<{}> {{\n  client.call(\"{}\", \"{}\"{});\n}}",
      name,
      parameters
        .as_ref()
        .map(|list| list.join(", "))
        .unwrap_or_default(),
      self.method.return_type.as_ts(),
      self.endpoint_name.as_ts(),
      name,
      parameter_names
        .as_ref()
        .map(|list| format!(", {{{}}}", list.join(", ")))
        .unwrap_or_default()
    )
  }
}

pub type EndpointFile = File<Rc<Vec<Method>>>;
type EndpointFileWithOptions<'a> = FileWithOptions<'a, Rc<Vec<Method>>>;

impl EndpointFile {
  pub fn new(name: Ident, imports: Option<Vec<Import>>, content: Option<Vec<Method>>) -> Self {
    Self {
      content: content.map(|val| Rc::new(val)),
      imports: imports.map(|val| Rc::new(val)),
      name,
    }
  }
}

impl TSConvertable for EndpointFileWithOptions<'_> {
  fn as_ts(&self) -> String {
    let items: Option<Vec<String>> = self.file.content.as_ref().map(|items| {
      items
        .iter()
        .map(|i| i.with_options(self.file.name.clone()).as_ts())
        .collect()
    });

    let exports: Option<Vec<String>> = self
      .file
      .content
      .as_ref()
      .map(|items| items.iter().map(|i| i.name.as_ts()).collect());

    concat_string!(
      self
        .header
        .as_ref()
        .map(|val| concat_string!(val, "\n"))
        .unwrap_or_default(),
      self
        .imports_as_ts()
        .map(|val| concat_string!(val, "\n\n"))
        .unwrap_or_default(),
      items
        .as_ref()
        .map(|i| concat_string!(i.join("\n\n"), "\n\n"))
        .unwrap_or_default(),
      exports
        .as_ref()
        .map(|e| {
          let exports: Vec<String> = e
            .iter()
            .map(|name| format!("  _{n} as {n},", n = name))
            .collect();

          format!("export {{\n{}\n}};", exports.join("\n"))
        })
        .unwrap_or_default()
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::{EndpointFile, Ident, Import, Method, Parameter, Type};
  use crate::ts::TSConvertable;

  #[test]
  fn should_generate_code() {
    let health_grid_item_struct_ident = Ident::new("HealthGridItem");
    let chart_series_struct_ident = Ident::new("ChartsSeries");
    let array_ident = Ident::new("Array");

    let ast = EndpointFile::new(
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

    let code = ast.with_options(Some("/**\n * Some header\n */")).as_ts();
    assert_eq!(
      code,
      "\
/**
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
}
