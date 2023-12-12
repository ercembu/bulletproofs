#![allow(non_snake_case)]

extern crate curve25519_dalek;
extern crate merlin;
extern crate rand;

use super::*;
use crate::{BulletproofGens, PedersenGens};
use crate::r1cs::enums::*;
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
    fn create_var_vecs(
        x: &[Scalar],
        x_: &[Scalar],
        c: &Scalar
    ) -> (Vec<Scalar>, Vec<Scalar>, Vec<Scalar>) {
        let n = x.len() * 2;
        let mut a_L: Vec<Scalar> = vec![Scalar::zero(); n];//Vec::new();
        let mut a_R: Vec<Scalar> = vec![Scalar::zero(); n];
        let mut a_O: Vec<Scalar> = vec![Scalar::zero(); n];

        println!("{}", print_scalar_vec(&x.to_vec()));
        println!("{}", print_scalar_vec(&x_.to_vec()));


        let offset = (n-1)/2;

        for i in 0..x.len() - 1 {

            a_R[i] = x[i+1] - c;
            a_R[i+offset] = x_[i+1] - c;

            if i == 0 {
                a_L[i] = x[i] - c;
                a_L[i + offset] = x_[i] - c;

            } else {
                a_L[i] = a_O[i - 1];
                a_L[i + offset] = a_O[i + offset - 1];

            }


            a_O[i] = a_L[i] * a_R[i];
            a_O[i + offset] = a_L[i + offset] * a_R[i + offset];



        }

        a_L[n-2] = a_O[n-3];
        a_R[n-2] = -Scalar::one().reduce();
        a_O[n-2] = a_L[n-2] * a_R[n-2];

        a_L[n-1] = a_O[offset-1] + a_O[n-2];
        a_R[n-1] = Scalar::one();
        a_O[n-1] = a_L[n-1] * a_L[n-1];
        

        (a_L, a_R, a_O)
        
    }
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

        let (_, _, mut curr_out) = cs.multiply(x[0] - *c, x[1] - *c);
        for i in 2..k {
            (_, _, curr_out) = cs.multiply(curr_out.into(), x[i] - *c);
        }
        let (_, _, mut curr_out_) = cs.multiply(x_[0] - *c, x_[1] - *c);
        for i in 2..k {
            (_, _, curr_out_) = cs.multiply(curr_out_.into(), x_[i] - *c);
        }

        (_, _, curr_out_) = cs.multiply(curr_out_.into(), (-Scalar::one()).into());


        let (_, _, last_out) = cs.multiply(curr_out + curr_out_, Scalar::one().into());

        cs.constrain(last_out.into());

        Ok(())
    }

    pub fn prove<'a, 'b>(
        pc_gens: &'b PedersenGens,
        bp_gens: &'b BulletproofGens,
        transcript: &'a mut Transcript,
        input: &[Scalar],
        output:&[Scalar],
        chall: &Scalar,
    ) -> Result<(PermProof, Vec<CompressedRistretto>, Vec<CompressedRistretto>, VarVecs), R1CSError> {
        let k = input.len();
        transcript.commit_bytes(b"dom-sep", b"PermProof");
        transcript.commit_bytes(b"k", Scalar::from(k as u64).as_bytes());

        let (aL, aR, aO): 
            (Vec<_>, Vec<_>, Vec<_>) = 
             PermProof::create_var_vecs(input,
                                        output,
                                        chall);

        let vecs: [Vec<Scalar>; 3] = [aL, aR, aO];
        let mats: [Vec<Vec<Scalar>>; 0] = [];
        let mut spaces: VarVecs = VarVecs::new(&vecs, &mats);

        spaces.add("v", MatorVec::Vector([input.clone(), output.clone(), &[chall.clone()]].concat()));

        println!("before: {}", spaces.print());

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

        Ok((PermProof(proof), input_commits, output_commits, spaces))
                                          
    }

    pub fn verify <'a, 'b>(
        &self,
        pc_gens: &'b PedersenGens,
        bp_gens: &'b BulletproofGens,
        transcript: &'a mut Transcript,
        input_commits: &Vec<CompressedRistretto>,
        output_commits: &Vec<CompressedRistretto>,
        chall: &Scalar,
        vec_bin: &mut VarVecs,
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
        vec_bin.add("wL", MatorVec::Matrix(wL));
        vec_bin.add("wR", MatorVec::Matrix(wR));
        vec_bin.add("wO", MatorVec::Matrix(wO));
        vec_bin.add("c", MatorVec::Vector(wc));
        vec_bin.add("wV", MatorVec::Matrix(wV));

        println!("after: {}", vec_bin.print());

        assert!(vec_bin.verify().is_ok());

        verifier.verify(&self.0, &pc_gens, &bp_gens)

    }
}

#[test]
fn perm_basic_test() {
    // Construct generators. 1024 Bulletproofs generators is enough for 512-size shuffles.
    let k: usize = 4;
    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new((2 * k).next_power_of_two(), 1);

    // Putting the prover code in its own scope means we can't
    // accidentally reuse prover data in the test.
    let c: Scalar = Scalar::from(3u64);
    let (proof, in_commitments, out_commitments, mut spaces) = {
        let inputs = [
            Scalar::from(1u64),
            Scalar::from(2u64),
            Scalar::from(4u64),
            Scalar::from(0u64),
        ];
        let outputs = [
            Scalar::from(1u64),
            Scalar::from(0u64),
            Scalar::from(2u64),
            Scalar::from(4u64),
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
            .verify(&pc_gens, &bp_gens, &mut verifier_transcript, &in_commitments, &out_commitments, &c, &mut spaces)
            .is_ok()
    );
}

fn test_helper(k: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let pc_gens = PedersenGens::default();
    let bp_gens = BulletproofGens::new((2 * k).next_power_of_two(), 1);

    let challenge_scalar: Scalar = Scalar::random(&mut rng);
    let (proof, input_commits, output_commits, mut spaces) = {
        let (min, max) = (0u64, 5u64);

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
                &challenge_scalar,
                &mut spaces,
                ).is_ok());
    }
}
fn perm_test_1() {
    test_helper(8 as usize);
}
