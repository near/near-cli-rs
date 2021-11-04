bip39
=====

A Rust implementation of [BIP-39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
mnemonic codes.


## Word lists (languages)

We support all languages
[specified in the BIP-39 standard](https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md)
as of writing.

The English language is always loaded and other languages can be loaded using the corresponding feature.

Use the `all-languages` feature to enable all languages.

- English (always enabled)
- Simplified Chinese (`chinese-simplified`)
- Traditional Chinese (`chinese-traditional`)
- Czech (`czech`)
- French (`french`)
- Italian (`italian`)
- Japanese (`japanese`)
- Korean (`korean`)
- Spanish (`spanish`)


## MSRV

This crate supports Rust v1.29 and up and works with `no_std`.

