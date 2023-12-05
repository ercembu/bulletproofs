use curve25519_dalek::scalar::Scalar;
use ethnum::I256;

///Util functions

pub fn hadamard_V(a: &Vec<Scalar>, b: &Vec<Scalar>) -> Vec<Scalar> {
    let a_len = a.len();

    if a_len != b.len() {
        panic!("hadamard_V(a, b): {} and {} should have same size", a_len, b.len());
    }

    let mut out: Vec<Scalar> = (0..a.len()).map(|_| Scalar::one()).collect();

    for i in 0..a_len {
        out[i] *= a[i] * b[i];
    }

    out
}

pub fn vm_mult(a: &Vec<Scalar>, b: &Vec<Vec<Scalar>>) -> Vec<Scalar> {
    let a_len = a.len();
    let b_len = b[0].len();

    if a_len != b_len {
        panic!("vm_mult(a,b): a -> 1x{}, b -> {}x{} needs to be", a_len, b_len, b.len());
    }

    let mut out: Vec<Scalar> = (0..b.len()).map(|_| Scalar::zero()).collect();
    
    for i in 0..b.len() {
        let col: Vec<Scalar> = (0..a_len).map(|j| b[i][j]).collect();
        out[i] += inner_product(&a, &col);
    }

    out
}

pub fn mv_mult(a: &Vec<Vec<Scalar>>, b: &Vec<Scalar>) -> Vec<Scalar> {
    let b_len = b.len();
    let a_len = a[0].len();

    if a_len != b_len {
        panic!("mv_mult(a,b): a->{}x{}, b->{}x1 needs to be", a.len(), a_len, b_len);
    }

    let mut out: Vec<Scalar> = vec![Scalar::zero(); a.len()];

    for i in 0..a.len(){
        let col: Vec<Scalar> = (0..a_len).map(|j| a[i][j]).collect();
        out[i] += inner_product(&col, b);
    }

    out
}

pub fn lm_mult(a: &[Scalar], b: &Vec<Vec<Scalar>>) -> Vec<Scalar> {
    let m = Vec::from(a);
    vm_mult(&m, b)
}

pub fn exp_iter(x:&Scalar) -> ScalarExp {
    ScalarExp { x: Scalar::one(), next_exp_x: x.clone() }
}

pub fn scalar_exp_u(x: &Scalar, pow: usize) -> Scalar {
    let mut result = Scalar::one();
    for i in 0..pow {
        result *= x;
    }

    result
}
pub fn scalar_exp(x: &Scalar, pow: i32) -> Scalar {
    let mut result = Scalar::one();
    for i in 0..pow {
        result *= x;
    }

    result
}

pub fn inner_product(a: &Vec<Scalar>, b: &Vec<Scalar>) -> Scalar {
    let mut out = Scalar::zero();
    if a.len() != b.len() {
        panic!("inner_product(a,b): lengths dont match, {}, {}", a.len(), b.len());
    }
    for i in 0..a.len() {
        out += a[i] * b[i];
    }
    out

}

pub fn give_n(n: i64) -> Scalar {
    let mut zero = Scalar::zero();
    for i in 0..n {
        zero += Scalar::one();
    }

    zero
}

pub fn format_scalar(s: &Scalar) -> String {
    I256::from_le_bytes(*s.reduce().as_bytes()).to_string()    
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

/// Iterator for Scalar exponentiation
pub struct ScalarExp {
    x: Scalar,
    next_exp_x: Scalar,
}

impl Iterator for ScalarExp {
    type Item = Scalar;

    fn next(&mut self) -> Option<Scalar> {
        let exp_x = self.next_exp_x;
        self.next_exp_x *= self.x;
        self.x = exp_x;
        Some(exp_x)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::max_value(), None)
    }
}
