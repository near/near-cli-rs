#![allow(unused_variables)]
#![allow(dead_code)]

use crate::mock::*;
use crate::types::{self, *};

mod __eclap_gen {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    pub struct AClapVariant {
        pub network: Option<Network>,

        #[clap(subcommand)]
        pub subcommand: Option<BMiddleLayer>,
    }

    #[derive(Default)]
    pub struct ABuilder {
        pub network: Option<Network>,
    }

    impl ABuilder {
        pub fn set_network(mut self, network: Network) -> Self {
            self.network = Some(network);
            self
        }
    }

    impl types::IntoScope<AScope> for ABuilder {
        fn into_scope(&self) -> AScope {
            AScope {
                network: self.network.as_ref().unwrap().clone(),
            }
        }
    }

    #[derive(Clone)]
    pub struct AScope {
        pub network: Network,
    }

    #[derive(Parser)]
    pub enum BMiddleLayer {
        B(BClapVariant),
    }

    impl BMiddleLayer {
        pub fn unwrap(self) -> BClapVariant {
            match self {
                BMiddleLayer::B(x) => x,
            }
        }
    }

    #[derive(Parser)]
    pub struct BClapVariant {
        pub account_id: Option<AccountId>,
    }

    #[derive(Default)]
    pub struct BBuilder {
        pub account_id: Option<AccountId>,
    }

    impl types::IntoScope<BScope> for BBuilder {
        fn into_scope(&self) -> BScope {
            BScope {
                account_id: self.account_id.as_ref().unwrap().clone(),
            }
        }
    }

    #[derive(Clone)]
    pub struct BScope {
        pub account_id: AccountId,
    }

}

impl types::Builder for A {
    type Builder = __eclap_gen::ABuilder;

    // fn scoped(&self) -> Self::Scope {
    //     Self::Scope {
    //         network: self.network.clone(),
    //     }
    // }
}

impl types::Scoped for A {
    type Scope = __eclap_gen::AScope;
}

impl types::ClapVariant for A {
    type Clap = __eclap_gen::AClapVariant;
}

impl types::Eclap for A {}

impl types::Builder for B {
    type Builder = __eclap_gen::BBuilder;

    // fn scoped(&self) -> Self::Scope {
    //     Self::Scope {
    //         account_id: self.account_id.clone(),
    //     }
    // }
}

impl types::Scoped for B {
    type Scope = __eclap_gen::BScope;
}

impl types::ClapVariant for B {
    type Clap = __eclap_gen::BClapVariant;
}

impl types::Eclap for B {}

////////////////////////////////////////////////////////////////////////////////
/// GENERATED CODE
////////////////////////////////////////////////////////////////////////////////

impl types::Interactive for A {
    fn interactive(clap: Option<&Self::Clap>, builder: Self::Builder) -> Self::Builder {
        if let Some(clap) = clap {
            let builder = match clap.network.as_ref() {
                Some(network) => builder.set_network(network.clone()),
                None => builder //eclap::mapped_fn(),
            };

            return builder;
        }

        builder
    }
}

impl types::Build for A {
    // type Err = crate::errors::BuildError;
    type Err = ();

    fn build(clap: Option<Self::Clap>, mut builder: Self::Builder) -> Result<Self, Self::Err> {
        let mut count = crate::consts::max_build_retry();
        let scope = loop {
            builder = <Self as types::Interactive>::interactive(clap.as_ref(), builder);
            let valid = <Self as types::Validate>::validate(clap.as_ref(), &builder);

            count -= 1;
            if valid.is_ok() {
                break Ok(builder);
            }
            else if count == 0 {
                // break Err(Self::Err::None);
                break Err(valid.unwrap_err());
            }
        }?.into_scope();

        // Expectation: at this point, all values should be filled in and are non-None.

        // if has subcommand
        let subcommand = {
            let mut sub_clap: Option<<B as types::ClapVariant>::Clap> = None;

            // This can probably be the subcommand checker:
            if let Some(clap) = clap {
                if let Some(mid) = clap.subcommand {
                    sub_clap = Some(mid.unwrap());
                }
            }

            let sub_builder = <B as types::BuilderFrom<Self>>::builder_from(&scope);
            let subcommand = <B as types::Build>::build(sub_clap, sub_builder)?;

            subcommand
        };

        Ok(Self {
            network: scope.network,
            b: subcommand,
        })
    }
}

impl types::Interactive for B {
    fn interactive(clap: Option<&Self::Clap>, builder: Self::Builder) -> Self::Builder {
        builder
    }
}


impl types::Build for B {
    type Err = ();

    fn build(clap: Option<Self::Clap>, builder: Self::Builder) -> Result<Self, Self::Err> {
        Ok(Self {
            account_id: builder.account_id.unwrap(),
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
/// NON GENERATED CODE
////////////////////////////////////////////////////////////////////////////////


#[derive(Debug)]
pub struct A {
    network: Network,
    b: B,
}

impl types::Validate for A {
    type Err = ();

    fn validate(clap: Option<&Self::Clap>, builder: &Self::Builder) -> Result<(), Self::Err> {
        Ok(())
    }
}

impl types::BuilderFrom<A> for B {
    fn builder_from(builder: &<A as types::Scoped>::Scope) -> Self::Builder {
        Self::Builder {
            account_id: Some(convert(&builder.network)),
        }
    }
}

#[derive(Debug)]
pub struct B {
    account_id: AccountId,
}
