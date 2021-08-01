use core::slice::Iter;
use std::iter::Map;

pub(crate) trait ResultIterator: Iterator {
  fn flat_map_res<T, R, E, F>(self, f: F) -> FlatMapRes<Self, F>
  where
    Self: Iterator<Item = Result<T, E>> + Sized,
    F: FnMut(T) -> Result<R, E>,
  {
    FlatMapRes { iter: self, f }
  }
}

impl<T: ?Sized> ResultIterator for T where T: Iterator {}

pub struct FlatMapRes<I, F> {
  iter: I,
  f: F,
}

impl<T, R, E, I: Iterator<Item = Result<T, E>>, F> Iterator for FlatMapRes<I, F>
where
  F: FnMut(T) -> Result<R, E>,
{
  type Item = Result<R, E>;

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    self.iter.next().map(|res| res.and_then(&mut self.f))
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    self.iter.size_hint()
  }

  fn fold<Acc, G>(mut self, init: Acc, mut g: G) -> Acc
  where
    G: FnMut(Acc, Self::Item) -> Acc,
  {
    fn map_fold<T, R, E, Acc>(
      mut f: impl FnMut(T) -> Result<R, E>,
      mut g: impl FnMut(Acc, Result<R, E>) -> Acc,
    ) -> impl FnMut(Acc, Result<T, E>) -> Acc {
      move |acc, elt| g(acc, elt.and_then(|v| f(v)))
    }

    self.iter.fold(init, map_fold(self.f, g))
  }
}
