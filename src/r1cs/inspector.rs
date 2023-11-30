use super::{LinearCombination, R1CSError, Variable};;;;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

/// The interface for a controllable and readable constraint system 
/// for inspection of the R1CS(Rank 1 Constraint System), 
/// which can be used to represent Arithmetic Circuits 
pub trait InspectorConstraintSystem: ConstraintSystem {


}
