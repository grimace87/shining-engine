
/// PresentResult enumeration
/// Possible outcomes of a presentation action.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum PresentResult {
    Ok,
    SwapchainOutOfDate
}
