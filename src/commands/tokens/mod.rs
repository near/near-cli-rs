use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod send_ft;
mod send_near;
mod send_nft;
mod view_ft_balance;
mod view_near_balance;
mod view_nft_assets;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TokensCommandsContext)]
pub struct TokensCommands {
    #[interactive_clap(skip_default_input_arg)]
    /// What is your account ID?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    tokens_actions: TokensActions,
}

#[derive(Debug, Clone)]
pub struct TokensCommandsContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_primitives::types::AccountId,
}

impl TokensCommandsContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TokensCommands as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            owner_account_id: scope.owner_account_id.clone().into(),
        })
    }
}

impl TokensCommands {
    pub fn input_owner_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is your account ID?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = TokensCommandsContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Select actions with tokens:
pub enum TokensActions {
    #[strum_discriminants(strum(
        message = "send-near         - The transfer is carried out in NEAR tokens"
    ))]
    /// The transfer is carried out in NEAR tokens
    SendNear(self::send_near::SendNearCommand),
    #[strum_discriminants(strum(
        message = "send-ft           - The transfer is carried out in FT tokens"
    ))]
    /// The transfer is carried out in FT tokens
    SendFt(self::send_ft::SendFtCommand),
    #[strum_discriminants(strum(
        message = "send-nft          - The transfer is carried out in NFT tokens"
    ))]
    /// The transfer is carried out in NFT tokens
    SendNft(self::send_nft::SendNftCommand),
    #[strum_discriminants(strum(message = "view-near-balance - View the balance of Near tokens"))]
    /// View the balance of Near tokens
    ViewNearBalance(self::view_near_balance::ViewNearBalance),
    #[strum_discriminants(strum(message = "view-ft-balance   - View the balance of FT tokens"))]
    /// View the balance of FT tokens
    ViewFtBalance(self::view_ft_balance::ViewFtBalance),
    #[strum_discriminants(strum(message = "view-nft-assets   - View the balance of NFT tokens"))]
    /// View the balance of NFT tokens
    ViewNftAssets(self::view_nft_assets::ViewNftAssets),
}
