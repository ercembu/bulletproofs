#![allow(non_snake_case)]

extern crate bulletproofs;
extern crate curve25519_dalek;
extern crate merlin;
extern crate rand;

use super::*;
use crate::{BulletproofGens, PedersenGens};
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand::seq::SliceRandom;
use rand::thread_rng;
use ethnum::{I256};

pub fn print_scalar_vec(v: &Vec<Scalar>) -> String {
    let mut result: String = String::from("[");
    for scalar in v {
        let mut str_result: String;
        let mut sc_str = I256::from_le_bytes(*scalar.as_bytes());
        if sc_str.to_string().len() > 10 { 
            let m_one = I256::from_le_bytes((-Scalar::one().reduce()).to_bytes());
            str_result = (sc_str - (m_one + 1)).to_string();
        } else {str_result = sc_str.to_string();}
        result += &str_result;
        result.push_str(", ");
    }
    result.push_str("]");

    result
}

struct PermProof(R1CSProof);

impl PermProof {
    fn create_constraints<CS: ConstraintSystem>(
        cs: &mut CS,
        x: Vec<Variable>,
        x_: Vec<Variable>,
        c: &Scalar
    ) -> Result<(), R1CSError> {
        assert_eq!(x.len(), x_.len());
        let k = x.len();

        if k == 1 {
            cs.constrain(x_[0] - x[0]);
            return Ok(());
        }

        let mut rng = rand::thread_rng();

        let (last_l, last_r, last_o) = cs.multiply(x[k-1] - c, x[k-2] - c);
        let first_o = (0..k-2).rev().fold(last_o, |prev_o, i| {
            let (l, r, o) = cs.multiply(prev_out.into(), x[i] - c);
            o
        });

        let (last_l_, last_r_, last_o_) = cs.multiply(x_[k-1] - c, x_[k-2] - c);
        let first_o_ = (0..k-2).rev().fold(last_o_, |prev_o, i| {
            let (l, r, o) = cs.multiply(prev_out.into(), x_[i] - c);
            o
        });

        cs.constrain(first_o - first_o_);

        Ok(())
    }

}
