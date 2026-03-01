use pinocchio::{AccountView, error::ProgramError};
#[repr(C)]
pub struct Contributor {
    pub amount: [u8; 8],
    pub bump: u8,
}

impl Contributor {
    pub const LEN: usize = 9;

    pub fn from_account_info(account_info: &AccountView) -> Result<&mut Self, ProgramError> {
        let mut data = account_info.try_borrow_mut()?;
        if data.len() != Contributor::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}
