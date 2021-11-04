mod foo {
    use easy_ext::ext;

    #[ext(StrExt1)]
    impl str {
        fn method1(&self, pat: &str) -> String {
            self.replace(pat, "_")
        }
    }

    #[ext(pub(self) StrExt2)]
    impl str {
        fn method2(&self, pat: &str) -> String {
            self.replace(pat, "_")
        }
    }

    pub mod bar {
        use easy_ext::ext;

        #[ext(pub(super) StrExt3)]
        impl str {
            fn method3(&self, pat: &str) -> String {
                self.replace(pat, "_")
            }
        }
    }

    #[allow(unused_imports)]
    use bar::StrExt3;
}

fn main() {
    use foo::StrExt1; //~ ERROR trait `StrExt1` is private [E0603]

    use foo::StrExt2; //~ ERROR trait `StrExt2` is private [E0603]

    use foo::bar::StrExt3; //~ ERROR trait `StrExt2` is private [E0603]
}
