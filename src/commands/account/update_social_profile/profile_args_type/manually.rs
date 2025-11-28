use std::collections::HashMap;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateSocialProfileContext)]
#[interactive_clap(output_context = ManuallyContext)]
pub struct Manually {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    name: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    image_url: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    image_ipfs_cid: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    background_image_url: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    background_image_ipfs_cid: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    description: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    twitter: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    github: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    telegram: Option<String>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    website: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    tags: Option<crate::types::vec_string::VecString>,
    #[interactive_clap(named_arg)]
    /// Specify signer account ID
    sign_as: super::super::sign_as::Signer,
}

#[derive(Clone)]
pub struct ManuallyContext(super::ArgsContext);

impl ManuallyContext {
    pub fn from_previous_context(
        previous_context: super::super::UpdateSocialProfileContext,
        scope: &<Manually as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let profile = near_socialdb_client::types::socialdb_types::Profile {
            name: scope.name.clone(),
            image: if scope.image_url.is_none() && scope.image_ipfs_cid.is_none() {
                None
            } else {
                Some(near_socialdb_client::types::socialdb_types::ProfileImage {
                    url: scope.image_url.clone().map(|url| url.into()),
                    ipfs_cid: scope.image_ipfs_cid.clone(),
                })
            },
            background_image: if scope.background_image_url.is_none()
                && scope.background_image_ipfs_cid.is_none()
            {
                None
            } else {
                Some(near_socialdb_client::types::socialdb_types::ProfileImage {
                    url: scope.background_image_url.clone().map(|url| url.into()),
                    ipfs_cid: scope.background_image_ipfs_cid.clone(),
                })
            },
            description: scope.description.clone(),
            linktree: if scope.twitter.is_none()
                && scope.github.is_none()
                && scope.telegram.is_none()
                && scope.website.is_none()
            {
                None
            } else {
                let mut linktree_map: HashMap<String, Option<String>> = HashMap::new();
                if scope.twitter.is_some() {
                    linktree_map.insert("twitter".to_string(), scope.twitter.clone());
                }
                if scope.telegram.is_some() {
                    linktree_map.insert("telegram".to_string(), scope.telegram.clone());
                }
                if scope.github.is_some() {
                    linktree_map.insert("github".to_string(), scope.github.clone());
                }
                if scope.website.is_some() {
                    linktree_map.insert(
                        "website".to_string(),
                        scope.website.as_ref().map(|website| website.to_string()),
                    );
                }
                Some(linktree_map)
            },
            tags: if let Some(tags) = scope.tags.clone() {
                let mut tags_map: HashMap<String, String> = HashMap::new();
                for tag in tags.0.iter() {
                    tags_map.insert(tag.clone(), "".to_string());
                }
                Some(tags_map)
            } else {
                None
            },
        };
        Ok(Self(super::ArgsContext {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            data: serde_json::to_vec(&profile)?,
        }))
    }
}

impl From<ManuallyContext> for super::ArgsContext {
    fn from(item: ManuallyContext) -> Self {
        item.0
    }
}

impl Manually {
    fn input_name(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter a name for the account profile";
        let confirm_no = "No, I don't want to enter a name for the account profile";
        if cliclack::select("Do you want to enter a name for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            Ok(Some(
                cliclack::input("Enter a name for the account profile:").interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_image_url(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the URL for the account profile image";
        let confirm_no = "No, I don't want to enter the URL of the account profile image";
        if cliclack::select("Do you want to enter an account profile image URL?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let url: crate::types::url::Url =
                cliclack::input("What is the account profile image URL?").interact()?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    fn input_image_ipfs_cid(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter ipfs_cid for the account profile image";
        let confirm_no = "No, I don't want to enter ipfs_cid for the account profile image";
        if cliclack::select("Do you want to enter ipfs_cid for the account profile image?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            Ok(Some(
                cliclack::input("Enter ipfs_cid for the account's profile image:").interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_background_image_url(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the URL for the account profile background image";
        let confirm_no =
            "No, I don't want to enter the URL of the account profile background image";
        if cliclack::select("Do you want to enter an account profile background image URL?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let url: crate::types::url::Url =
                cliclack::input("What is the account profile background image URL?").interact()?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    fn input_background_image_ipfs_cid(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter ipfs_cid for the account profile background image";
        let confirm_no =
            "No, I don't want to enter ipfs_cid for the account profile background image";
        if cliclack::select(
            "Do you want to enter ipfs_cid for the account profile background image?",
        )
        .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
        .interact()?
        {
            Ok(Some(
                cliclack::input("Enter ipfs_cid for the account profile background image:")
                    .interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_description(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter a description for the account profile";
        let confirm_no = "No, I don't want to enter a description for the account profile";
        if cliclack::select("Do you want to enter a description for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            Ok(Some(
                cliclack::input("Enter a description for the account profile:").interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_twitter(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter a Twitter nickname for the account profile";
        let confirm_no = "No, I don't want to enter a Twitter nickname for the account profile";
        if cliclack::select("Do you want to enter a Twitter nickname for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            Ok(Some(
                cliclack::input("Enter a Twitter nickname for the account profile:").interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_github(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter a Github nickname for the account profile";
        let confirm_no = "No, I don't want to enter a Github nickname for the account profile";
        if cliclack::select("Do you want to enter a Github nickname for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            Ok(Some(
                cliclack::input("Enter a Github nickname for the account profile:").interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_telegram(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let confirm_yes = "Yes, I want to enter a Telegram nickname for the account profile";
        let confirm_no = "No, I don't want to enter a Telegram nickname for the account profile";
        if cliclack::select("Do you want to enter a Telegram nickname for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            Ok(Some(
                cliclack::input("Enter a Telegram nickname for the account profile:").interact()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_website(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the website URL for the account profile";
        let confirm_no = "No, I don't want to enter the website URL for the account profile";
        if cliclack::select("Do you want to enter the website URL for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let url: crate::types::url::Url =
                cliclack::input("Enter the website URL for the account profile:").interact()?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    fn input_tags(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        let confirm_yes = "Yes, I want to enter tags for an account profile";
        let confirm_no = "No, I don't want to enter tags for an account profile";
        if cliclack::select("Do you want to enter tags for the account profile?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let tags: crate::types::vec_string::VecString =
                cliclack::input("Enter a comma-separated list of tags for account profile:")
                    .interact()?;
            Ok(Some(tags))
        } else {
            Ok(None)
        }
    }
}
