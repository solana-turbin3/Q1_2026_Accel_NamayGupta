use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{
        Attribute, Attributes, ExternalCheckResult, ExternalPluginAdapterInitInfo,
        HookableLifecycleEvent, OracleInitInfo, Plugin, PluginAuthority, PluginAuthorityPair,
        ValidationResultsOffset,
    },
    ID as MPL_CORE_ID,
};

use crate::state::Oracle;
#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub collection: Signer<'info>,
    /// CHECK: PDA Update authority
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,
    pub oracle_account: Account<'info, Oracle>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}
impl<'info> CreateCollection<'info> {
    pub fn create_collection(
        &mut self,
        name: String,
        uri: String,
        bumps: &CreateCollectionBumps,
    ) -> Result<()> {
        // Signer seeds for the update authority
        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];
        let mut external_plugin_adapters: Vec<ExternalPluginAdapterInitInfo> = vec![];

        external_plugin_adapters.push(ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
            base_address: self.oracle_account.key(), //????
            results_offset: Some(ValidationResultsOffset::Anchor),
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                ExternalCheckResult { flags: 4 }, ////????
            )],
            base_address_config: None,
            init_plugin_authority: None,
        }));
        // Create the collection with CPI builder with Attributes plugin
        CreateCollectionV2CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.payer.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .plugins(vec![PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![Attribute {
                        key: "total_staked".to_string(),
                        value: "0".to_string(),
                    }],
                }),
                authority: Some(PluginAuthority::UpdateAuthority),
            }])
            .external_plugin_adapters(external_plugin_adapters)
            .invoke_signed(&[signer_seeds])?;

        // //Add the oracle plugin
        // AddCollectionExternalPluginAdapterV1CpiBuilder::new(
        //     &self.mpl_core_program.to_account_info(),
        // )
        // .collection(&self.collection.to_account_info())
        // .payer(&self.payer.to_account_info())
        // .init_info(ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
        //     base_address: self.oracle_account.key(),//????
        //     results_offset: Some(ValidationResultsOffset::Anchor),
        //     lifecycle_checks: vec![(
        //         HookableLifecycleEvent::Transfer,
        //         ExternalCheckResult { flags: 4 },////????
        //     )],
        //     base_address_config: None,
        //     init_plugin_authority: None,
        // }))
        // .invoke_signed(&[signer_seeds])?;
        Ok(())
    }
}
