use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod send_ft;
mod send_near;
mod send_nft;
mod view_ft_balance;
mod view_near_balance;
mod view_nft_assets;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct TokensCommands {
    ///What is your account ID?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    tokens_actions: TokensActions,
}

impl TokensCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.tokens_actions
            .process(config, self.owner_account_id.clone().into())
            .await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select actions with tokens
pub enum TokensActions {
    #[strum_discriminants(strum(
        message = "send-near         - The transfer is carried out in NEAR tokens"
    ))]
    ///The transfer is carried out in NEAR tokens
    SendNear(self::send_near::SendNearCommand),
    #[strum_discriminants(strum(
        message = "send-ft           - The transfer is carried out in FT tokens"
    ))]
    ///The transfer is carried out in FT tokens
    SendFt(self::send_ft::SendFtCommand),
    #[strum_discriminants(strum(
        message = "send-nft          - The transfer is carried out in NFT tokens"
    ))]
    ///The transfer is carried out in NFT tokens
    SendNft(self::send_nft::SendNftCommand),
    #[strum_discriminants(strum(message = "view-near-balance - View the balance of Near tokens"))]
    ///View the balance of Near tokens
    ViewNearBalance(self::view_near_balance::ViewNearBalance),
    #[strum_discriminants(strum(message = "view-ft-balance   - View the balance of FT tokens"))]
    ///View the balance of FT tokens
    ViewFtBalance(self::view_ft_balance::ViewFtBalance),
    #[strum_discriminants(strum(message = "view-nft-assets   - View the balance of NFT tokens"))]
    ///View the balance of NFT tokens
    ViewNftAssets(self::view_nft_assets::ViewNftAssets),
}

impl TokensActions {
    async fn process(
        &self,
        config: crate::config::Config,
        owner_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        match self {
            Self::SendNear(send_near_command) => {
                send_near_command.process(config, owner_account_id).await
            }
            Self::ViewNearBalance(view_near_balance) => {
                view_near_balance.process(config, owner_account_id).await
            }
            Self::SendFt(send_ft_command) => {
                send_ft_command.process(config, owner_account_id).await
            }
            Self::SendNft(send_nft_command) => {
                send_nft_command.process(config, owner_account_id).await
            }
            Self::ViewFtBalance(view_ft_balance) => {
                view_ft_balance.process(config, owner_account_id).await
            }
            Self::ViewNftAssets(view_nft_assets) => {
                view_nft_assets.process(config, owner_account_id).await
            }
        }
    }
}
