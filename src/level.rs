/// Diagnostic levels
///
/// ## Default
///
/// [`Level`] implements [`Default`], which will return `Level::Missing`.
///
/// ```rust
/// use mock_http_connector::Level;
///
/// assert_eq!(Level::default(), Level::Missing);
/// ```
///
/// ## Ordering
///
/// [`Level`] implements [`PartialOrd`] and [`Ord`] to facilitates comparing two [`Level`]s to
/// know which one is more verbose. The one with a greater value will be more verbose than the
/// other.
///
/// ```rust
/// use mock_http_connector::Level;
///
/// assert!(Level::Error < Level::Missing);
/// assert!(Level::None < Level::Error);
/// assert_eq!(Level::Missing, Level::Missing);
/// ```
///
/// ## Internal value guarantees
///
/// This enum implements [`PartialOrd`] and [`Ord`] through an internal representation as `u8`.
/// However, there is no guarantees that the actual integer values for each enum variants will
/// stay consistent from version to version.
///
/// If you want to compare a [`Level`] to a certain value, you should always compare it to a
/// [`Level`] and not to an integer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Level {
    /// Display information when no cases match the incoming request
    #[default]
    Missing = 2,
    /// Display diagnostic information on errors
    Error = 1,
    /// Never display diagnostic information
    None = 0,
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Level {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}
