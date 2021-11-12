
pub trait Builder: Scoped {
    type Builder: Default;

    // fn scoped(&self) -> Self::Scope;
}

pub trait Scoped {
    type Scope: Clone;
}

pub trait IntoScope<T> {
    fn into_scope(&self) -> T;
}

pub trait ClapVariant {
    type Clap: clap::Parser;
}

// pub trait ClapSubcommand {
//     type Sub;

//     fn subcommand(&self) -> Self::Sub;
// }

pub trait BuilderFrom<T>: Builder
where
    T: Scoped,
{
    fn builder_from(prev: &T::Scope) -> Self::Builder;
}

pub trait Interactive: ClapVariant + Builder {
    fn interactive(clap: Option<&Self::Clap>, builder: Self::Builder) -> Self::Builder;
}

pub trait Validate: ClapVariant + Builder {
    // TODO: maybe a const for retrying
    type Err;

    fn validate(clap: Option<&Self::Clap>, builder: &Self::Builder) -> Result<(), Self::Err>;
}

pub trait Build: ClapVariant + Builder + Interactive
where
    Self: Sized,
{
    type Err;

    fn build(clap: Option<Self::Clap>, builder: Self::Builder) -> Result<Self, Self::Err>;
}

pub trait InteractiveParse: Build where Self: Sized {
    fn iparse() -> Result<Self, Self::Err>;
}

impl<T> InteractiveParse for T
where
    T: Build
{
    fn iparse() -> Result<Self, Self::Err> {
        let clap = <<T as ClapVariant>::Clap as clap::Parser>::parse();
        Self::build(Some(clap), <T as Builder>::Builder::default())
    }
}

pub trait Eclap: InteractiveParse {}
