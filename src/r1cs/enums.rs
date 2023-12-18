#![allow(non_snake_case)]
#![allow(dead_code)]

use super::util::*;
use curve25519_dalek::scalar::Scalar;
use std::collections::HashMap;

use std::fmt;

#[derive(Clone, Debug)]
pub struct MatCheckError;

impl fmt::Display for MatCheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "matrice mult dont hold")
   }
}

/// enum for either matrix or vector return values
pub enum MatorVec {
    Vector(Vec<Scalar>),
    Matrix(Vec<Vec<Scalar>>),
}

/// bin for keeping
///     Vectors:
///         - Multiplication Gate Left Inputs: aL
///         - Multiplication Gate Right Inputs: aR
///         - Multiplication Gate Outputs: aO
///         - Constants: c
///         - Variables: v
///     Matrices:
///         - Left Input Weights for Verification: wL
///         - Right Input Weights for Verification: wR
///         - Output Weights for Verification: wO
///         - Variable Weights for Verification: wV
/// needed for verification
#[derive(Clone, Default)]
pub struct VarVecs {
    pub vectors: HashMap<String, Vec<Scalar>>,
    pub matrices: HashMap<String, Vec<Vec<Scalar>>>,
}

impl VarVecs {
    
    ///Return the state of the current bin as String
    pub fn print(&self) -> String {
        let mut res = String::default();

        for (k, v) in self.vectors.iter() {
            res.push_str(&(k.clone() + ": " + &print_scalar_vec(v) + "\n"))
        }

        for (k, v) in self.matrices.iter() {
            res.push_str(&(k.clone() + ":\n" + &print_scalar_mat(v) + "\n"))
        }
        res
    }

    ///Return the corresponding index(key) value as MatorVec
    pub fn index(&self, index: &str) -> MatorVec {
        match index.starts_with("w") {
            true => MatorVec::Matrix(self.matrices[index].clone()),
            false => MatorVec::Vector(self.vectors[index].clone()),
        }
    }

    ///Add MatorVec to the corresponding index
    pub fn add(&mut self, index: &str, val: MatorVec) {
        let _ = match val {
            MatorVec::Matrix(i) => {self.matrices.insert(index.into(), i.clone()); ()}
            MatorVec::Vector(i) => {self.vectors.insert(index.into(), i.clone()); ()}
        };
    }

    ///Constructor that can take lists of vectors and matrices
    /// It needs an order for the addition:
    ///     Vectors: [aL, aR, aO, c, v], empty values are ok
    ///     Vectors: [wL, wR, wO, wV], empty values are ok
    pub fn new(vectors: &[Vec<Scalar>], matrices: &[Vec<Vec<Scalar>>]) -> Self {
        let mut vecs: HashMap<String, Vec<Scalar>> = HashMap::new();
        let mut mats: HashMap<String, Vec<Vec<Scalar>>> = HashMap::new();

        assert!(vectors.len() <= 5, "aL, aR, aO, c, v are the only 4 vectors");
        assert!(matrices.len() <= 4, "wL, wR, wO, wV are the only 4 vectors");

        for (i, v) in vectors.iter()
                            .enumerate() {
                match i {
                    0 => vecs.insert("aL".into(), v.clone()),
                    1 => vecs.insert("aR".into(), v.clone()),
                    2 => vecs.insert("aO".into(), v.clone()),
                    3 => vecs.insert("c".into(), v.clone()),
                    4 => vecs.insert("v".into(), v.clone()),
                    _ => None
                };
            }

        for (i, v) in matrices.iter()
                            .enumerate() {
                match i {
                    0 => mats.insert("wL".into(), v.clone()),
                    1 => mats.insert("wR".into(), v.clone()),
                    2 => mats.insert("wO".into(), v.clone()),
                    3 => mats.insert("wV".into(), v.clone()),
                    _ => None
                };
            }

        VarVecs{vectors: vecs, matrices: mats}
    }

    ///Verify the equation:
    /// wL*aL + wR*aR - wO*aO = wV*v + c
    ///Return () if holds
    ///       MatCheckError if not
    pub fn verify(&self) -> Result<(), MatCheckError>{
        let aL: Vec<Scalar> = self.vectors["aL"].clone();
        let aR: Vec<Scalar> = self.vectors["aR"].clone();
        let aO: Vec<Scalar> = self.vectors["aO"].clone();
        let v: Vec<Scalar> = self.vectors["v"].clone();
        let c: Vec<Scalar> = self.vectors["c"].clone();

        let wL: Vec<Vec<Scalar>> = self.matrices["wL"].clone();
        let wR: Vec<Vec<Scalar>> = self.matrices["wR"].clone();
        let wO: Vec<Vec<Scalar>> = self.matrices["wO"].clone();
        let wV: Vec<Vec<Scalar>> = self.matrices["wV"].clone();

        let L = mv_mult(&wL, &aL);
        let R = mv_mult(&wR, &aR);
        let O = mv_mult(&wO, &aO);
        let V = mv_mult(&wV, &v);

        println!("{}", self.print());

        println!("multiplication term L: {}", print_scalar_vec(&L));
        println!("multiplication term R: {}", print_scalar_vec(&R));
        println!("multiplication term O: {}", print_scalar_vec(&O));
        println!("multiplication term V: {}", print_scalar_vec(&V));
        println!("constant term c: {}", print_scalar_vec(&c));

        let left_side: Vec<Scalar> = L.iter()
            .zip(R.iter()
                 .zip(O.iter()))
            .map(|(l, (r, o))| l + r - o)
            .collect();
        let right_side: Vec<Scalar> = V.iter()
            .zip(c.iter())
            .map(|(v_, c_)| v_ + c_)
            .collect();

        println!("left hand side  = {}", print_scalar_vec(&left_side));
        println!("right hand side = {}", print_scalar_vec(&right_side));
        let results: Vec<bool> = left_side.iter()
            .zip(right_side.iter())
            .map(|(l, r)| l == r)
            .collect();

        if results.iter().all(|x| *x) {
            Ok(())
        } else {
            Err(MatCheckError)
        }
        
    }


}
