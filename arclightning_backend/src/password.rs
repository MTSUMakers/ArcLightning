pub extern crate blake2_rfc;
extern crate hex; 
 pub use password::blake2_rfc::blake2b::blake2b;
 pub fn check_password(password: String, hash: &[u8]) -> bool {
    hex::encode(blake2b(64, &[], password.as_bytes())).as_bytes() == hash
}
