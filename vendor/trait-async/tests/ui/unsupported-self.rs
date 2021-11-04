use trait_async::trait_async;

#[trait_async]
pub trait Trait {
    async fn method();
}

#[trait_async]
impl Trait for &'static str {
    async fn method() {
        let _ = Self;
    }
}

fn main() {}
