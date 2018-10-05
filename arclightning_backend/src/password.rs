extern crate blake2_rfc;

pub use password::blake2_rfc::blake2b::blake2b;

pub fn check_password(password: String, hash: String) -> bool {
    blake2b(64, &[], password.as_bytes()).as_bytes() == hash.as_bytes()
}
