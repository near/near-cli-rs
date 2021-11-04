#![warn(rust_2018_idioms, single_use_lifetimes)]

use easy_ext::ext;

#[test]
fn simple() {
    #[ext]
    impl str {
        fn foo(&self, pat: &str) -> String {
            self.replace(pat, "_")
        }
    }

    assert_eq!("--".foo("-"), "__");
}

#[test]
fn params() {
    #[ext]
    impl<T, E> Result<T, E> {
        fn err_into<U>(self) -> Result<T, U>
        where
            E: Into<U>,
        {
            self.map_err(Into::into)
        }
    }

    let err: Result<(), _> = Err(1_u32);
    assert_eq!(err.err_into::<u64>().unwrap_err(), 1_u64);
}

#[test]
fn lifetime() {
    #[ext(OptionExt)]
    impl<'a, T> &'a mut Option<T> {
        fn into_ref(self) -> Option<&'a T> {
            self.as_ref()
        }
    }

    let _: Option<&u8> = (&mut Some(1)).into_ref();
}

/*

## Visibility

* The generated extension trait inherits the visibility of the item in the original `impl`.

* The visibility of all the items in the original `impl` must be identical.

*/

mod bar {
    use easy_ext::ext;

    // assoc-item-level visibility + named
    #[ext(StrExt)]
    impl str {
        pub const FOO1: &'static str = "_";

        pub fn foo1(&self, pat: &str) -> String {
            self.replace(pat, Self::FOO1)
        }
    }

    // assoc-item-level visibility + unnamed
    #[ext]
    impl str {
        pub const FOO2: &'static str = "_";

        pub fn foo2(&self, pat: &str) -> String {
            self.replace(pat, Self::FOO2)
        }
    }

    // impl-level visibility + named
    #[ext(pub StrExt2)]
    impl str {
        const FOO3: &'static str = "_";

        fn foo3(&self, pat: &str) -> String {
            self.replace(pat, Self::FOO3)
        }
    }

    // impl-level visibility + unnamed
    #[ext(pub)]
    impl str {
        const FOO4: &'static str = "_";

        fn foo4(&self, pat: &str) -> String {
            self.replace(pat, Self::FOO4)
        }
    }

    pub(super) mod baz {
        use easy_ext::ext;

        #[ext(StrExt3)]
        impl str {
            pub(super) fn bar(&self, pat: &str) -> String {
                self.replace(pat, "_")
            }
        }

        #[ext(StrExt4)]
        impl str {
            pub fn baz(&self, pat: &str) -> String {
                self.replace(pat, "_")
            }

            pub fn baz2(&self, pat: &str) -> String {
                self.replace(pat, "-")
            }
        }

        #[ext(pub(super) StrExt5)]
        impl str {
            fn bar2(&self, pat: &str) -> String {
                self.replace(pat, "_")
            }
        }

        #[ext(pub StrExt6)]
        impl str {
            fn baz3(&self, pat: &str) -> String {
                self.replace(pat, "_")
            }

            fn baz4(&self, pat: &str) -> String {
                self.replace(pat, "-")
            }
        }
    }
}

#[test]
fn visibility() {
    use self::bar::{
        baz::{StrExt4, StrExt6},
        StrExt, StrExt2,
    };

    assert_eq!("..".foo1("."), "__");
    assert_eq!("..".foo3("."), "__");
    assert_eq!("..".baz("."), "__");
    assert_eq!("..".baz2("."), "--");
    assert_eq!("..".baz3("."), "__");
    assert_eq!("..".baz4("."), "--");
}

#[test]
fn generics() {
    #[ext(IterExt)]
    impl<I: IntoIterator> I {
        fn _next(self) -> Option<I::Item> {
            self.into_iter().next()
        }
    }

    assert_eq!(vec![1, 2, 3]._next(), Some(1_u8));
}

#[test]
fn trait_generics() {
    #[derive(Debug, PartialEq, Eq)]
    struct A {}

    impl Iterator for A {
        type Item = ();
        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }

    #[ext(ConstInit)]
    impl A {
        const INIT: Self = Self {};
        const INIT2: A = A {};
    }

    #[ext(Ext)]
    impl<I: Iterator + ConstInit> I {
        const CONST: Self = Self::INIT;
        const CONST2: I = I::INIT;
        fn method(mut self) -> Option<Self::Item> {
            self.next()
        }
        fn method2(mut self) -> Option<I::Item> {
            self.next()
        }
    }

    fn a<T: Ext + Eq + std::fmt::Debug>(mut x: T) {
        let y = T::CONST;
        let _ = T::CONST2;
        assert_eq!(x, y);
        assert!(x.next().is_none());
    }

    assert_eq!(A {}.method(), None);
    assert_eq!(A {}.method2(), None);

    a(A::INIT);
    a(A::INIT2);
}

#[test]
fn inline() {
    #[ext]
    impl str {
        #[inline]
        fn auto(&self) {}
        #[inline(always)]
        fn always(&self) {}
        #[inline(never)]
        fn never(&self) {}
        #[inline]
        #[inline]
        fn multiple(&self) {}
    }
}
