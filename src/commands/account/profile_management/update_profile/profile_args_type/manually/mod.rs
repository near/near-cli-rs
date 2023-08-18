use std::collections::HashMap;

use inquire::{CustomType, Select, Text};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::UpdateAccountProfileContext)]
#[interactive_clap(output_context = ManuallyContext)]
#[interactive_clap(skip_default_from_cli)]
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
        previous_context: super::super::UpdateAccountProfileContext,
        scope: &<Manually as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let profile = crate::types::socialdb_types::Profile {
            name: scope.name.clone(),
            image: if scope.image_url.is_none() && scope.image_ipfs_cid.is_none() {
                None
            } else {
                Some(crate::types::socialdb_types::ProfileImage {
                    url: scope.image_url.clone().map(|url| url.into()),
                    ipfs_cid: scope.image_ipfs_cid.clone(),
                })
            },
            background_image: if scope.background_image_url.is_none()
                && scope.background_image_ipfs_cid.is_none()
            {
                None
            } else {
                Some(crate::types::socialdb_types::ProfileImage {
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
                        Some(scope.website.clone().expect("Unexpected error").to_string()),
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
            get_contract_account_id: previous_context.get_contract_account_id,
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

impl interactive_clap::FromCli for Manually {
    type FromCliContext = super::super::UpdateAccountProfileContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();
        if clap_variant.name.is_none() {
            clap_variant.name = match Self::input_name(&context) {
                Ok(optional_name) => optional_name,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let name = clap_variant.name.clone();
        if clap_variant.image_url.is_none() {
            clap_variant.image_url = match Self::input_image_url(&context) {
                Ok(optional_image_url) => optional_image_url,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let image_url = clap_variant.image_url.clone();
        if clap_variant.image_ipfs_cid.is_none() {
            clap_variant.image_ipfs_cid = match Self::input_image_ipfs_cid(&context) {
                Ok(optional_image_ipfs_cid) => optional_image_ipfs_cid,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let image_ipfs_cid = clap_variant.image_ipfs_cid.clone();
        if clap_variant.background_image_url.is_none() {
            clap_variant.background_image_url = match Self::input_background_image_url(&context) {
                Ok(optional_background_image_url) => optional_background_image_url,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let background_image_url = clap_variant.background_image_url.clone();
        if clap_variant.background_image_ipfs_cid.is_none() {
            clap_variant.background_image_ipfs_cid =
                match Self::input_background_image_ipfs_cid(&context) {
                    Ok(optional_background_image_ipfs_cid) => optional_background_image_ipfs_cid,
                    Err(err) => {
                        return interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                    }
                };
        };
        let background_image_ipfs_cid = clap_variant.background_image_ipfs_cid.clone();
        if clap_variant.description.is_none() {
            clap_variant.description = match Self::input_description(&context) {
                Ok(optional_description) => optional_description,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let description = clap_variant.description.clone();
        if clap_variant.twitter.is_none() {
            clap_variant.twitter = match Self::input_twitter(&context) {
                Ok(optional_twitter) => optional_twitter,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let twitter = clap_variant.twitter.clone();
        if clap_variant.github.is_none() {
            clap_variant.github = match Self::input_github(&context) {
                Ok(optional_github) => optional_github,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let github = clap_variant.github.clone();
        if clap_variant.telegram.is_none() {
            clap_variant.telegram = match Self::input_telegram(&context) {
                Ok(optional_telegram) => optional_telegram,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let telegram = clap_variant.telegram.clone();
        if clap_variant.website.is_none() {
            clap_variant.website = match Self::input_website(&context) {
                Ok(optional_website) => optional_website,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let website = clap_variant.website.clone();
        if clap_variant.tags.is_none() {
            clap_variant.tags = match Self::input_tags(&context) {
                Ok(optional_tags) => optional_tags,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        };
        let tags = clap_variant.tags.clone();
        let new_context_scope = InteractiveClapContextScopeForManually {
            name,
            image_url,
            image_ipfs_cid,
            background_image_url,
            background_image_ipfs_cid,
            description,
            twitter,
            github,
            telegram,
            website,
            tags,
        };
        let output_context =
            match ManuallyContext::from_previous_context(context, &new_context_scope) {
                Ok(new_context) => new_context,
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        let context = output_context;
        let optional_field = clap_variant
            .sign_as
            .take()
            .map(|ClapNamedArgSignerForManually::SignAs(cli_arg)| cli_arg);
        match <super::super::sign_as::Signer as interactive_clap::FromCli>::from_cli(
            optional_field,
            context.into(),
        ) {
            interactive_clap::ResultFromCli::Ok(cli_field) => {
                clap_variant.sign_as = Some(ClapNamedArgSignerForManually::SignAs(cli_field));
            }
            interactive_clap::ResultFromCli::Cancel(optional_cli_field) => {
                clap_variant.sign_as =
                    optional_cli_field.map(ClapNamedArgSignerForManually::SignAs);
                return interactive_clap::ResultFromCli::Cancel(Some(clap_variant));
            }
            interactive_clap::ResultFromCli::Back => return interactive_clap::ResultFromCli::Back,
            interactive_clap::ResultFromCli::Err(optional_cli_field, err) => {
                clap_variant.sign_as =
                    optional_cli_field.map(ClapNamedArgSignerForManually::SignAs);
                return interactive_clap::ResultFromCli::Err(Some(clap_variant), err);
            }
        };
        interactive_clap::ResultFromCli::Ok(clap_variant)
    }
}
impl Manually {
    fn input_name(
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
        _context: &super::super::UpdateAccountProfileContext,
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
