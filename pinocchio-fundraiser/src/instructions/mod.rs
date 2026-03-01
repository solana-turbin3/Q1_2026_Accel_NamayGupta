use pinocchio::error::ProgramError;

pub mod initialize;
pub use initialize::*;
pub mod contribute;
pub use contribute::*;
pub mod check;
pub use check::*;
pub mod refund;
pub use refund::*;

pub enum FundraiserInstructions {
    Initialize = 0,
    Contribute = 1,
    Check = 2,
    Refund = 3,
}

impl TryFrom<&u8> for FundraiserInstructions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(FundraiserInstructions::Initialize),
            1 => Ok(FundraiserInstructions::Contribute),
            2 => Ok(FundraiserInstructions::Check),
            3 => Ok(FundraiserInstructions::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
