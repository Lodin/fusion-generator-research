use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::rc::Rc;

use coffer::{Class, ReadWrite};
use concat_string::concat_string;
use itertools::Itertools;
use walkdir::WalkDir;
use zip::read::ZipFile;
use zip::result::ZipError;
use zip::ZipArchive;

use crate::error::ResolutionError;
use crate::utils::ResultIterator;

pub struct Resolver {
  paths: Rc<Vec<String>>,
}

impl Resolver {
  fn resolve(&self, dep: &str) -> Result<Class, ResolutionError> {
    let dep = {
      let tail = ".class";
      concat_string!(dep, tail)
    };

    self
      .paths
      .iter()
      .filter(|path| !path.ends_with(".jar"))
      .flat_map(|path| WalkDir::new(path).into_iter())
      .filter_map(|entry| entry.ok())
      .filter_map(|entry| entry.path().to_str().map(|s| s.to_owned()))
      .filter(|path| path.contains(&dep))
      .map(|path| File::open(path).map_err(ResolutionError::from))
      .flat_map_res(|mut file| Ok(Class::read_from(&mut file)?))
      .next()
      .or(
        self
          .paths
          .iter()
          .filter(|path| path.ends_with(".jar"))
          .map(|path| File::open(path).map_err(ResolutionError::from))
          .map_ok(BufReader::new)
          .flat_map_res(|mut reader| Ok(ZipArchive::new(reader)?))
          .flat_map_res(|mut archive| {
            Ok(match archive.by_name(&dep).ok() {
              Some(mut file) => Some(Class::read_from(&mut file)?),
              None => None,
            })
          })
          .filter_map_ok(|class| class)
          .next(),
      )
      .ok_or(ResolutionError::DependencyNotResolved(dep.to_string()))?
  }
}
