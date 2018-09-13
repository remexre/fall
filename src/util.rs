//! Miscellaneous utilities.

use std::sync::atomic::{AtomicUsize, Ordering};

use futures::Stream;

/// Boxes a stream, since `.boxed()` is apparently deprecated. (It makes sense to deprecate the
/// futures version, since `Either` exists, but since there's no Stream `Either`...) This is mainly
/// a way to give a hint to type inference that we want a trait object.
pub fn box_stream<E, S, T>(stream: S) -> Box<Stream<Item = T, Error = E> + Send>
where
    S: 'static + Stream<Item = T, Error = E> + Send,
{
    Box::new(stream)
}

/// Generates a new number.
pub fn gensym() -> usize {
    lazy_static! {
        static ref N: AtomicUsize = AtomicUsize::new(0);
    }
    N.fetch_add(1, Ordering::SeqCst)
}
