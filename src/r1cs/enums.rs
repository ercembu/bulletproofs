use curve25519_dalek::scalar::Scalar;
use std::collections::HashMap;

use std::fmt;
use ethnum::{I256};

pub enum MatorVec {
    Vector(Vec<Scalar>),
    Matrix(Vec<Vec<Scalar>>),
}
#[derive(Clone, Default)]
pub(crate) struct VarVecs {
    pub vectors: HashMap<String, Vec<Scalar>>,
    pub matrices: HashMap<String, Vec<Vec<Scalar>>>,
}

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

    result

}
impl fmt::Debug for VarVecs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = String::default();

        println!("{:?}", self.vectors);
        let _ = self.vectors.iter().map(|(k, v)| {
            res.push_str(&(k.clone() + "\n" + &print_scalar_vec(v)))
        });

        let _ = self.matrices.iter().map(|(k, v)| {
            res.push_str(&(k.clone() + "\n" + &print_scalar_mat(v)))
        });
        write!(f, "{}", res)
    }
}


impl VarVecs {
    pub fn index(&self, index: &str) -> MatorVec {
        match index.starts_with("w") {
            true => MatorVec::Matrix(self.matrices[index].clone()),
            false => MatorVec::Vector(self.vectors[index].clone()),
        }
    }
    pub fn add(&mut self, index: &str, val: MatorVec) {
        //TODO: doesnt save states
        match val {
            MatorVec::Matrix(i) => {self.matrices.insert(index.into(), i.clone()); ()}
            MatorVec::Vector(i) => {self.vectors.insert(index.into(), i.clone()); ()}
        };
    }
    pub fn new(vectors: &[Vec<Scalar>], matrices: &[Vec<Vec<Scalar>>]) -> Self {
        //TODO: doesnt save states
        let mut res: VarVecs = VarVecs::default();
        assert!(vectors.len() <= 4, "aL, aR, aO, c are the only 4 vectors");
        assert!(matrices.len() <= 4, "wL, wR, wO, wV are the only 4 vectors");
        let _ = vectors.iter()
            .enumerate()
            .map(|(i, v)| {
                match i {
                    0 => res.vectors.insert("aL".into(), v.clone()),
                    1 => res.vectors.insert("aR".into(), v.clone()),
                    2 => res.vectors.insert("aO".into(), v.clone()),
                    3 => res.vectors.insert("c".into(), v.clone()),
                    _ => None
                };
            });

        let _ = matrices.iter()
            .enumerate()
            .map(|(i, v)| {
                match i {
                    0 => res.matrices.insert("wL".into(), v.clone()),
                    1 => res.matrices.insert("wR".into(), v.clone()),
                    2 => res.matrices.insert("wO".into(), v.clone()),
                    3 => res.matrices.insert("wV".into(), v.clone()),
                    _ => None
                };
            });
        res
    }


}
