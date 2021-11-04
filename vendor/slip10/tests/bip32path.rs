use core::str::FromStr;

use slip10::BIP32Path;

const HARDEND: u32 = 1 << 31;

#[test]
fn test_parse_path() {
    let smaples = vec![
        ("", BIP32Path::from(Vec::new())),
        ("m", BIP32Path::from(Vec::new())),
        ("m/0H", BIP32Path::from(vec![HARDEND + 0])),
        ("m/0H/1", BIP32Path::from(vec![HARDEND + 0, 1])),
        (
            "m/0H/1/2H",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2]),
        ),
        (
            "m/0H/1/2H/2",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2]),
        ),
        (
            "m/0H/1/2H/2/1000000000",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2, 1000000000]),
        ),
        ("0H", BIP32Path::from(vec![HARDEND + 0])),
        ("0H/1", BIP32Path::from(vec![HARDEND + 0, 1])),
        (
            "0H/1/2H",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2]),
        ),
        (
            "0H/1/2H/2",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2]),
        ),
        (
            "0H/1/2H/2/1000000000",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2, 1000000000]),
        ),
        ("m/0'", BIP32Path::from(vec![HARDEND + 0])),
        ("m/0'/1", BIP32Path::from(vec![HARDEND + 0, 1])),
        (
            "m/0'/1/2'",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2]),
        ),
        (
            "m/0'/1/2'/2",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2]),
        ),
        (
            "m/0'/1/2'/2/1000000000",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2, 1000000000]),
        ),
        ("0'", BIP32Path::from(vec![HARDEND + 0])),
        ("0'/1", BIP32Path::from(vec![HARDEND + 0, 1])),
        (
            "0'/1/2'",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2]),
        ),
        (
            "0'/1/2'/2",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2]),
        ),
        (
            "0'/1/2'/2/1000000000",
            BIP32Path::from(vec![HARDEND + 0, 1, HARDEND + 2, 2, 1000000000]),
        ),
        (
            "0/2147483647'/1/2147483646'/2",
            BIP32Path::from(vec![0, HARDEND + 2147483647, 1, HARDEND + 2147483646, 2]),
        ),
        (
            "0/0/0/0/0/0/0/0/0/0/0/0/0/0/0/0",
            BIP32Path::from(vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
        ),
    ];

    for (bip32_path_str, expected_bip32_path) in smaples {
        let parsed_bip32_path = BIP32Path::from_str(bip32_path_str).unwrap();
        assert_eq!(parsed_bip32_path, expected_bip32_path);

        let bip32_path_normalized_str = bip32_path_str.replace('H', "'");
        let bip32_path_normalized_str = if !bip32_path_normalized_str.starts_with("m/") {
            if bip32_path_str == "m" {
                format!("m/")
            } else {
                format!("m/{}", bip32_path_normalized_str)
            }
        } else {
            bip32_path_normalized_str
        };
        assert_eq!(parsed_bip32_path.to_string(), bip32_path_normalized_str);
    }

    let errors = vec![
        "44'/2147483648",
        "44'/2147483648'",
        "44'/-1",
        "44'//0",
        "/0'/1/2'",
        "44'/'",
        "44'/'0",
        "44'/0h",
        "44'/0''",
        "44'/0H'",
        "wrong",
    ];

    for i in errors {
        assert!(BIP32Path::from_str(i).is_err());
    }
}
