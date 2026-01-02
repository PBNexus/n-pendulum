// src/logic.rs
// This module implements the simulation loop and numerical integration, analogous to UserEquationSolver in the double-pendulum reference.
// It defines NPendulumSolver for fixed parameters (n, masses, lengths) and handles state evolution via RK4 (since no odeint equivalent; RK4 is accurate/stable for this ODE).
// deriv computes dy/dt = [ω1, ω2, ..., α1, α2, ...] where α = M^{-1} (-C - G), using matrices from math.rs.
// solve_linear_system is a basic Gaussian elimination with partial pivoting (no external crate, self-contained; handles small n ~10 efficiently).
// Assumptions: Initial velocities=0 (like reference); fixed dt for simplicity (matches linspace); angles in radians; state y = [θ1..θn, ω1..ωn] (no dummies in y).
// RK4 step size is fixed; tolerances not needed (fixed steps). No mxstep limit, but n_points=4000 is small.
// Matches reference's t_max=60, n_points=4000; returns t and sol (states over time).

use crate::math::NPendulumMath; // Import from math.rs.
use std::f64;

/// Basic Gaussian elimination solver for M x = b (n x n system). In-place on copies; partial pivoting for stability.
/// Written explicitly (no crate) to match reference's linalg.solve simplicity; assumes M invertible (physical system).
/// Forward elimination + back-substitution; handles f64 precision.
pub fn solve_linear_system(a: &[Vec<f64>], b: &[f64]) -> Vec<f64> {
    // Takes & for borrow; returns owned Vec.
    let n = b.len(); // Assumes square n x n.
    let mut mat = a.to_vec(); // Copy matrix for modification.
    let mut rhs = b.to_vec(); // Copy RHS.
                             // Forward elimination with partial pivoting.
    for i in 0..n {
        // Pivot column i.
        // Find max pivot row.
        let mut max_row = i; // Start with current row.
        for k in (i + 1)..n {
            // Search below.
            if mat[k][i].abs() > mat[max_row][i].abs() {
                // Absolute value for stability.
                max_row = k; // Update max.
            }
        }
        // Swap rows.
        mat.swap(i, max_row); // Swap matrix rows.
        rhs.swap(i, max_row); // Swap RHS.
                              // Eliminate below.
        for k in (i + 1)..n {
            // For each row below.
            let c = -mat[k][i] / mat[i][i]; // Elimination coefficient (avoid div0 assumed).
            for j in i..n {
                // Columns from i (pivot col onward).
                if i == j {
                    // Set pivot col to 0.
                    mat[k][j] = 0.0;
                } else {
                    mat[k][j] += c * mat[i][j]; // Update.
                }
            }
            rhs[k] += c * rhs[i]; // Update RHS.
        }
    }
    // Back-substitution.
    let mut x = vec![0.0; n]; // Solution vec.
    for i in (0..n).rev() {
        // From bottom up.
        let mut sum_ax = 0.0; // Accumulator for sum mat[i][j>i] * x[j].
        for (j, &x_j) in x[(i + 1)..n].iter().enumerate() {
            // j > i.
            sum_ax += mat[i][i + 1 + j] * x_j;
        }
        x[i] = (rhs[i] - sum_ax) / mat[i][i]; // Solve x[i] = (b - sum) / diag.
    }
    x // Return solution α.
}

/// Solver struct: holds fixed params, computes deriv and integrates.
pub struct NPendulumSolver {
    // Fields for params (immutable after new).
    pub n: usize,
    pub masses: Vec<f64>,  // Full [0, m1, ...].
    pub lengths: Vec<f64>, // Full [0(unused), L1, ...].
}

impl NPendulumSolver {
    /// Creates solver with params.
    pub fn new(n: usize, masses: Vec<f64>, lengths: Vec<f64>) -> Self {
        Self {
            n,
            masses,
            lengths,
        }
    }

    /// Computes α = M^{-1} (-C - G) using current θ, ω. Builds full 1-indexed vectors with dummy 0.
    pub fn accelerations(&self, angles: &[f64], ang_vels: &[f64]) -> Vec<f64> {
        // angles/ang_vels are full [0, θ1, ...].
        let math = NPendulumMath::new(
            self.n,
            self.masses.clone(),
            self.lengths.clone(),
            angles.to_vec(),
            ang_vels.to_vec(),
        ); // Temp instance.
        let m_mat = math.set_mass_matrix(); // n x n.
        let c_vec = math.set_centripetal_matrix(); // Vec n.
        let g_vec = math.set_grav_matrix(); // Vec n.
        let mut rhs = vec![0.0; self.n]; // RHS = - (C + G).
        for i in 0..self.n {
            // 0-based for vec.
            rhs[i] = -(c_vec[i] + g_vec[i]);
        }
         // Solve.
        solve_linear_system(&m_mat, &rhs)
    }

    /// Computes dy/dt for state y = [θ1, ..., θn, ω1, ..., ωn] (2n vec, no dummies).
    pub fn deriv(&self, y: &[f64], _t: f64) -> Vec<f64> {
        // _t unused (autonomous ODE).
        let n = self.n;
        let mut angles = vec![0.0; n + 1]; // Full 1-indexed with dummy 0.
        let mut ang_vels = vec![0.0; n + 1];
        angles[1..n + 1].copy_from_slice(&y[0..n]);
        ang_vels[1..n + 1].copy_from_slice(&y[n..2 * n]);
        let alpha = self.accelerations(&angles, &ang_vels); // Get α (size n, 0-based).
        let mut dydt = vec![0.0; 2 * n]; // dy/dt size 2n.
        dydt[0..n].copy_from_slice(&ang_vels[1..n + 1]); // dθ_i / dt = ω_i.
        dydt[n..2 * n].copy_from_slice(&alpha); // dω_i / dt = α_i.
        dydt
    }

    /// Single RK4 step: y_{t+dt} = y + (dt/6)(k1 + 2k2 + 2k3 + k4). Standard formula for accuracy.
    fn rk4_step(&self, y: &[f64], t: f64, dt: f64) -> Vec<f64> {
        let k1 = self.deriv(y, t); // f(y, t).
        let y2: Vec<f64> = (0..y.len()).map(|i| y[i] + 0.5 * dt * k1[i]).collect(); // y + (dt/2) k1.
        let k2 = self.deriv(&y2, t + 0.5 * dt);
        let y3: Vec<f64> = (0..y.len()).map(|i| y[i] + 0.5 * dt * k2[i]).collect(); // y + (dt/2) k2.
        let k3 = self.deriv(&y3, t + 0.5 * dt);
        let y4: Vec<f64> = (0..y.len()).map(|i| y[i] + dt * k3[i]).collect(); // y + dt k3.
        let k4 = self.deriv(&y4, t + dt);
        let mut y_new = vec![0.0; y.len()];
        for i in 0..y.len() {
            // Weighted sum.
            y_new[i] = y[i] + (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        y_new
    }

    /// Integrates from t=0 to t_max with n_points steps. Returns t (linspace), sol (n_points x 2n states).
    /// Initial ω=0; θ from input (radians, full [0, θ1,..]); fixed dt = t_max / (n_points-1).
    pub fn solve(
        &self,
        initial_angles: Vec<f64>,
        initial_ang_vels: Vec<f64>,
        t_max: f64,
        n_points: usize,
    ) -> (Vec<f64>, Vec<Vec<f64>>) {
        let n = self.n;
        let num_steps = n_points - 1; // For linspace(0, t_max, n_points).
        let dt = t_max / num_steps as f64; // Fixed step.
        let mut t = vec![0.0]; // t[0] = 0.
        let mut y = vec![0.0; 2 * n]; // Initial state (2n).
        y[0..n].copy_from_slice(&initial_angles[1..n + 1]);
        y[n..2 * n].copy_from_slice(&initial_ang_vels[1..n + 1]);
        let mut sol = vec![y.clone()]; // sol[0] = y0.
        let mut curr_t = 0.0;
        for _ in 1..n_points {
            // Loop n_points-1 times.
            y = self.rk4_step(&y, curr_t, dt); // Advance.
            curr_t += dt;
            t.push(curr_t); // Append t.
            sol.push(y.clone()); // Append state.
        }
        (t, sol) // Return.
    }
}
