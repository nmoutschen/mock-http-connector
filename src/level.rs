/// Diagnostic levels
pub enum Level {
    /// Display information for all cases
    Debug,
    /// Display information when no cases match the incoming request
    Missing,
    /// Never display diagnostic information
    None,
}
