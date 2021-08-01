use std::io;

use thiserror::Error;
use zip::result::ZipError;

use crate::translator::TranslationError;

#[derive(Error, Debug)]
pub enum GeneratorError {
  #[error(transparent)]
  ResolutionError(#[from] ResolutionError),
  #[error(transparent)]
  TranslationError(#[from] TranslationError),
}

#[derive(Error, Debug)]
pub enum ResolutionError {
  #[error("jar file not found")]
  JarNotFound(String),
  #[error("dependency not resolved")]
  DependencyNotResolved(String),
  #[error(transparent)]
  IO(#[from] io::Error),
  #[error(transparent)]
  Zip(#[from] ZipError),
  #[error(transparent)]
  Coffer(#[from] coffer::Error),
}
