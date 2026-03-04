use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    instructions::TransferV1CpiBuilder,
    types::UpdateAuthority,
    ID as MPL_CORE_ID,
};

use crate::{errors::StakingError, state::Oracle};

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    /// CHECK: NFT asset - validated by mpl_core
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: Collection - validated by mpl_core
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: New owner of the NFT
    pub receiver: UncheckedAccount<'info>,

    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: MPL Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Transfer<'info> {
    pub fn transfer_nft(&self) -> Result<()> {
        // Verify NFT owner and collection
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(
            base_asset.owner == self.owner.key(),
            StakingError::InvalidOwner
        );
        require!(
            base_asset.update_authority == UpdateAuthority::Collection(self.collection.key()),
            StakingError::InvalidAuthority
        );
        let _base_collection = BaseCollectionV1::try_from(&self.collection.to_account_info())?;

        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .new_owner(&self.receiver.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .add_remaining_account(
                &self.oracle.to_account_info(),
                true, // writable
                true, // signer
            )
            .invoke()?;

        Ok(())
    }
}
