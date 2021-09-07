use std::convert::TryInto;

use amplify::num::u256;

use bitcoin_hashes::{self, Hash};

use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT as G, edwards::EdwardsPoint as ed25519Point,
    scalar::Scalar as ed25519Scalar, traits::Identity,
};

use rand::Rng;

fn _max_ed25519() -> u256 {
    let two = u256::from(2u32);
    (two << 252) + 27742317777372353535851937790883648493u128
}

// TODO: this is disgusting and must be removed asap
fn G_p() -> ed25519Point {
    monero::util::key::H.point.decompress().unwrap()
}

#[cfg(feature = "experimental")]
use ecdsa_fun::fun::{Point as secp256k1Point, Scalar as secp256k1Scalar, G as H};
#[cfg(feature = "experimental")]
use secp256kfun::{g, s, marker::*};

fn _max_secp256k1() -> u256 {
    // let order_injected: [u8;32] = [
    //     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    //     0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    //     0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b,
    //     0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41
    //     ];

    // n = 2^256 − 432420386565659656852420866394968145599
    //   = 2^256 - 2^128 - 92138019644721193389046258963199934143
    //   = (2^256-1) - (2^128-1) - 92138019644721193389046258963199934143
    let mut n = u256::from_be_bytes([255u8; 32]);
    n -= u128::from_be_bytes([255u8; 16]);
    n -= 92138019644721193389046258963199934143u128;

    // assert_eq!(u256::from_be_bytes(order_injected), n);
    n
}

// Hash to curve of the generator G as explained over here:
// https://crypto.stackexchange.com/a/25603
// Matches the result here:
// https://github.com/mimblewimble/rust-secp256k1-zkp/blob/caa49992ae67f131157f6341f4e8b0b0c1e53055/src/constants.rs#L79-L136
// TODO: this is disgusting and must be removed asap (i.e. change to constant)
fn H_p() -> secp256k1Point {
    let hash_G: [u8; 32] =
        bitcoin_hashes::sha256::Hash::hash(&H.to_bytes_uncompressed()).into_inner();
    let even_y_prepend_hash_G: [u8; 33] = [2u8]
        .iter()
        .chain(hash_G.iter())
        .cloned()
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap();
    secp256k1Point::from_bytes(even_y_prepend_hash_G).expect("Alternate basepoint is invalid")
    // secp256k1Point::from_bytes([2, 80, 146, 155, 116, 193, 160, 73, 84, 183, 139, 75, 96, 53, 233, 122, 94, 7, 138, 90, 15, 40, 236, 150, 213, 71, 191, 238, 154, 206, 128, 58, 192])
    // .expect("Alternate basepoint is invalid")
}

struct PedersenCommitment<Point, Scalar> {
    commitment: Point,
    blinder: Scalar,
}

impl From<(bool, usize)> for PedersenCommitment<ed25519Point, ed25519Scalar> {
    fn from((bit, index): (bool, usize)) -> PedersenCommitment<ed25519Point, ed25519Scalar> {
        let mut csprng = rand_alt::rngs::OsRng;
        let blinder = ed25519Scalar::random(&mut csprng);

        let one: u256 = u256::from(1u32);
        let order = one << index;

        let commitment = match bit {
            false => blinder * G,
            true => G_p() * ed25519Scalar::from_bits(order.to_le_bytes()) + blinder * G,
        };

        PedersenCommitment {
            commitment,
            blinder,
        }
    }
}

impl From<(bool, usize, ed25519Scalar)> for PedersenCommitment<ed25519Point, ed25519Scalar> {
    fn from(
        (bit, index, blinder): (bool, usize, ed25519Scalar),
    ) -> PedersenCommitment<ed25519Point, ed25519Scalar> {
        let one: u256 = u256::from(1u32);
        let order = one << index;

        let commitment = match bit {
            false => blinder * G,
            true => G_p() * ed25519Scalar::from_bits(order.to_le_bytes()) + blinder * G,
        };

        PedersenCommitment {
            commitment,
            blinder,
        }
    }
}

impl From<(bool, usize)> for PedersenCommitment<secp256k1Point, secp256k1Scalar> {
    fn from((bit, index): (bool, usize)) -> PedersenCommitment<secp256k1Point, secp256k1Scalar> {
        let blinder = secp256k1Scalar::random(&mut rand::thread_rng());

        let one: u256 = u256::from(1u32);
        let order = one << index;

        let order_on_curve = secp256k1Scalar::from_bytes(order.to_le_bytes())
            .expect("integer greater than curve order");
        let blinder_point = g!(blinder * H).mark::<NonZero>().unwrap();

        let H_p = H_p();

        let commitment = match bit {
            true => g!(order_on_curve * H_p + blinder_point).mark::<NonZero>().unwrap(),
            false => blinder_point,
        }.mark::<Normal>();

        PedersenCommitment {
            commitment,
            blinder,
        }
    }
}

impl From<(bool, usize, secp256k1Scalar)> for PedersenCommitment<secp256k1Point, secp256k1Scalar> {
    fn from(
        (bit, index, blinder): (bool, usize, secp256k1Scalar),
    ) -> PedersenCommitment<secp256k1Point, secp256k1Scalar> {
        let one: u256 = u256::from(1u32);
        let order = one << index;

        let order_on_curve = secp256k1Scalar::from_bytes(order.to_le_bytes())
            .expect("integer greater than curve order");

        let H_p = H_p();
        let blinder_point = g!(blinder * H);

        let commitment = match bit {
            true => g!(order_on_curve * H_p + blinder_point).mark::<NonZero>().unwrap(),
            false => blinder_point,
        }.mark::<Normal>();

        PedersenCommitment {
            commitment,
            blinder
        }
    }
}

fn key_commitment(
    x: [u8; 32],
    msb_index: usize,
) -> Vec<PedersenCommitment<ed25519Point, ed25519Scalar>> {
    let x_bits = bitvec::prelude::BitSlice::<bitvec::order::Lsb0, u8>::from_slice(&x).unwrap();
    let mut commitment: Vec<PedersenCommitment<ed25519Point, ed25519Scalar>> = x_bits
        .iter()
        .take(msb_index)
        .enumerate()
        .map(|(index, bit)| (*bit, index).into())
        .collect();
    let commitment_last = x_bits.get(msb_index).unwrap();
    let commitment_last_value = match *commitment_last {
        true => ed25519Scalar::one(),
        false => ed25519Scalar::zero(),
    };
    let blinder_last = commitment
        .iter()
        .fold(ed25519Scalar::zero(), |acc, x| acc - x.blinder);
    commitment.push((*commitment_last, msb_index, blinder_last).into());
    commitment
}

fn key_commitment_secp256k1(
    x: [u8; 32],
    msb_index: usize,
) -> Vec<PedersenCommitment<secp256k1Point, secp256k1Scalar>> {
    let x_bits = bitvec::prelude::BitSlice::<bitvec::order::Lsb0, u8>::from_slice(&x).unwrap();
    let mut commitment: Vec<PedersenCommitment<secp256k1Point, secp256k1Scalar>> = x_bits
        .iter()
        .take(msb_index)
        .enumerate()
        .map(|(index, bit)| (*bit, index).into())
        .collect();
    let commitment_last = x_bits.get(msb_index).unwrap();
    let commitment_last_value = match *commitment_last {
        true => secp256k1Scalar::one().mark::<Zero>(),
        false => secp256k1Scalar::zero(),
    };
    let blinder_last = commitment
        .iter()
        .fold(secp256k1Scalar::zero(), |acc, x| s!(acc - x.blinder));
    commitment.push((*commitment_last, msb_index, blinder_last.mark::<NonZero>().expect("is zero")).into());
    commitment
}

struct DLEQProof {
    xg_p: ed25519Point,
    xh_p: secp256k1Point,
    c_g: Vec<PedersenCommitment<ed25519Point, ed25519Scalar>>,
    c_h: Vec<PedersenCommitment<secp256k1Point, secp256k1Scalar>>,
    e_g_0: Vec<ed25519Scalar>,
    e_h_0: Vec<secp256k1Scalar>,
    e_g_1: Vec<ed25519Scalar>,
    e_h_1: Vec<secp256k1Scalar>,
    a_0: Vec<ed25519Scalar>,
    a_1: Vec<secp256k1Scalar>,
    b_0: Vec<ed25519Scalar>,
    b_1: Vec<secp256k1Scalar>,
}

impl DLEQProof {
    fn generate(x: [u8; 32]) -> Self {
        let highest_bit = 255;

        let x_ed25519 = ed25519Scalar::from_bytes_mod_order(x);
        let xg_p = x_ed25519 * G_p();

        // TODO: do properly
        let mut x_secp256k1: secp256k1Scalar<_> = secp256k1Scalar::from_bytes(x)
            .unwrap()
            .mark::<NonZero>()
            .expect("x is zero");
        let xh_p = secp256k1Point::from_scalar_mul(H, &mut x_secp256k1).mark::<Normal>();


        DLEQProof {
            xg_p,
            xh_p,
            c_g: key_commitment(x, highest_bit),
            c_h: key_commitment_secp256k1(x, highest_bit),
            e_g_0: vec![ed25519Scalar::default()],
            e_h_0: vec![secp256k1Scalar::random(&mut rand::thread_rng())],
            e_g_1: vec![ed25519Scalar::default()],
            e_h_1: vec![secp256k1Scalar::random(&mut rand::thread_rng())],
            a_0: vec![ed25519Scalar::default()],
            a_1: vec![secp256k1Scalar::random(&mut rand::thread_rng())],
            b_0: vec![ed25519Scalar::default()],
            b_1: vec![secp256k1Scalar::random(&mut rand::thread_rng())],
        }
    }
}

#[test]
fn pedersen_commitment_works() {
    let mut x: [u8; 32] = rand::thread_rng().gen();
    // ensure 256th bit is 0
    x[31] &= 0b0111_1111;
    let key_commitment = key_commitment(x, 255);
    let commitment_acc = key_commitment
        .iter()
        .fold(ed25519Point::identity(), |acc, bit_commitment| {
            acc + bit_commitment.commitment
        });
    assert_eq!(
        ed25519Scalar::from_bytes_mod_order(x) * G_p(),
        commitment_acc
    );
}

#[test]
fn pedersen_commitment_sec256k1_works() {
    let x: [u8; 32] = rand::thread_rng().gen();
    // let mut x: [u8; 32] = rand::thread_rng().gen();
    // ensure 256th bit is 0
    // x[31] &= 0b0111_1111;
    let key_commitment = key_commitment_secp256k1(x, 255);
    // let commitment_acc: secp256k1Point<Jacobian, Public, Zero> = key_commitment
    let commitment_acc = key_commitment
        .iter()
        .fold(secp256k1Point::zero(), |acc, bit_commitment| g!(acc + bit_commitment.commitment).mark::<Normal>()
        // .fold(secp256k1Point::zero().mark::<Jacobian>(), |acc, bit_commitment| g!(acc + bit_commitment.commitment)
);
    let x_secp256k1 = secp256k1Scalar::from_bytes_mod_order(x);
    let H_p = H_p();
    assert_eq!(
        g!(x_secp256k1 * H_p),
        commitment_acc
    );
}

#[test]
fn blinders_sum_to_zero() {
    let x: [u8; 32] = rand::thread_rng().gen();
    let key_commitment = key_commitment(x, 255);
    let blinder_acc = key_commitment
        .iter()
        .fold(ed25519Scalar::zero(), |acc, bit_commitment| {
            acc + bit_commitment.blinder
        });
    assert_eq!(blinder_acc, ed25519Scalar::zero());
}
