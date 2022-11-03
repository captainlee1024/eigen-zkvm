use crate::poseidon_bn128::{Fr, Poseidon};
use crate::ElementDigest;
use ff::*;
use winter_crypto::{Digest, Hasher};
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, StarkField};

use crate::errors::Result;

pub struct LinearHashBN128 {
    h: Poseidon,
}

use crate::constant::*;

impl LinearHashBN128 {
    pub fn new() -> Self {
        LinearHashBN128 { h: Poseidon::new() }
    }

    /// used for hash leaves only, converting element from GL to BN128
    pub fn hash_element_matrix(&self, columns: &Vec<Vec<BaseElement>>) -> Result<Fr> {
        let mut st = Fr::zero();
        let mut vals3: Vec<Fr> = vec![];

        let mut acc = Fr::zero();
        let mut accN = 0;

        for col in columns.iter() {
            for elem in col.iter() {
                // NOTE: BaseElement to Fr
                let mut e = Fr::from_str(&elem.as_int().to_string()).unwrap();
                if accN == 1 {
                    e.mul_assign(&OFFSET_2_64);
                } else if accN == 2 {
                    e.mul_assign(&OFFSET_2_128);
                }
                acc.add_assign(&e);
                accN += 1;
                if accN == 3 {
                    vals3.push(acc);
                    acc = Fr::zero();
                    accN = 0;
                }
            }
        }
        if accN > 0 {
            vals3.push(acc);
        }
        if vals3.len() == 0 {
            return Ok(st);
        } else if vals3.len() == 1 {
            return Ok(vals3[0]);
        }
        let mut inHash: Vec<Fr> = vec![];

        for val3 in vals3.iter() {
            inHash.push(val3.clone());
            if inHash.len() == 16 {
                st = self.h.hash(&inHash, &st)?;
                inHash = vec![];
            }
        }
        if inHash.len() > 0 {
            st = self.h.hash(&inHash, &st)?;
        }
        Ok(st)
    }
}

/// hasher element over BN128
impl Hasher for LinearHashBN128 {
    type Digest = ElementDigest;

    fn hash(bytes: &[u8]) -> Self::Digest {
        let hasher = LinearHashBN128::new();
        let elems: &[BaseElement] = unsafe { BaseElement::bytes_as_elements(bytes).unwrap() };
        let digest = hasher.hash_element_matrix(&vec![elems.to_vec()]).unwrap();
        Self::Digest::from(&digest)
    }

    /// Returns a hash of two digests. This method is intended for use in construction of
    /// Merkle trees.
    fn merge(values: &[Self::Digest; 2]) -> Self::Digest {
        let hasher = Poseidon::new();
        let inp = vec![values[0].into(), values[1].into()];
        let init_state = Fr::zero();
        Self::Digest::from(&hasher.hash(&inp, &init_state).unwrap())
    }

    /// Returns hash(`seed` || `value`). This method is intended for use in PRNG and PoW contexts.
    fn merge_with_int(seed: Self::Digest, value: u64) -> Self::Digest {
        panic!("Unimplemented method");
        ElementDigest::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::linearhash_bn128::LinearHashBN128;
    use crate::poseidon_bn128::{Fr, Poseidon};
    use ff::*;
    use winter_math::fields::f64::BaseElement;
    use winter_math::StarkField;

    #[test]
    fn test_linearhashBN128() {
        let inputs: Vec<_> = (0..100).collect::<Vec<u64>>();
        let inputs: Vec<Vec<BaseElement>> = inputs
            .iter()
            .map(|e: &u64| {
                vec![
                    BaseElement::from(e.clone()),
                    BaseElement::from(e * 1000),
                    BaseElement::from(e * 1000000),
                ]
            })
            .collect();

        let st = LinearHashBN128::new().hash_element_matrix(&inputs).unwrap();
        assert_eq!(
            st.to_string(),
            "Fr(0x29c2ac38b7b8d18b9c1b575369cb4ab930ef71ebd5e4631b3916360233a29cae)",
        );
    }
}