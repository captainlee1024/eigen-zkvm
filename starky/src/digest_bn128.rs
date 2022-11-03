use crate::poseidon_bn128::Fr;
use core::slice;
use ff::*;
use winter_crypto::Digest;
use winter_math::StarkField;
use winter_math::{fields::f64::BaseElement, FieldElement};
use winter_utils::{ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable};

use num_bigint::BigUint;
use num_traits::Num;
use num_traits::ToPrimitive;

use std::ops::{AddAssign, MulAssign};

const DIGEST_SIZE: usize = 4;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ElementDigest([BaseElement; DIGEST_SIZE]);

impl ElementDigest {
    pub fn new(value: [BaseElement; DIGEST_SIZE]) -> Self {
        Self(value)
    }

    pub fn as_elements(&self) -> &[BaseElement] {
        &self.0
    }

    pub fn digests_as_elements(digests: &[Self]) -> &[BaseElement] {
        let p = digests.as_ptr();
        let len = digests.len() * DIGEST_SIZE;
        unsafe { slice::from_raw_parts(p as *const BaseElement, len) }
    }
}

/// Field mapping
/// Fr always consists of [u64; limbs], here for bn128, the limbs is 4.
impl From<&Fr> for ElementDigest {
    fn from(e: &Fr) -> Self {
        let mut result = [BaseElement::ZERO; DIGEST_SIZE];
        result[0] = BaseElement::from(e.0 .0[0]);
        result[1] = BaseElement::from(e.0 .0[1]);
        result[2] = BaseElement::from(e.0 .0[2]);
        result[3] = BaseElement::from(e.0 .0[3]);
        ElementDigest::new(result)
    }
}

impl Into<Fr> for ElementDigest {
    fn into(self) -> Fr {
        let mut result = Fr::zero();
        result.0 .0[0] = self.0[0].as_int().into();
        result.0 .0[1] = self.0[1].as_int().into();
        result.0 .0[2] = self.0[2].as_int().into();
        result.0 .0[3] = self.0[3].as_int().into();
        result
    }
}

impl crate::traits::FieldMapping for ElementDigest {
    fn to_BN128(e: &[BaseElement; 4]) -> Fr {
        let mut result = BigUint::from(e[0].as_int());

        let mut added = BigUint::from(e[1].as_int());
        added = added << 64;
        result += added;

        let mut added = BigUint::from(e[2].as_int());
        added = added << 128;
        result += added;

        let mut added = BigUint::from(e[3].as_int());
        added = added << 192;
        result += added;

        Fr::from_str(&result.to_string()).unwrap()
    }

    /// by js:
    /// const r = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;
    /// const n64 = Math.floor((bitLength(r - 1n) - 1)/64) +1;
    /// const f1size = n64*8;
    /// return BigInt(a) * ( 1n << BigInt(f1size*8)) % r;
    fn to_montgomery(e: &Fr) -> Fr {
        // opt: precompute
        let _2_256 = BigUint::from(1u32) << 256;
        let ee: BigUint =
            BigUint::from_str_radix(&(to_hex(e).trim_start_matches('0')), 16).unwrap();
        let r = BigUint::from_str_radix(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617",
            10,
        )
        .unwrap();

        let ee: BigUint = (&_2_256 * ee) % r;
        Fr::from_str(&ee.to_string()).unwrap()
    }

    fn to_GL(f: &Fr) -> [BaseElement; 4] {
        let mut f = BigUint::from_str_radix(&(to_hex(f).trim_start_matches('0')), 16).unwrap();
        let mask = BigUint::from_str_radix("ffffffffffffffff", 16).unwrap();

        let mut result = [BaseElement::ZERO; 4];

        for i in 0..4 {
            let t = &f & &mask;
            result[i] = BaseElement::from(t.to_u64().unwrap());
            f = &f >> 64;
        }

        result
    }
}

impl Digest for ElementDigest {
    fn as_bytes(&self) -> [u8; 32] {
        let mut result = [0; 32];
        result[..8].copy_from_slice(&self.0[0].as_int().to_le_bytes());
        result[8..16].copy_from_slice(&self.0[1].as_int().to_le_bytes());
        result[16..24].copy_from_slice(&self.0[2].as_int().to_le_bytes());
        result[24..].copy_from_slice(&self.0[3].as_int().to_le_bytes());

        result
    }
}

impl Default for ElementDigest {
    fn default() -> Self {
        ElementDigest([BaseElement::default(); DIGEST_SIZE])
    }
}

impl Serializable for ElementDigest {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write_u8_slice(&self.as_bytes());
    }
}

impl Deserializable for ElementDigest {
    fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
        let e1 = BaseElement::new(source.read_u64()?);
        let e2 = BaseElement::new(source.read_u64()?);
        let e3 = BaseElement::new(source.read_u64()?);
        let e4 = BaseElement::new(source.read_u64()?);
        // TODO: check if the field elements are valid?

        Ok(Self([e1, e2, e3, e4]))
    }
}

impl From<[BaseElement; DIGEST_SIZE]> for ElementDigest {
    fn from(value: [BaseElement; DIGEST_SIZE]) -> Self {
        Self(value)
    }
}

impl From<ElementDigest> for [BaseElement; DIGEST_SIZE] {
    fn from(value: ElementDigest) -> Self {
        value.0
    }
}

impl From<ElementDigest> for [u8; 32] {
    fn from(value: ElementDigest) -> Self {
        value.as_bytes()
    }
}

#[cfg(test)]
pub mod tests {
    use crate::digest_bn128::ElementDigest;
    use crate::poseidon_bn128::Fr;
    use crate::traits::FieldMapping;
    use ff::PrimeField;
    use rand_utils::rand_vector;
    use winter_math::fields::f64::BaseElement;
    use winter_math::StarkField;

    #[test]
    fn test_fr_to_element_digest_and_versus() {
        let b4 = rand_vector::<BaseElement>(4);
        let b4 = ElementDigest::new(b4.try_into().unwrap());
        let f1: Fr = b4.into();

        let b4_: ElementDigest = ElementDigest::from(&f1);
        assert_eq!(b4, b4_);

        let f: Fr = Fr::from_str(
            "21888242871839275222246405745257275088548364400416034343698204186575808495616", // Fr::MODULE - 1
        )
        .unwrap();

        let e = ElementDigest::from(&f);
        let f2: Fr = e.into();
        assert_eq!(f, f2);
    }

    #[test]
    fn test_fr_to_mont_to_element_digest_and_versus() {
        let b4: Vec<BaseElement> = vec![3u32, 1003, 2003, 0]
            .iter()
            .map(|e| BaseElement::from(e.clone()))
            .collect();
        let mut f1: Fr = ElementDigest::to_BN128(&b4[..].try_into().unwrap());
        println!("f111 {:?}", f1.to_string());

        // to Montgomery
        let f1 = ElementDigest::to_montgomery(&f1);
        println!("f111 {:?}", f1.to_string());

        let e1 = ElementDigest::to_GL(&f1);
        let expected: [BaseElement; 4] = vec![
            10593660675180540444u64,
            2538813791642109216,
            4942736554053463004,
            3183287946373923876,
        ]
        .iter()
        .map(|e| BaseElement::from(e.clone()))
        .collect::<Vec<BaseElement>>()
        .try_into()
        .unwrap();
        assert_eq!(expected, e1);
    }
}