pub use crate::ast::common::*;
pub use crate::ast::endpoint::*;
use crate::ast::{Ident, Import};
use crate::ts::TSConvertable;
use std::rc::Rc;

mod common;
mod declaration;
mod endpoint;

#[derive(Clone)]
pub struct File<T> {
  content: Option<T>,
  imports: Option<Rc<Vec<Import>>>,
  name: Ident,
}

impl<T> File<T> {
  pub(crate) fn with_options(&self, header: Option<&str>) -> FileWithOptions<T> {
    FileWithOptions {
      file: self,
      header: header.map(|h| h.to_string()),
    }
  }
}

impl<T> AST for File<T> {}

pub(crate) struct FileWithOptions<'a, T> {
  file: &'a File<T>,
  header: Option<String>,
}

impl<T> FileWithOptions<'_, T> {
  #[inline]
  pub(crate) fn imports_as_ts(&self) -> Option<String> {
    let imports: Option<Vec<String>> = self
      .file
      .imports
      .as_ref()
      .map(|list| list.iter().map(|i| i.as_ts()).collect());

    imports.map(|list| list.join("\n"))
  }
}

pub trait AST {}
