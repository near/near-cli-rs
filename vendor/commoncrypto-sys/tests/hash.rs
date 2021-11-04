extern crate commoncrypto_sys;
extern crate hex;

use hex::ToHex;

const TO_HASH: &'static str = "The quick brown fox jumps over the lazy dog";
const TO_HASH_MD5: &'static str = "9e107d9d372bb6826bd81d3542a419d6";
const TO_HASH_SHA1: &'static str = "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12";
const TO_HASH_SHA256: &'static str = "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592";
const TO_HASH_SHA384: &'static str = "ca737f1014a48f4c0b6dd43cb177b0afd9e5169367544c494011e3317dbf9a509cb1e5dc1e85a941bbee3d7f2afbc9b1";
const TO_HASH_SHA512: &'static str = "07e547d9586f6a73f73fbac0435ed76951218fb7d0c8d788a309d785436bbb642e93a252a954f23912547d1e8a3b5ed6e1bfd7097821233fa0538f3db854fee6";

macro_rules! test_cc_hash {
    (
        $test_name: ident,
        $ctx_ty: ident,
        $digest_len: ident,
        $init_func: ident,
        $update_func: ident,
        $final_func: ident,
        $expected_hash: ident
    ) => {
        #[test]
        fn $test_name() {
            let mut ctx = commoncrypto_sys::$ctx_ty::default();
            let mut md = [0u8; commoncrypto_sys::$digest_len];
            unsafe {
                assert_eq!(commoncrypto_sys::$init_func(&mut ctx), 1);
                assert_eq!(commoncrypto_sys::$update_func(&mut ctx, TO_HASH.as_ptr(), TO_HASH.len()), 1);
                assert_eq!(commoncrypto_sys::$final_func(md.as_mut_ptr(), &mut ctx), 1);
            }
            assert_eq!(md.to_vec().to_hex(), $expected_hash);
        }
    }
}

macro_rules! test_ccdigest {
    (
        $test_name: ident,
        $algorithm: ident,
        $digest_len: ident,
        $expected_hash: ident
    ) => {
        #[test]
        fn $test_name() {
            let mut md = [0u8; commoncrypto_sys::$digest_len];
            unsafe {
                assert_eq!(commoncrypto_sys::CCDigest(commoncrypto_sys::CCDigestAlgorithm::$algorithm,
                                           TO_HASH.as_ptr(),
                                           TO_HASH.len(),
                                           md.as_mut_ptr()), 0)
            }
            assert_eq!(md.to_vec().to_hex(), $expected_hash);
        }
    }
}

macro_rules! test_ccdigestgetoutputsize {
    (
        $test_name: ident,
        $algorithm: ident,
        $expected_digest_len: ident
    ) => {
        #[test]
        fn $test_name() {
            unsafe {
                assert_eq!(commoncrypto_sys::CCDigestGetOutputSize(commoncrypto_sys::CCDigestAlgorithm::$algorithm),
                           commoncrypto_sys::$expected_digest_len);
            }
        }
    }
}

test_cc_hash!(md5_hash,
              CC_MD5_CTX,
              MD5_DIGEST_LENGTH,
              CC_MD5_Init,
              CC_MD5_Update,
              CC_MD5_Final,
              TO_HASH_MD5);
test_cc_hash!(sha1_hash,
              CC_SHA_CTX,
              SHA1_DIGEST_LENGTH,
              CC_SHA1_Init,
              CC_SHA1_Update,
              CC_SHA1_Final,
              TO_HASH_SHA1);
test_cc_hash!(sha256_hash,
              CC_SHA256_CTX,
              SHA256_DIGEST_LENGTH,
              CC_SHA256_Init,
              CC_SHA256_Update,
              CC_SHA256_Final,
              TO_HASH_SHA256);
test_cc_hash!(sha384_hash,
              CC_SHA512_CTX,
              SHA384_DIGEST_LENGTH,
              CC_SHA384_Init,
              CC_SHA384_Update,
              CC_SHA384_Final,
              TO_HASH_SHA384);
test_cc_hash!(sha512_hash,
              CC_SHA512_CTX,
              SHA512_DIGEST_LENGTH,
              CC_SHA512_Init,
              CC_SHA512_Update,
              CC_SHA512_Final,
              TO_HASH_SHA512);

test_ccdigest!(md5_ccdigest, kCCDigestMD5, MD5_DIGEST_LENGTH, TO_HASH_MD5);
test_ccdigest!(sha1_ccdigest,
               kCCDigestSHA1,
               SHA1_DIGEST_LENGTH,
               TO_HASH_SHA1);
test_ccdigest!(sha256_ccdigest,
               kCCDigestSHA256,
               SHA256_DIGEST_LENGTH,
               TO_HASH_SHA256);
test_ccdigest!(sha384_ccdigest,
               kCCDigestSHA384,
               SHA384_DIGEST_LENGTH,
               TO_HASH_SHA384);
test_ccdigest!(sha512_ccdigest,
               kCCDigestSHA512,
               SHA512_DIGEST_LENGTH,
               TO_HASH_SHA512);

test_ccdigestgetoutputsize!(md5_ccdigestoutputsize, kCCDigestMD5, MD5_DIGEST_LENGTH);
test_ccdigestgetoutputsize!(sha1_ccdigestoutputsize, kCCDigestSHA1, SHA1_DIGEST_LENGTH);
test_ccdigestgetoutputsize!(sha256_ccdigestoutputsize,
                            kCCDigestSHA256,
                            SHA256_DIGEST_LENGTH);
test_ccdigestgetoutputsize!(sha384_ccdigestoutputsize,
                            kCCDigestSHA384,
                            SHA384_DIGEST_LENGTH);
test_ccdigestgetoutputsize!(sha512_ccdigestoutputsize,
                            kCCDigestSHA512,
                            SHA512_DIGEST_LENGTH);
