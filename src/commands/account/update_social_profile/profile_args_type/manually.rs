use std::collections::HashMap;

use inquire::{CustomType, Select, Text};

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
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter a name for the account profile")]
            Yes,
            #[strum(to_string = "No, I don't want to enter a name for the account profile")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter a name for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter a name for the account profile:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_image_url(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter the URL for the account profile image")]
            Yes,
            #[strum(to_string = "No, I don't want to enter the URL of the account profile image")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter an account profile image URL?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let url: crate::types::url::Url =
                CustomType::new("What is the account profile image URL?").prompt()?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    fn input_image_ipfs_cid(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter ipfs_cid for the account profile image")]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter ipfs_cid for the account profile image"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter ipfs_cid for the account profile image?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter ipfs_cid for the account's profile image:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_background_image_url(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to enter the URL for the account profile background image"
            )]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter the URL of the account profile background image"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter an account profile background image URL?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let url: crate::types::url::Url =
                CustomType::new("What is the account profile background image URL?").prompt()?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    fn input_background_image_ipfs_cid(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to enter ipfs_cid for the account profile background image"
            )]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter ipfs_cid for the account profile background image"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter ipfs_cid for the account profile background image?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter ipfs_cid for the account profile background image:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_description(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter a description for the account profile")]
            Yes,
            #[strum(to_string = "No, I don't want to enter a description for the account profile")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter a description for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter a description for the account profile:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_twitter(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter a Twitter nickname for the account profile")]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter a Twitter nickname for the account profile"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter a Twitter nickname for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter a Twitter nickname for the account profile:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_github(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter a Github nickname for the account profile")]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter a Github nickname for the account profile"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter a Github nickname for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter a Github nickname for the account profile:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_telegram(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to enter a Telegram nickname for the account profile"
            )]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter a Telegram nickname for the account profile"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter a Telegram nickname for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            Ok(Some(
                Text::new("Enter a Telegram nickname for the account profile:").prompt()?,
            ))
        } else {
            Ok(None)
        }
    }

    fn input_website(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter the website URL for the account profile")]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter the website URL for the account profile"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter the website URL for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let url: crate::types::url::Url =
                CustomType::new("Enter the website URL for the account profile:").prompt()?;
            Ok(Some(url))
        } else {
            Ok(None)
        }
    }

    fn input_tags(
        _context: &super::super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter tags for an account profile")]
            Yes,
            #[strum(to_string = "No, I don't want to enter tags for an account profile")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter tags for the account profile?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let tags: crate::types::vec_string::VecString =
                CustomType::new("Enter a comma-separated list of tags for account profile:")
                    .prompt()?;
            Ok(Some(tags))
        } else {
            Ok(None)
        }
    }
}
