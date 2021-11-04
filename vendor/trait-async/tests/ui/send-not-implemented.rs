use trait_async::trait_async;
use std::sync::Mutex;

async fn f() {}

#[trait_async]
trait Test {
    async fn test(&self) {
        let mutex = Mutex::new(());
        let _guard = mutex.lock().unwrap();
        f().await;
    }
}

fn main() {}
