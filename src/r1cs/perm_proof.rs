#![allow(non_snake_case)]

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

pub fn print_scalar_mat(m: &Vec<Vec<Scalar>>) -> String {
    let mut result: String = String::from("[");
    for v in m {
        result.push_str(print_scalar_vec(&v).as_str());
        result.push_str(",\n");
    }
    result.push_str("]");

    println!("{}", result);
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

        let (last_l, last_r, last_o) = cs.multiply(x[k-1] - *c, x[k-2] - *c);
        let first_o = (0..k-2).rev().fold(last_o, |prev_o, i| {
            let (l, r, o) = cs.multiply(prev_o.into(), x[i] - *c);
            o
        });

        let (last_l_, last_r_, last_o_) = cs.multiply(x_[k-1] - *c, x_[k-2] - *c);
        let first_o_ = (0..k-2).rev().fold(last_o_, |prev_o, i| {
            let (l, r, o) = cs.multiply(prev_o.into(), x_[i] - *c);
            o
        });

        cs.constrain(first_o - first_o_);

        Ok(())
    }

    pub fn prove<'a, 'b>(
        pc_gens: &'b PedersenGens,
        bp_gens: &'b BulletproofGens,
        transcript: &'a mut Transcript,
        input: &[Scalar],
        output:&[Scalar],
        chall: &Scalar,
    ) -> Result<(PermProof, Vec<CompressedRistretto>, Vec<CompressedRistretto>), R1CSError> {
        let k = input.len();
        transcript.commit_bytes(b"dom-sep", b"PermProof");
        transcript.commit_bytes(b"k", Scalar::from(k as u64).as_bytes());

        let mut prover = Prover::new(&pc_gens, transcript);

        let mut blinding_rng = rand::thread_rng();

        let (input_commits, input_vars): (Vec<_>, Vec<_>) = input.into_iter()
            .map(|v| 
                 prover.commit(*v, Scalar::random(&mut blinding_rng))
            )
            .unzip();

        let (output_commits, output_vars): (Vec<_>, Vec<_>) = output.into_iter()
            .map(|v| 
                 prover.commit(*v, Scalar::random(&mut blinding_rng))
            )
            .unzip();

        PermProof::create_constraints(&mut prover, input_vars, output_vars, chall)?;

        let proof = prover.prove(&bp_gens)?;

        Ok((PermProof(proof), input_commits, output_commits))
                                          
    }

    pub fn verify <'a, 'b>(
        &self,
        pc_gens: &'b PedersenGens,
        bp_gens: &'b BulletproofGens,
        transcript: &'a mut Transcript,
        input_commits: &Vec<CompressedRistretto>,
        output_commits: &Vec<CompressedRistretto>,
        chall: &Scalar,
    ) -> Result<(), R1CSError> {
        let k = input_commits.len();
        transcript.commit_bytes(b"dom-sep", b"PermProof");
        transcript.commit_bytes(b"k", Scalar::from(k as u64).as_bytes());

        let mut verifier = Verifier::new(transcript);

        let input_vars: Vec<_> = input_commits.iter()
            .map(|commit| verifier.commit(*commit))
            .collect();

        let output_vars: Vec<_> = output_commits.iter()
            .map(|commit| verifier.commit(*commit))
            .collect();

        PermProof::create_constraints(&mut verifier, input_vars, output_vars, chall)?;

        //let (wL, wR, wO, wV, wc) = verifier.flattened_constraints(chall);
        let (wL, wR, wO, wV, wc) = verifier.get_weights();

        print_scalar_mat(&wL);
        print_scalar_mat(&wR);
        print_scalar_mat(&wO);
        print_scalar_mat(&wV);
        println!("{}", print_scalar_vec(&wc));

        verifier.verify(&self.0, &pc_gens, &bp_gens)

    }
}

fn perm_basic_test() {
    // Construct generators. 1024 Bulletproofs generators is enough for 512-size shuffles.
    let k: usize = 4;
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new((2 * k).next_power_of_two(), 1);

    // Putting the prover code in its own scope means we can't
    // accidentally reuse prover data in the test.
    let c: Scalar = Scalar::from(1u64);
    let (proof, in_commitments, out_commitments) = {
        let inputs = [
            Scalar::from(0u64),
            Scalar::from(1u64),
            Scalar::from(2u64),
            Scalar::from(3u64),
        ];
        let outputs = [
            Scalar::from(2u64),
            Scalar::from(0u64),
            Scalar::from(3u64),
            Scalar::from(1u64),
        ];

        let mut prover_transcript = Transcript::new(b"PermProofTest");
        PermProof::prove(
            &pc_gens,
            &bp_gens,
            &mut prover_transcript,
            &inputs,
            &outputs,
            &c
        )
        .expect("error during proving")
    };

    let mut verifier_transcript = Transcript::new(b"PermProofTest");
    assert!(
        proof
            .verify(&pc_gens, &bp_gens, &mut verifier_transcript, &in_commitments, &out_commitments, &c)
            .is_ok()
    );
}

fn test_helper(k: usize) {
    use rand::Rng;

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new((2 * k).next_power_of_two(), 1);

    let challenge_scalar: Scalar = Scalar::from(2u64);
    let (proof, input_commits, output_commits) = {
        let mut rng = rand::thread_rng();
        let (min, max) = (0u64, 200u64);

        let input: Vec<Scalar> = (0..k)
            .map(|_| Scalar::from(rng.gen_range(min, max)))
            .collect();
        let mut output = input.clone();
        output.shuffle(&mut rand::thread_rng());

        let mut prover_transcript = Transcript::new(b"PermProofTest");
        PermProof::prove(&pc_gens, 
             &bp_gens, 
             &mut prover_transcript, 
             &input, 
             &output,
             &challenge_scalar)
        .unwrap()
    };

    {
        let mut verifier_transcript = Transcript::new(b"PermProofTest");
        assert!(proof.verify(
                &pc_gens,
                &bp_gens,
                &mut verifier_transcript,
                &input_commits,
                &output_commits,
                &challenge_scalar
                ).is_ok());
    }
}
#[test]
fn perm_test_1() {
    test_helper(4);
}
