// Appendix A:  Padding, (Page-24)
// 
// For the ECB, CBC, and CFB modes, the plaintext must be a sequence of one or more complete data blocks 
// (or, for CFB mode, data segments). In other words, for these three modes, the total number of bits in 
// the plaintext must be a positive multiple of the block (or segment) size. 


// 6.2 The Cipher Block Chaining Mode, (Page-17)
// https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
// 
// NOTE:
//      ECB 和 CBC 分组模式都无法处理不定长的输入数据，
//      需要自己手动为不定长数据按照块密码算法的块大小做对齐工作。
// 
use crate::blockcipher::{
    Rc2FixedSize, Sm4,
    Aes128, Aes192, Aes256,
    Camellia128, Camellia192, Camellia256,
};


macro_rules! impl_block_cipher_with_cbc_mode {
    ($name:tt, $cipher:tt) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            iv: [u8; Self::BLOCK_LEN],
            cipher: $cipher,
        }

        impl $name {
            pub const BLOCK_LEN: usize = $cipher::BLOCK_LEN;
            pub const KEY_LEN: usize   = $cipher::KEY_LEN;
            pub const NONCE_LEN: usize = $cipher::BLOCK_LEN;

            pub fn new(key: &[u8], nonce: &[u8]) -> Self {
                assert_eq!(key.len(), Self::KEY_LEN);
                assert_eq!(nonce.len(), Self::NONCE_LEN);

                let cipher = $cipher::new(key);
                let mut iv = [0u8; Self::BLOCK_LEN];
                iv[..Self::BLOCK_LEN].copy_from_slice(nonce);

                Self { cipher, iv }
            }
            
            /// the plaintext must be a sequence of one or more complete data blocks.
            /// the total number of bits in the plaintext must be a positive multiple 
            /// of the block (or segment) size.
            pub fn encrypt(&mut self, blocks: &mut [u8]) {
                assert_eq!(blocks.len() % Self::BLOCK_LEN, 0);

                let mut last_block = self.iv.clone();
                for plaintext in blocks.chunks_mut(Self::BLOCK_LEN) {
                    debug_assert_eq!(plaintext.len(), Self::BLOCK_LEN);

                    for i in 0..Self::BLOCK_LEN {
                        plaintext[i] ^= last_block[i];
                    }

                    self.cipher.encrypt(plaintext);

                    for i in 0..Self::BLOCK_LEN {
                        last_block[i] = plaintext[i];
                    }
                }
            }

            /// the plaintext must be a sequence of one or more complete data blocks.
            /// the total number of bits in the plaintext must be a positive multiple 
            /// of the block (or segment) size.
            pub fn decrypt(&mut self, blocks: &mut [u8]) {
                assert_eq!(blocks.len() % Self::BLOCK_LEN, 0);

                let mut last_block = self.iv.clone();
                for ciphertext in blocks.chunks_mut(Self::BLOCK_LEN) {
                    debug_assert_eq!(ciphertext.len(), Self::BLOCK_LEN);

                    let mut output_block = [0u8; Self::BLOCK_LEN];
                    for i in 0..Self::BLOCK_LEN {
                        output_block[i] = ciphertext[i];
                    }

                    self.cipher.decrypt(&mut output_block);
                    
                    for i in 0..Self::BLOCK_LEN {
                        output_block[i] ^= last_block[i];
                        last_block[i] = ciphertext[i];
                        ciphertext[i] = output_block[i];
                    }
                }
            }
        }
    }
}

impl_block_cipher_with_cbc_mode!(Aes128Cbc, Aes128);
impl_block_cipher_with_cbc_mode!(Aes192Cbc, Aes192);
impl_block_cipher_with_cbc_mode!(Aes256Cbc, Aes256);

impl_block_cipher_with_cbc_mode!(Camellia128Cbc, Camellia128);
impl_block_cipher_with_cbc_mode!(Camellia192Cbc, Camellia192);
impl_block_cipher_with_cbc_mode!(Camellia256Cbc, Camellia256);

impl_block_cipher_with_cbc_mode!(Rc2FixedSizeCbc, Rc2FixedSize);

impl_block_cipher_with_cbc_mode!(Sm4Cbc, Sm4);


#[cfg(test)]
#[bench]
fn bench_aes128_cbc_enc(b: &mut test::Bencher) {
    let key = hex::decode("00000000000000000000000000000000").unwrap();
    let nonce = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
    
    let mut cipher = Aes128Cbc::new(&key, &nonce);
    
    b.bytes = Aes128Cbc::BLOCK_LEN as u64;
    b.iter(|| {
        let mut ciphertext = test::black_box([1u8; Aes128Cbc::BLOCK_LEN]);
        cipher.encrypt(&mut ciphertext);
        ciphertext
    })
}
#[cfg(test)]
#[bench]
fn bench_aes128_cbc_dec(b: &mut test::Bencher) {
    let key = hex::decode("00000000000000000000000000000000").unwrap();
    let nonce = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
    
    let mut cipher = Aes128Cbc::new(&key, &nonce);
    
    b.bytes = Aes128Cbc::BLOCK_LEN as u64;
    b.iter(|| {
        let mut ciphertext = test::black_box([1u8; Aes128Cbc::BLOCK_LEN]);
        cipher.decrypt(&mut ciphertext);
        ciphertext
    })
}

#[test]
fn test_aes128_cbc_enc() {
    // F.2.1  CBC-AES128.Encrypt, (Page-34)
    // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
    let key   = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").unwrap();
    let nonce = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();

    let mut cipher = Aes128Cbc::new(&key, &nonce);

    let plaintext = hex::decode("\
6bc1bee22e409f96e93d7e117393172a\
ae2d8a571e03ac9c9eb76fac45af8e51\
30c81c46a35ce411e5fbc1191a0a52ef\
f69f2445df4f9b17ad2b417be66c3710").unwrap();

    let mut ciphertext = plaintext.clone();
    cipher.encrypt(&mut ciphertext);

    assert_eq!(&ciphertext[..], &hex::decode("\
7649abac8119b246cee98e9b12e9197d\
5086cb9b507219ee95db113a917678b2\
73bed6b8e3c1743b7116e69e22229516\
3ff1caa1681fac09120eca307586e1a7").unwrap()[..]);
}

#[test]
fn test_aes128_cbc_dec() {
    // F.2.2  CBC-AES128.Decrypt, (Page-34)
    // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
    let key   = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").unwrap();
    let nonce = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();

    let mut cipher = Aes128Cbc::new(&key, &nonce);

    let ciphertext = hex::decode("\
7649abac8119b246cee98e9b12e9197d\
5086cb9b507219ee95db113a917678b2\
73bed6b8e3c1743b7116e69e22229516\
3ff1caa1681fac09120eca307586e1a7").unwrap();

    let mut plaintext = ciphertext.clone();
    cipher.decrypt(&mut plaintext);

    assert_eq!(&plaintext[..], &hex::decode("\
6bc1bee22e409f96e93d7e117393172a\
ae2d8a571e03ac9c9eb76fac45af8e51\
30c81c46a35ce411e5fbc1191a0a52ef\
f69f2445df4f9b17ad2b417be66c3710").unwrap()[..]);
}