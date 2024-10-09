//! Provides an alternative of the `embedded_io::ErrorType` trait.

/// A base trait identical in spirit to `embedded_io::ErrorType` except
/// also implemented for `&T where T: ErrorType`.
///
/// Useful, since all traits in this crate take `&self` and not `&mut self`,
/// so for these it makes sense to also be implemented for `&T`.
/// However, that's not possible if their base `ErrorType` is also not implemented for `&T`.
pub trait ErrorType {
    /// Error type of all the IO operations on this type.
    type Error: embedded_io_async::Error;
}

impl<C> ErrorType for &C
where
    C: ErrorType,
{
    type Error = C::Error;
}

impl<C> ErrorType for &mut C
where
    C: ErrorType,
{
    type Error = C::Error;
}
