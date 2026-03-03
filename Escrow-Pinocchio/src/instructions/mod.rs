pub mod cancel;
pub mod cancelv2;
pub mod make;
pub mod makev2;
pub mod take;
pub mod takev2;

pub use cancel::*;
pub use cancelv2::*;
pub use make::*;
pub use makev2::*;
use pinocchio::error::ProgramError;
pub use take::*;
pub use takev2::*;

// Re-export wincode data structs for V2 tests
pub(crate) use cancelv2::CancelV2Data;
pub(crate) use makev2::MakeV2Data;
pub(crate) use takev2::TakeV2Data;

pub enum EscrowInstrctions {
    Make = 0,
    Take = 1,
    Cancel = 2,
    MakeV2 = 3,
    TakeV2 = 4,
    CancelV2 = 5,
}

impl TryFrom<&u8> for EscrowInstrctions {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(EscrowInstrctions::Make),
            1 => Ok(EscrowInstrctions::Take),
            2 => Ok(EscrowInstrctions::Cancel),
            3 => Ok(EscrowInstrctions::MakeV2),
            4 => Ok(EscrowInstrctions::TakeV2),
            5 => Ok(EscrowInstrctions::CancelV2),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
