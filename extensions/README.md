# NEAR CLI extensions system
`NEAR CLI` is built to scale. The number of possible features is endless. Instead of choosing only some of them, we are creating an `Extensions System` that will empower our users to choose, build and share `NEAR CLI` functionality.

## How it works
Extensibility is achieved by translating a `NEAR CLI` invocation of the form `near (?<command>[^ ]+)` into an invocation of an external tool `near-${command}` that then needs to be present in one of the user's `$PATH` directories.
It means that you can write it in any language and with the use of any framework, it just needs to be called `near-cli-{extension-name}` and be installed on your system. This approach is inspired by [Cargo](https://github.com/rust-lang/cargo).

## How to build an extension
As mentioned above, any binary can become an extension, but we are encouraging developers to use [Rust](https://www.rust-lang.org/), [Clap](https://docs.rs/clap/2.33.0/clap/), and a set of libraries developed by NEAR. Here is some of them:
- `near-cli-builder` - CLI specific helpers to make your life easier and follow the standards of `NEAR CLI` at the same time (NOTE: Under development)
- `near-api-rs` - Rust library to interact with accounts and smart contracts on NEAR. (NOTE: Under development)
- [near-jsonrpc-client-rs](https://github.com/near/near-jsonrpc-client-rs) - Lower-level JSON RPC API for interfacing with the NEAR Protocol.

## Example
Core `NEAR CLI` does not have validator specific functionality, but we can add it as a simple bash script:

`near-cli-staking-pool-info.sh`
```bash
#!/bin/sh
POOL_ID=$1
near execute view-method network mainnet contract "name.near" call "get_fields_by_pool" '{"pool_id": "'"$POOL_ID"'"}' at-final-block
```

Make sure that this script is in your `$PATH` and has proper permissions to be executed. Then call it like this:

```bash
$ near staking-pool-info aurora.near
{
  "country": "Gibraltar",
  "country_code": "gi",
  "github": "auroraisnear",
  "twitter": "auroraisnear",
  "telegram": "auroraisnear",
  "url": "https://aurora.dev/",
  "city": "Gibraltar",
  "description": "Aurora validator fees are spent on supporting the Rainbow Bridge infrastructure, keeping the bridge free and accessible to everyone (except for the gas fees).",
  "logo": "https://aurora.dev/static/favicon-32x32.png",
  "name": "Aurora"
}
```
<!-- TODO: add example written in Rust when it will be available -->