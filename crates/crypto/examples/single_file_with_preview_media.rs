#![cfg(feature = "headers")]

use tokio::fs::File;

use sd_crypto::{
	crypto::Encryptor,
	header::file::{FileHeader, HeaderObjectType},
	primitives::LATEST_FILE_HEADER,
	types::{Algorithm, HashingAlgorithm, Key, Params, Salt},
	Protected,
};

const ALGORITHM: Algorithm = Algorithm::XChaCha20Poly1305;
const HASHING_ALGORITHM: HashingAlgorithm = HashingAlgorithm::Argon2id(Params::Standard);

async fn encrypt() {
	let password = Protected::new(b"password".to_vec());

	// Open both the source and the output file
	let mut reader = File::open("test").await.unwrap();
	let mut writer = File::create("test.encrypted").await.unwrap();

	// This needs to be generated here, otherwise we won't have access to it for encryption
	let master_key = Key::generate();

	// These should ideally be done by a key management system
	let content_salt = Salt::generate();
	let hashed_password = HASHING_ALGORITHM
		.hash(password, content_salt, None)
		.unwrap();

	let pvm = b"a nice mountain".to_vec();

	// Create the header for the encrypted file
	let mut header = FileHeader::new(LATEST_FILE_HEADER, ALGORITHM).unwrap();

	// Create a keyslot to be added to the header
	header
		.add_keyslot(
			HASHING_ALGORITHM,
			content_salt,
			hashed_password,
			master_key.clone(),
		)
		.await
		.unwrap();

	header
		.add_object(HeaderObjectType::PreviewMedia, master_key.clone(), &pvm)
		.await
		.unwrap();

	// Write the header to the file
	header.write(&mut writer).await.unwrap();

	// Use the nonce created by the header to initialise a stream encryption object
	let encryptor = Encryptor::new(master_key, header.get_nonce(), header.get_algorithm()).unwrap();

	// Encrypt the data from the reader, and write it to the writer
	// Use AAD so the header can be authenticated against every block of data
	encryptor
		.encrypt_streams(&mut reader, &mut writer, &header.get_aad())
		.await
		.unwrap();
}

async fn decrypt_preview_media() {
	let password = Protected::new(b"password".to_vec());

	// Open the encrypted file
	let mut reader = File::open("test.encrypted").await.unwrap();

	// Deserialize the header, keyslots, etc from the encrypted file
	let header = FileHeader::from_reader(&mut reader).await.unwrap();

	let master_key = header
		.decrypt_master_key_with_password(password)
		.await
		.unwrap();

	// Decrypt the preview media
	let media = header
		.decrypt_object(HeaderObjectType::PreviewMedia, master_key)
		.await
		.unwrap();

	println!("{:?}", media.expose());
}

#[tokio::main]
async fn main() {
	encrypt().await;

	decrypt_preview_media().await;
}
