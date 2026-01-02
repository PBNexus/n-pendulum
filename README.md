# n-pendulum-rust

A high-performance physics engine for simulating N-coupled pendulums, built with Rust and Lagrangian mechanics.

## 🚀 Live Demo
**[Click here to view the simulation live](YOUR_DEPLOYED_LINK_HERE)**

## 📖 The Physics
This project began as a standard double pendulum simulation, but I wanted to push the complexity further.

I manually derived the equations of motion for 2, 3, and 4 coupled pendulums on paper to identify the underlying mathematical pattern. Using these findings, I generalized the Lagrangian dynamics to solve for an arbitrary number of masses ($N$).

The engine uses **Rust** to handle the intense computational load and solves the system of differential equations using the **Runge-Kutta 4 (RK4)** numerical integration method.

## ✨ Features
* **Generalized N-System:** Simulates chaotic motion for any number of coupled pendulums (default limit set to 10 for UI performance).
* **High Precision:** Uses 4th-order Runge-Kutta (RK4) for stable integration.
* **Rust Backend:** Blazing fast matrix operations and state management.
* **Visualization:** * Real-time web animation.
    * Server-side phase-space trajectory rendering (via `plotters`).

## 🛠️ Local Development
If you want to run the code on your own machine:

1.  **Clone the repository**
    ```bash
    git clone [https://github.com/your-username/n-pendulum-rust.git](https://github.com/your-username/n-pendulum-rust.git)
    cd n-pendulum-rust
    ```

2.  **Run the server**
    ```bash
    cargo run
    ```

3.  **Open in browser**
    Navigate to `http://localhost:8000`

## 📂 Code Structure
* **`src/math.rs` & `src/logic.rs`**: **(Core)** My original implementation of the generalized Lagrangian matrix derivation and the RK4 solver.
* **`src/ui.rs` & `src/main.rs`**: Web server endpoints and image generation.
* **`static/`**: Frontend HTML/JS for the simulation controls and canvas animation.

## 📜 License
[MIT]