use trait_async::trait_async;

macro_rules! picky {
    (ident) => {};
}

#[trait_async]
trait Trait {
    async fn method();
}

struct Struct;

#[trait_async]
impl Trait for Struct {
    async fn method() {
        picky!({ 123 });
    }
}

fn main() {}
