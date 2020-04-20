use super::Karx;

/// Returns a handle to the current Karx.
///
/// # Panics
///
/// This function will panic if not called within the context of a Karx created by [`block_on`],
/// [`spawn`], or [`Builder::spawn`].
///
/// [`block_on`]: fn.block_on.html
/// [`spawn`]: fn.spawn.html
/// [`Builder::spawn`]: struct.Builder.html#method.spawn
///
/// # Examples
///
/// ```
/// # kayrx::karx::block_on(async {
/// #
/// use kayrx::karx;
///
/// println!("The name of this karx is {:?}", karx::current().name());
/// #
/// # })
/// ```
pub fn current() -> Karx {
    Karx::get_current(|t| t.clone())
        .expect("`karx::current()` called outside the context of a karx")
}
