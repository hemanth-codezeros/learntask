use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::RngCore;
use rand::rngs::OsRng;

pub fn generate_keypair() -> Result<([u8; 32], [u8; 32]), Box<dyn std::error::Error>> {
    let mut csprng = OsRng;
    let mut seed = [0u8; 32];
    csprng.fill_bytes(&mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    // Get the verifying key (public key) from the signing key
    let verifying_key = signing_key.verifying_key();

    // Convert keys to bytes for storage or transmission
    let private_key_bytes = signing_key.to_bytes();
    let public_key_bytes = verifying_key.to_bytes();
    Ok((public_key_bytes, private_key_bytes))
}

pub fn sign_messg(message: &[u8], private_key_bytes: [u8; 32]) -> Signature {
    let signing_key = SigningKey::from_bytes(&private_key_bytes);

    signing_key.sign(message)
}

pub fn verify_signature(
    message: &[u8],
    public_key_bytes: [u8; 32],
    signature: Signature,
) -> Result<bool, Box<dyn std::error::Error>> {
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes).unwrap();
    match verifying_key.verify(message, &signature) {
        Ok(_) => {
            println!("Signature verification succeeded! ✓");
            Ok(true)
        }
        Err(e) => {
            println!("Signature verification failed: {}", e);
            Ok(false)
        }
    }
}
