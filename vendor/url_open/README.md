# url_open
A simple crate to open URLs in the default web browser.

### Usage

```rust
extern crate url;
extern crate url_open;

use url::Url;
use url_open::UrlOpen;

fn main() {
    Url::parse("https://github.com/overdrivenpotato/url_open").unwrap().open();
}
```
