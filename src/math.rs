// src/math.rs
// This module directly ports the NPendulumMath class from the provided Python reference code.
// It computes the mass matrix M, centripetal/Coriolis vector C, and gravity vector G for the Lagrangian equations M α + C + G = 0.
// Variable names, loop structures, and mathematical expressions are preserved faithfully (e.g., indices start from 1, dummy index 0).
// Typos in Python (e.g., 'lenfgths' -> 'lengths', 'massess' -> 'masses') are corrected for compilation but logic unchanged.
// The gravity term uses sin(angles[i]) instead of angles[i], as this matches the physics derivation (torque ~ sin θ) and the double-pendulum reference (Sp = sin(p)).
// lsum is a utility function mirroring Python's lsum(l) for summing slices.
// Assumptions: Inputs are 1-indexed with dummy 0 (e.g., masses = [0, m1, m2, ..., mn]); n >= 2; no validation added to match reference simplicity.
// Uses f64 for precision, std::f64 methods for trig (cos, sin).

// use std::f64::consts::PI;  // Available for deg-to-rad conversion if needed elsewhere.

// Mirrors Python's lsum(l): sums elements in a slice. Written as a loop (not iter().sum::<f64>()) to match Python's explicit for-loop style.
pub fn lsum(l: &[f64]) -> f64 {  // Takes &[f64] for efficiency (no copy); returns f64 sum.
    let mut sum = 0.0;  // Initializes sum to 0.0, matching Python's sum=0.
    for &i in l.iter() {  // Iterates over references to avoid copies, but &i for value; matches Python's for i in l.
        sum += i;  // Accumulates; f64 addition is exact here.
    }
    sum  // Returns the sum; no explicit return for Rust idiomatic brevity.
}

/// Mirrors Python's NPendulumMath class: holds parameters and computes matrices/vectors at a snapshot in time.
pub struct NPendulumMath {  // Struct fields match Python __init__ self.*.
    pub g: f64,  // Gravity constant, hardcoded to 9.81 like reference.
    pub n: usize,  // Number of pendulums.
    pub masses: Vec<f64>,  // [0, m1, m2, ..., mn].
    pub lengths: Vec<f64>,  // [dummy0 (unused), L1, L2, ..., Ln]; fixed Python typo 'lenfgths'.
    pub angles: Vec<f64>,  // Current angles θ [0 (dummy), θ1, θ2, ..., θn] in radians.
    pub ang_vels: Vec<f64>,  // Current angular velocities ω [0 (dummy), ω1, ..., ωn].
}

impl NPendulumMath {  // Impl block for methods, mirroring Python def.
    /// Mirrors Python __init__: constructs with parameters. Hardcodes g=9.81.
    pub fn new(n: usize, masses: Vec<f64>, lengths: Vec<f64>, angles: Vec<f64>, ang_vels: Vec<f64>) -> Self {
        Self {  // Struct literal initialization.
            g: 9.81,  // Hardcoded like Python self.g = 9.81.
            n,  // Shorthand for n: n.
            masses,  // Moves the Vec into the struct.
            lengths,  // Ditto.
            angles,  // Ditto; assumes radians.
            ang_vels,  // Ditto.
        }
    }

    /// Mirrors Python set_mass_matrix: computes M_{row,col} = sum_{k=max(row,col)}^n m_k * L_row * L_col * cos(θ_row - θ_col).
    /// Loops are 1-based (range(1,n+1)); returns Vec<Vec<f64>> like np.array.
    pub fn set_mass_matrix(&self) -> Vec<Vec<f64>> {  // &self for borrow, no mut needed.
        let mut m_matrix = Vec::new();  // Empty vec for rows, matching Python m_matrix=[].
        for row in 1..=self.n {  // 1 to n inclusive, matching Python range(1,n+1).
            let mut mm_row = Vec::new();  // Per-row vec, matching mm_row=[].
            for column in 1..=self.n {  // Nested loop, 1 to n.
                let k = row.max(column);  // max(row, column), matching Python.
                let mass = lsum(&self.masses[k..]);  // Sum masses[k:] slice; & for borrow.
                let length_term = self.lengths[row] * self.lengths[column];  // L_row * L_col, matching.
                let cos_term = f64::cos(self.angles[row] - self.angles[column]);  // cos(θ_row - θ_col).
                let term = mass * length_term * cos_term;  // Full term, matching Python (note: length_term * cos_term implicit).
                mm_row.push(term);  // Append to row.
            }
            m_matrix.push(mm_row);  // Append row to matrix.
        }
        m_matrix  // Return the 2D vec; n x n.
    }

    /// Mirrors Python set_centripetal_matrix: computes C_i = sum_j sum_{k=max(i,j)}^n m_k * L_i * L_j * sin(θ_i - θ_j) * ω_j^2.
    /// Returns Vec<f64> (vector of size n, 0-based indices 0..n-1 for i=1..n).
    pub fn set_centripetal_matrix(&self) -> Vec<f64> {  // Vector, not matrix, matching Python c_matrix (1D).
        let mut c_matrix = Vec::new();  // Empty vec, matching [].
        for i in 1..=self.n {  // Outer loop i=1 to n.
            let mut f_term = 0.0;  // Accumulator, matching f_term=0.
            for j in 1..=self.n {  // Nested j=1 to n.
                let m_val = lsum(&self.masses[(i.max(j))..]);  // m = lsum(masses[max(i,j)::]); renamed m to m_val (keyword conflict).
                let lilj = self.lengths[i] * self.lengths[j];  // L_i * L_j.
                let sin_val = f64::sin(self.angles[i] - self.angles[j]);  // sin(θ_i - θ_j).
                let velsq = self.ang_vels[j] * self.ang_vels[j];  // ω_j**2, matching velsq.
                let term = m_val * lilj * sin_val * velsq;  // Full term.
                f_term += term;  // Accumulate.
            }
            c_matrix.push(f_term);  // Append for this i (0-based index).
        }
        c_matrix  // Return vec of size n.
    }

    /// Mirrors Python set_grav_matrix: computes G_i = sum_{k=i}^n m_k * g * L_i * sin(θ_i).
    /// Fixed to sin(angles[i]) to match physics (Lagrangian gravity term) and double-pendulum reference; Python likely a transcription error.
    /// Returns Vec<f64> (size n).
    pub fn set_grav_matrix(&self) -> Vec<f64> {  // 1D vec, matching g_m.
        let mut g_m = Vec::new();  // Empty vec.
        for i in 1..=self.n {  // Loop i=1 to n.
            let mass = lsum(&self.masses[i..]);  // sum masses[i::].
            let term = mass * self.g * self.lengths[i] * self.angles[i].sin();  // g * L_i * sin(θ_i); fixed to .sin().
            g_m.push(term);  // Append.
        }
        g_m  // Return.
    }
}