//! This module contains constant values and functions that are used around the crate.
//!
//! This includes things such as cryptographically-secure random salt/master key/nonce generation,
//! lengths for master keys and even the streaming block size.
use rand::{seq::SliceRandom, RngCore, SeedableRng};
use zeroize::Zeroize;

use crate::{
	crypto::stream::Algorithm,
	header::{
		file::FileHeaderVersion, keyslot::KeyslotVersion, metadata::MetadataVersion,
		preview_media::PreviewMediaVersion,
	},
	Error, Protected, Result,
};

/// This is the default salt size, and the recommended size for argon2id.
pub const SALT_LEN: usize = 16;

/// The size used for streaming encryption/decryption. This size seems to offer the best performance compared to alternatives.
///
/// The file size gain is 16 bytes per 1048576 bytes (due to the AEAD tag)
pub const BLOCK_SIZE: usize = 1_048_576;

/// The length of the encrypted master key
pub const ENCRYPTED_MASTER_KEY_LEN: usize = 48;

/// The length of the (unencrypted) master key
pub const MASTER_KEY_LEN: usize = 32;

pub const PASSPHRASE_LEN: usize = 7;

pub const LATEST_FILE_HEADER: FileHeaderVersion = FileHeaderVersion::V1;
pub const LATEST_KEYSLOT: KeyslotVersion = KeyslotVersion::V1;
pub const LATEST_METADATA: MetadataVersion = MetadataVersion::V1;
pub const LATEST_PREVIEW_MEDIA: PreviewMediaVersion = PreviewMediaVersion::V1;

/// This should be used for generating nonces for encryption.
///
/// An algorithm is required so this function can calculate the length of the nonce.
///
/// This function uses `ChaCha20Rng` for generating cryptographically-secure random data
#[must_use]
pub fn generate_nonce(algorithm: Algorithm) -> Vec<u8> {
	let mut nonce = vec![0u8; algorithm.nonce_len()];
	rand_chacha::ChaCha20Rng::from_entropy().fill_bytes(&mut nonce);
	nonce
}

/// This should be used for generating salts for hashing.
///
/// This function uses `ChaCha20Rng` for generating cryptographically-secure random data
#[must_use]
pub fn generate_salt() -> [u8; SALT_LEN] {
	let mut salt = [0u8; SALT_LEN];
	rand_chacha::ChaCha20Rng::from_entropy().fill_bytes(&mut salt);
	salt
}

/// This generates a master key, which should be used for encrypting the data
///
/// This is then stored (encrypted) within the header.
///
/// This function uses `ChaCha20Rng` for generating cryptographically-secure random data
#[must_use]
pub fn generate_master_key() -> Protected<[u8; MASTER_KEY_LEN]> {
	let mut master_key = [0u8; MASTER_KEY_LEN];
	rand_chacha::ChaCha20Rng::from_entropy().fill_bytes(&mut master_key);
	Protected::new(master_key)
}

/// This is used for converting a `Vec<u8>` to an array of bytes
///
/// It's main usage is for converting an encrypted master key from a `Vec<u8>` to `[u8; ENCRYPTED_MASTER_KEY_LEN]`
///
/// As the master key is encrypted at this point, it does not need to be `Protected<>`
///
/// This function still `zeroize`s any data it can
pub fn to_array<const I: usize>(bytes: Vec<u8>) -> Result<[u8; I]> {
	bytes.try_into().map_err(|mut b: Vec<u8>| {
		b.zeroize();
		Error::VecArrSizeMismatch
	})
}

/// This generates a 7 word diceware passphrase, separated with `-`
#[must_use]
pub fn generate_passphrase() -> Protected<String> {
	let wordlist = include_str!("../assets/eff_large_wordlist.txt")
		.lines()
		.collect::<Vec<&str>>();

	let words: Vec<String> = wordlist
		.choose_multiple(
			&mut rand_chacha::ChaCha20Rng::from_entropy(),
			PASSPHRASE_LEN,
		)
		.map(ToString::to_string)
		.collect();

	let passphrase = words
		.iter()
		.enumerate()
		.map(|(i, word)| {
			if i < PASSPHRASE_LEN - 1 {
				word.clone() + "-"
			} else {
				word.clone()
			}
		})
		.into_iter()
		.collect();

	Protected::new(passphrase)
}

const PASSWORD_CHARS: [char; 94] = [
	'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
	't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
	'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
	'5', '6', '7', '8', '9', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '-', '=',
	'{', '}', '[', ']', ':', '"', '\'', ';', '<', '>', '?', ',', '.', '/', '\\', '|', '`', '~',
];

#[must_use]
pub fn generate_password(length: usize) -> Protected<String> {
	let mut rng = rand_chacha::ChaCha20Rng::from_entropy();
	let mut chars: Vec<String> = Vec::new();

	for _ in 0..length / 4 {
		chars.push(
			PASSWORD_CHARS
				.choose_multiple(&mut rng, 4)
				.map(ToString::to_string)
				.collect(),
		);
	}

	Protected::new(chars.iter().map(ToString::to_string).collect())
}
