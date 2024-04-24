use thiserror::Error;

#[derive(Error, Debug)]
pub enum SplitterError {
    #[error("Invalid m parameter. m={0} and expected-accounts-len={1} But provided-accounts-len={2}")]
    InvalidMParameter(u16, usize, usize)
}
impl Into<u32> for SplitterError {
    fn into(self) -> u32 {
        match self {
            Self::InvalidMParameter(_, _ , _) => 0u32
        }
    }
}