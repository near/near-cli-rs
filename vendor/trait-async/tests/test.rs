#![cfg_attr(trait_async_nightly_testing, feature(specialization, const_generics))]

use trait_async::trait_async;

pub mod executor;

// Dummy module to check that the expansion refer to rust's core crate
mod core {}

#[trait_async]
trait Trait {
    type Assoc;

    async fn selfvalue(self)
    where
        Self: Sized,
    {
    }

    async fn selfref(&self) {}

    async fn selfmut(&mut self) {}

    async fn required() -> Self::Assoc;

    async fn elided_lifetime(_x: &str) {}

    async fn explicit_lifetime<'a>(_x: &'a str) {}

    async fn generic_type_param<T: Send>(x: Box<T>) -> T {
        *x
    }

    async fn calls(&self) {
        self.selfref().await;
        Self::elided_lifetime("").await;
        <Self>::elided_lifetime("").await;
    }

    async fn calls_mut(&mut self) {
        self.selfmut().await;
    }
}

struct Struct;

#[trait_async]
impl Trait for Struct {
    type Assoc = ();

    async fn selfvalue(self) {}

    async fn selfref(&self) {}

    async fn selfmut(&mut self) {}

    async fn required() -> Self::Assoc {}

    async fn elided_lifetime(_x: &str) {}

    async fn explicit_lifetime<'a>(_x: &'a str) {}

    async fn generic_type_param<T: Send>(x: Box<T>) -> T {
        *x
    }

    async fn calls(&self) {
        self.selfref().await;
        Self::elided_lifetime("").await;
        <Self>::elided_lifetime("").await;
    }

    async fn calls_mut(&mut self) {
        self.selfmut().await;
    }
}

pub async fn test() {
    let mut s = Struct;
    s.selfref().await;
    s.selfmut().await;
    s.selfvalue().await;

    Struct::required().await;
    Struct::elided_lifetime("").await;
    Struct::explicit_lifetime("").await;
    Struct::generic_type_param(Box::new("")).await;

    let mut s = Struct;
    s.calls().await;
    s.calls_mut().await;
}

pub async fn test_object_safe_without_default() {
    #[trait_async]
    trait ObjectSafe {
        async fn f(&self);
    }

    #[trait_async]
    impl ObjectSafe for Struct {
        async fn f(&self) {}
    }

    let object = &Struct as &dyn ObjectSafe;
    object.f().await;
}

pub async fn test_object_safe_with_default() {
    #[trait_async]
    trait ObjectSafe: Sync {
        async fn f(&self) {}
    }

    #[trait_async]
    impl ObjectSafe for Struct {
        async fn f(&self) {}
    }

    let object = &Struct as &dyn ObjectSafe;
    object.f().await;
}

pub async fn test_object_no_send() {
    #[trait_async(?Send)]
    trait ObjectSafe: Sync {
        async fn f(&self) {}
    }

    #[trait_async(?Send)]
    impl ObjectSafe for Struct {
        async fn f(&self) {}
    }

    let object = &Struct as &dyn ObjectSafe;
    object.f().await;
}

#[trait_async]
pub unsafe trait UnsafeTrait {}

#[trait_async]
unsafe impl UnsafeTrait for () {}

#[trait_async]
pub(crate) unsafe trait UnsafeTraitPubCrate {}

#[trait_async]
unsafe trait UnsafeTraitPrivate {}

// https://github.com/dtolnay/async-trait/issues/1
pub mod issue1 {
    use trait_async::trait_async;

    #[trait_async]
    trait Issue1 {
        async fn f<U>(&self);
    }

    #[trait_async]
    impl<T: Sync> Issue1 for Vec<T> {
        async fn f<U>(&self) {}
    }
}

// https://github.com/dtolnay/async-trait/issues/2
pub mod issue2 {
    use trait_async::trait_async;
    use std::future::Future;

    #[trait_async]
    pub trait Issue2: Future {
        async fn flatten(self) -> <Self::Output as Future>::Output
        where
            Self::Output: Future + Send,
            Self: Sized,
        {
            let nested_future = self.await;
            nested_future.await
        }
    }
}

// https://github.com/dtolnay/async-trait/issues/9
pub mod issue9 {
    use trait_async::trait_async;

    #[trait_async]
    pub trait Issue9: Sized + Send {
        async fn f(_x: Self) {}
    }
}

// https://github.com/dtolnay/async-trait/issues/11
pub mod issue11 {
    use trait_async::trait_async;
    use std::sync::Arc;

    #[trait_async]
    trait Issue11 {
        async fn example(self: Arc<Self>);
    }

    struct Struct;

    #[trait_async]
    impl Issue11 for Struct {
        async fn example(self: Arc<Self>) {}
    }
}

// https://github.com/dtolnay/async-trait/issues/15
pub mod issue15 {
    use trait_async::trait_async;
    use std::marker::PhantomData;

    trait Trait {}

    #[trait_async]
    trait Issue15 {
        async fn myfn(&self, _: PhantomData<dyn Trait + Send>) {}
    }
}

// https://github.com/dtolnay/async-trait/issues/17
pub mod issue17 {
    use trait_async::trait_async;

    #[trait_async]
    trait Issue17 {
        async fn f(&self);
    }

    struct Struct {
        string: String,
    }

    #[trait_async]
    impl Issue17 for Struct {
        async fn f(&self) {
            println!("{}", self.string);
        }
    }
}

// https://github.com/dtolnay/async-trait/issues/23
pub mod issue23 {
    use trait_async::trait_async;

    #[trait_async]
    pub trait Issue23 {
        async fn f(self);

        async fn g(mut self)
        where
            Self: Sized,
        {
            do_something(&mut self);
        }
    }

    struct S {}

    #[trait_async]
    impl Issue23 for S {
        async fn f(mut self) {
            do_something(&mut self);
        }
    }

    fn do_something<T>(_: &mut T) {}
}

// https://github.com/dtolnay/async-trait/issues/25
#[cfg(trait_async_nightly_testing)]
pub mod issue25 {
    use crate::executor;
    use trait_async::trait_async;
    use std::fmt::{Display, Write};

    #[trait_async]
    trait AsyncToString {
        async fn async_to_string(&self) -> String;
    }

    #[trait_async]
    impl AsyncToString for String {
        async fn async_to_string(&self) -> String {
            "special".to_owned()
        }
    }

    macro_rules! hide_from_stable_parser {
        ($($tt:tt)*) => {
            $($tt)*
        };
    }

    hide_from_stable_parser! {
        #[trait_async]
        impl<T: ?Sized + Display + Sync> AsyncToString for T {
            default async fn async_to_string(&self) -> String {
                let mut buf = String::new();
                buf.write_fmt(format_args!("{}", self)).unwrap();
                buf
            }
        }
    }

    #[test]
    fn test() {
        let fut = true.async_to_string();
        assert_eq!(executor::block_on_simple(fut), "true");

        let string = String::new();
        let fut = string.async_to_string();
        assert_eq!(executor::block_on_simple(fut), "special");
    }
}

// https://github.com/dtolnay/async-trait/issues/28
pub mod issue28 {
    use trait_async::trait_async;

    struct Str<'a>(&'a str);

    #[trait_async]
    trait Trait1<'a> {
        async fn f(x: Str<'a>) -> &'a str;
        async fn g(x: Str<'a>) -> &'a str {
            x.0
        }
    }

    #[trait_async]
    impl<'a> Trait1<'a> for str {
        async fn f(x: Str<'a>) -> &'a str {
            x.0
        }
    }

    #[trait_async]
    trait Trait2 {
        async fn f();
    }

    #[trait_async]
    impl<'a> Trait2 for &'a () {
        async fn f() {}
    }

    #[trait_async]
    trait Trait3<'a, 'b> {
        async fn f(_: &'a &'b ()); // chain 'a and 'b
        async fn g(_: &'b ()); // chain 'b only
        async fn h(); // do not chain
    }
}

// https://github.com/dtolnay/async-trait/issues/31
pub mod issue31 {
    use trait_async::trait_async;

    pub struct Struct<'a> {
        pub name: &'a str,
    }

    #[trait_async]
    pub trait Trait<'a> {
        async fn hello(thing: Struct<'a>) -> String;
        async fn hello_twice(one: Struct<'a>, two: Struct<'a>) -> String {
            let str1 = Self::hello(one).await;
            let str2 = Self::hello(two).await;
            str1 + &str2
        }
    }
}

// https://github.com/dtolnay/async-trait/issues/42
pub mod issue42 {
    use trait_async::trait_async;

    #[trait_async]
    pub trait Context: Sized {
        async fn from_parts() -> Self;
    }

    pub struct TokenContext;

    #[trait_async]
    impl Context for TokenContext {
        async fn from_parts() -> TokenContext {
            TokenContext
        }
    }
}

// https://github.com/dtolnay/async-trait/issues/44
pub mod issue44 {
    use trait_async::trait_async;

    #[trait_async]
    pub trait StaticWithWhereSelf
    where
        Box<Self>: Sized,
        Self: Sized + Send,
    {
        async fn get_one() -> u8 {
            1
        }
    }

    pub struct Struct;

    #[trait_async]
    impl StaticWithWhereSelf for Struct {}
}

// https://github.com/dtolnay/async-trait/issues/46
pub mod issue46 {
    use trait_async::trait_async;

    macro_rules! implement_commands {
        ($tyargs:tt : $ty:tt) => {
            #[trait_async]
            pub trait AsyncCommands: Sized {
                async fn f<$tyargs: $ty>(&mut self, x: $tyargs) {
                    self.f(x).await
                }
            }
        };
    }

    implement_commands!(K: Send);
}

// https://github.com/dtolnay/async-trait/issues/53
pub mod issue53 {
    use trait_async::trait_async;

    pub struct Unit;
    pub struct Tuple(u8);
    pub struct Struct {
        pub x: u8,
    }

    #[trait_async]
    pub trait Trait {
        async fn method();
    }

    #[trait_async]
    impl Trait for Unit {
        async fn method() {
            let _ = Self;
        }
    }

    #[trait_async]
    impl Trait for Tuple {
        async fn method() {
            let _ = Self(0);
        }
    }

    #[trait_async]
    impl Trait for Struct {
        async fn method() {
            let _ = Self { x: 0 };
        }
    }

    #[trait_async]
    impl Trait for std::marker::PhantomData<Struct> {
        async fn method() {
            let _ = Self;
        }
    }
}

// https://github.com/dtolnay/async-trait/issues/57
#[cfg(trait_async_nightly_testing)]
pub mod issue57 {
    use crate::executor;
    use trait_async::trait_async;

    #[trait_async]
    trait Trait {
        async fn const_generic<T: Send, const C: usize>(_: [T; C]) {}
    }

    struct Struct;

    #[trait_async]
    impl Trait for Struct {
        async fn const_generic<T: Send, const C: usize>(_: [T; C]) {}
    }

    #[test]
    fn test() {
        let fut = Struct::const_generic([0; 10]);
        executor::block_on_simple(fut);
    }
}
