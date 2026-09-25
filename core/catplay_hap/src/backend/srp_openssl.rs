use alloc::{vec, vec::Vec};
use core::{cmp::Ordering, result::Result};
use openssl::{
    bn::{BigNum, BigNumContext, BigNumRef},
    error::ErrorStack,
};
use ring::digest;

const SRP_USERNAME_IN_X: bool = true;

#[derive(Debug, Clone, Copy, Default)]
pub struct Client;

#[derive(Debug, Clone, Copy, Default)]
pub struct Server;

#[derive(Debug, Clone)]
pub struct ClientVerifier {
    key: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct LegacyServerVerifier {
    key: Vec<u8>,
}

impl Client {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn compute_verifier(&self, username: &[u8], password: &[u8], salt: &[u8]) -> Result<Vec<u8>, ErrorStack> {
        let x = compute_x(identity_username(username), password, salt);
        let g = group_g()?;
        let n = group_n()?;
        mod_exp(&g, &x, &n)
    }

    pub fn compute_public_ephemeral(&self, a: &[u8]) -> Result<Vec<u8>, ErrorStack> {
        let a = BigNum::from_slice(a)?;
        let g = group_g()?;
        let n = group_n()?;
        mod_exp(&g, &a, &n)
    }

    pub fn process_reply(
        &self,
        a: &[u8],
        username: &[u8],
        password: &[u8],
        salt: &[u8],
        b_pub_bytes: &[u8],
    ) -> Result<ClientVerifier, ErrorStack> {
        let n = group_n()?;
        let g = group_g()?;
        let a = BigNum::from_slice(a)?;
        let a_pub_bytes = mod_exp(&g, &a, &n)?;
        let b_pub = BigNum::from_slice(b_pub_bytes)?;

        validate_pub(&b_pub, &n)?;

        let u = hash_to_bn(&[&a_pub_bytes, b_pub_bytes])?;
        let k = compute_k(&n, &g)?;
        let x = compute_x(identity_username(username), password, salt);
        let g_x = mod_exp_bn(&g, &x, &n)?;
        let ux = mul_bn(&u, &x)?;
        let exp = add_bn(&a, &ux)?;
        let kgx = mod_mul_bn(&k, &g_x, &n)?;
        let base = mod_sub_bn(&b_pub, &kgx, &n)?;
        let key = mod_exp(&base, &exp, &n)?;

        Ok(ClientVerifier { key })
    }
}

impl Server {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn compute_public_ephemeral(&self, b: &[u8], verifier: &[u8]) -> Result<Vec<u8>, ErrorStack> {
        let n = group_n()?;
        let g = group_g()?;
        let b = BigNum::from_slice(b)?;
        let v = BigNum::from_slice(verifier)?;
        let k = compute_k(&n, &g)?;
        let gv = mod_exp_bn(&g, &b, &n)?;
        let kv = mod_mul_bn(&k, &v, &n)?;
        mod_add(&kv, &gv, &n)
    }

    pub fn process_reply_legacy(&self, b: &[u8], verifier: &[u8], a_pub_bytes: &[u8]) -> Result<LegacyServerVerifier, ErrorStack> {
        let n = group_n()?;
        let g = group_g()?;
        let b = BigNum::from_slice(b)?;
        let v = BigNum::from_slice(verifier)?;
        let k = compute_k(&n, &g)?;

        let g_b = mod_exp_bn(&g, &b, &n)?;
        let kv = mod_mul_bn(&k, &v, &n)?;
        let b_pub_bytes = mod_add(&kv, &g_b, &n)?;

        self.process_reply_legacy_with_b_pub_inner(&n, &b, &v, a_pub_bytes, &b_pub_bytes)
    }

    pub fn process_reply_legacy_with_b_pub(
        &self,
        b: &[u8],
        verifier: &[u8],
        a_pub_bytes: &[u8],
        b_pub_bytes: &[u8],
    ) -> Result<LegacyServerVerifier, ErrorStack> {
        let n = group_n()?;
        let b = BigNum::from_slice(b)?;
        let v = BigNum::from_slice(verifier)?;
        self.process_reply_legacy_with_b_pub_inner(&n, &b, &v, a_pub_bytes, b_pub_bytes)
    }

    fn process_reply_legacy_with_b_pub_inner(
        &self,
        n: &BigNumRef,
        b: &BigNumRef,
        v: &BigNumRef,
        a_pub_bytes: &[u8],
        b_pub_bytes: &[u8],
    ) -> Result<LegacyServerVerifier, ErrorStack> {
        let a_pub = BigNum::from_slice(a_pub_bytes)?;

        validate_pub(&a_pub, n)?;
        let u = hash_to_bn(&[a_pub_bytes, b_pub_bytes])?;
        let v_u = mod_exp_bn(v, &u, n)?;
        let base = mod_mul_bn(&a_pub, &v_u, n)?;
        let key = mod_exp(&base, b, n)?;

        Ok(LegacyServerVerifier { key })
    }
}

impl ClientVerifier {
    pub fn key(&self) -> &[u8] {
        &self.key
    }
}

impl LegacyServerVerifier {
    pub fn key(&self) -> &[u8] {
        &self.key
    }
}

fn identity_username(username: &[u8]) -> &[u8] {
    if SRP_USERNAME_IN_X { username } else { &[] }
}

fn compute_x(username: &[u8], password: &[u8], salt: &[u8]) -> BigNum {
    let identity_hash = sha512_parts(&[username, b":", password]);
    hash_to_bn(&[salt, &identity_hash]).expect("sha512 output is always a valid bignum")
}

fn compute_k(n: &BigNumRef, g: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let n_bytes = n.to_vec();
    let g_bytes = g.to_vec();
    let mut padded_g = vec![0u8; n_bytes.len()];
    let offset = padded_g.len() - g_bytes.len();
    padded_g[offset..].copy_from_slice(&g_bytes);
    hash_to_bn(&[&n_bytes, &padded_g])
}

fn validate_pub(value: &BigNumRef, modulus: &BigNumRef) -> Result<(), ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let mut reduced = BigNum::new()?;
    let zero = BigNum::new()?;
    reduced.nnmod(value, modulus, &mut ctx)?;
    if reduced.ucmp(&zero) == Ordering::Equal {
        return Err(ErrorStack::get());
    }
    Ok(())
}

fn group_n() -> Result<BigNum, ErrorStack> {
    BigNum::get_rfc3526_prime_3072()
}

fn group_g() -> Result<BigNum, ErrorStack> {
    BigNum::from_u32(5)
}

fn hash_to_bn(parts: &[&[u8]]) -> Result<BigNum, ErrorStack> {
    BigNum::from_slice(&sha512_parts(parts))
}

fn sha512_parts(parts: &[&[u8]]) -> [u8; 64] {
    let mut ctx = digest::Context::new(&digest::SHA512);
    for part in parts {
        ctx.update(part);
    }

    let mut out = [0u8; 64];
    out.copy_from_slice(ctx.finish().as_ref());
    out
}

fn add_bn(a: &BigNumRef, b: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let mut out = BigNum::new()?;
    out.checked_add(a, b)?;
    Ok(out)
}

fn mul_bn(a: &BigNumRef, b: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let mut out = BigNum::new()?;
    out.checked_mul(a, b, &mut ctx)?;
    Ok(out)
}

fn mod_add(a: &BigNumRef, b: &BigNumRef, n: &BigNumRef) -> Result<Vec<u8>, ErrorStack> {
    Ok(mod_add_bn(a, b, n)?.to_vec())
}

fn mod_add_bn(a: &BigNumRef, b: &BigNumRef, n: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let mut out = BigNum::new()?;
    out.mod_add(a, b, n, &mut ctx)?;
    Ok(out)
}

fn mod_sub_bn(a: &BigNumRef, b: &BigNumRef, n: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let mut out = BigNum::new()?;
    out.mod_sub(a, b, n, &mut ctx)?;
    Ok(out)
}

fn mod_mul_bn(a: &BigNumRef, b: &BigNumRef, n: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let mut out = BigNum::new()?;
    out.mod_mul(a, b, n, &mut ctx)?;
    Ok(out)
}

fn mod_exp(base: &BigNumRef, exp: &BigNumRef, modulus: &BigNumRef) -> Result<Vec<u8>, ErrorStack> {
    Ok(mod_exp_bn(base, exp, modulus)?.to_vec())
}

fn mod_exp_bn(base: &BigNumRef, exp: &BigNumRef, modulus: &BigNumRef) -> Result<BigNum, ErrorStack> {
    let mut ctx = BigNumContext::new()?;
    let mut out = BigNum::new()?;
    out.mod_exp(base, exp, modulus, &mut ctx)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{Client, Server};
    use crate::RingSha512;
    use srp::groups::G3072;

    #[test]
    fn openssl_client_verifier_matches_srp() {
        let client = Client::new();
        let ref_client = srp::Client::<G3072, RingSha512>::new();
        let salt = [0x11u8; 16];
        let a = [0x22u8; 64];
        let b = [0x33u8; 64];
        let verifier = ref_client.compute_verifier(b"Pair-Setup", b"3939", &salt);
        let b_pub = srp::Server::<G3072, RingSha512>::new().compute_public_ephemeral(&b, &verifier);

        assert_eq!(
            client
                .compute_verifier(b"Pair-Setup", b"3939", &salt)
                .unwrap(),
            verifier
        );
        assert_eq!(
            client.compute_public_ephemeral(&a).unwrap(),
            ref_client.compute_public_ephemeral(&a)
        );
        assert_eq!(
            client
                .process_reply(&a, b"Pair-Setup", b"3939", &salt, &b_pub)
                .unwrap()
                .key(),
            ref_client
                .process_reply(&a, b"Pair-Setup", b"3939", &salt, &b_pub)
                .unwrap()
                .key()
        );
    }

    #[test]
    fn openssl_server_legacy_matches_srp() {
        let ref_client = srp::Client::<G3072, RingSha512>::new();
        let ref_server = srp::Server::<G3072, RingSha512>::new();
        let server = Server::new();
        let salt = [0x44u8; 16];
        let a = [0x55u8; 64];
        let b = [0x66u8; 64];
        let verifier = ref_client.compute_verifier(b"Pair-Setup", b"3939", &salt);
        let a_pub = ref_client.compute_public_ephemeral(&a);

        assert_eq!(
            server.compute_public_ephemeral(&b, &verifier).unwrap(),
            ref_server.compute_public_ephemeral(&b, &verifier)
        );
        #[allow(deprecated)]
        let legacy_key = ref_server
            .process_reply_legacy(&b, &verifier, &a_pub)
            .unwrap()
            .key()
            .to_vec();
        assert_eq!(
            server
                .process_reply_legacy(&b, &verifier, &a_pub)
                .unwrap()
                .key(),
            legacy_key
        );
        let b_pub = ref_server.compute_public_ephemeral(&b, &verifier);
        assert_eq!(
            server
                .process_reply_legacy_with_b_pub(&b, &verifier, &a_pub, &b_pub)
                .unwrap()
                .key(),
            legacy_key
        );
    }
}
