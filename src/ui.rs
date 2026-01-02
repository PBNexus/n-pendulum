// src/ui.rs
use crate::logic::NPendulumSolver; // Import the N-pendulum physics solver from the logic module
use actix_web::{web, HttpResponse, Result}; // Actix-web types for request handling and HTTP responses
use base64::{engine::general_purpose, Engine as _}; // Base64 encoder for embedding image data
use plotters::prelude::*; // Plotters plotting library prelude
use serde::{Deserialize, Serialize}; // Serde traits for JSON (de)serialization
use std::io::{self, Cursor}; // IO utilities and Cursor for in-memory byte writing
use plotters::style::Palette99; // Color palette for generating multiple distinct colors
use image::{ImageFormat}; // Image encoding utilities for PNG output

#[derive(Deserialize)]
pub struct SimParams {
    n: usize,                // Number of pendulums
    masses: String,          // Comma-separated masses as a string
    lengths: String,         // Comma-separated lengths as a string
    initial_angles: String,  // Comma-separated initial angles (degrees) as a string
    t_max: f64,              // Maximum simulation time
    n_points: usize,         // Number of time steps / samples
}

#[derive(Serialize)]
struct SimResponse {
    success: bool,               // Whether the simulation succeeded
    trajectory_image: String,    // Base64-encoded PNG image of trajectories
    animation_data: AnimationData, // Raw position data for frontend animation
}

#[derive(Serialize)]
struct AnimationData {
    positions: Vec<Vec<f64>>, // Positions over time: [x1, y1, x2, y2, ...]
    n: usize,                 // Number of pendulums
    limit: f64,               // Plot boundary limit for consistent scaling
}

pub async fn simulate_handler(params: web::Json<SimParams>) -> Result<HttpResponse> {
    // Parse masses from comma-separated string into a vector of f64
    let masses: Vec<f64> = params.masses
        .split(',')                 // Split string by commas
        .filter_map(|s| s.trim().parse().ok()) // Trim and parse each value, skip invalid ones
        .collect();                 // Collect into a Vec<f64>

    // Validate that the number of masses matches n
    if masses.len() != params.n {
        return Ok(HttpResponse::Ok().json(SimResponse {
            success: false,                 // Mark simulation as failed
            trajectory_image: "".to_string(), // No image data
            animation_data: AnimationData {
                positions: vec![],          // Empty animation data
                n: 0,
                limit: 0.0,
            },
        }));
    }

    // Parse pendulum lengths from comma-separated string
    let lengths: Vec<f64> = params.lengths
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Parse initial angles (degrees) from comma-separated string
    let initial_angles_deg: Vec<f64> = params.initial_angles
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    // Convert initial angles from degrees to radians
    let initial_angles_rad: Vec<f64> = initial_angles_deg
        .iter()
        .map(|&d| d.to_radians())
        .collect();

    // Prepend dummy zero so indices align with 1-based pendulum math
    let mut full_masses = vec![0.0];
    full_masses.extend(masses);

    // Same padding for lengths
    let mut full_lengths = vec![0.0];
    full_lengths.extend(lengths);

    // Same padding for initial angles
    let mut full_initial_angles = vec![0.0];
    full_initial_angles.extend(initial_angles_rad);

    // Initialize angular velocities to zero for all pendulums
    let initial_ang_vels = vec![0.0; params.n + 1];

    // Create the N-pendulum solver instance
    let solver = NPendulumSolver::new(
        params.n,
        full_masses.clone(),
        full_lengths.clone(),
    );

    // Run the simulation and obtain time vector and state solution
    let (_t, sol) = solver.solve(
        full_initial_angles,
        initial_ang_vels,
        params.t_max,
        params.n_points,
    );

    // Compute total length of the pendulum system
    let sum_l: f64 = full_lengths[1..].iter().sum();

    // Define plot limits with padding
    let limit = sum_l + 0.5;

    // Allocate position storage: [time][x1, y1, x2, y2, ...]
    let mut positions = vec![vec![0.0; 2 * params.n]; sol.len()];

    // Convert angular states into Cartesian coordinates
    for (idx, state) in sol.iter().enumerate() {
        let mut curr_x = 0.0; // Current x position of the chain
        let mut curr_y = 0.0; // Current y position of the chain

        for k in 0..params.n {
            // Increment x using sin(theta)
            curr_x += full_lengths[k + 1] * state[k].sin();

            // Increment y using cos(theta) (negative for downward direction)
            curr_y -= full_lengths[k + 1] * state[k].cos();

            // Store x position
            positions[idx][2 * k] = curr_x;

            // Store y position
            positions[idx][2 * k + 1] = curr_y;
        }
    }

    // Image width in pixels
    const W: u32 = 500;

    // Image height in pixels
    const H: u32 = 500;

    // Allocate RGB pixel buffer (3 bytes per pixel)
    let mut pixel_buffer = vec![0u8; (W * H * 3) as usize];

    {
        // Create a Plotters drawing area backed by the raw pixel buffer
        let root = BitMapBackend::with_buffer(&mut pixel_buffer, (W, H))
            .into_drawing_area();

        // Fill background with white
        root.fill(&WHITE).map_err(io::Error::other)?;

        // Build a square Cartesian chart with equal axis limits
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Trajectories (n={})", params.n),
                ("sans-serif", 20).into_font(),
            )
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(-limit..limit, -limit..limit)
            .map_err(io::Error::other)?;

        // Draw grid lines and axes
        chart.configure_mesh().draw().map_err(io::Error::other)?;

        // Predefine a set of distinct line colors
        let mut colors: Vec<ShapeStyle> = vec![
            BLUE.mix(0.75).stroke_width(1),
            RED.mix(0.75).stroke_width(1),
            GREEN.mix(0.75).stroke_width(1),
            CYAN.mix(0.75).stroke_width(1),
            MAGENTA.mix(0.75).stroke_width(1),
            YELLOW.mix(0.75).stroke_width(1),
        ];

        // Generate additional colors if n exceeds predefined ones
        for i in colors.len()..params.n {
            colors.push(Palette99::pick(i).stroke_width(2));
        }

        // Draw trajectory for each pendulum mass
        for k in 0..params.n {
            // Extract x coordinates over time
            let xs: Vec<f64> = (0..sol.len())
                .map(|i| positions[i][2 * k])
                .collect();

            // Extract y coordinates over time
            let ys: Vec<f64> = (0..sol.len())
                .map(|i| positions[i][2 * k + 1])
                .collect();

            // Draw the line series for this pendulum
            chart
                .draw_series(LineSeries::new(
                    xs.iter().zip(ys.iter()).map(|(&x, &y)| (x, y)),
                    colors[k % colors.len()],
                ))
                .map_err(io::Error::other)?;
        }

        // Finalize drawing into the pixel buffer
        root.present().map_err(io::Error::other)?;
    }

    // Create an image buffer from raw RGB pixels
    let img_buffer = image::ImageBuffer::from_raw(W, H, pixel_buffer)
        .ok_or_else(|| io::Error::other("Failed to create image buffer"))?;

    let dynamic_image = image::DynamicImage::ImageRgb8(img_buffer);

    // Create an in-memory buffer to hold the PNG file bytes
    let mut png_buffer = Cursor::new(Vec::new());

    // Encode raw RGB pixels into PNG format
    dynamic_image.write_to(&mut png_buffer, ImageFormat::Png)
        .map_err(|e| io::Error::other(e.to_string()))?;

    let png_buffer = png_buffer.into_inner();

    // Convert PNG bytes into a Base64 data URL
    let plot_url = format!(
        "data:image/png;base64,{}",
        general_purpose::STANDARD.encode(&png_buffer)
    );

    // Return successful JSON response
    Ok(HttpResponse::Ok().json(SimResponse {
        success: true,
        trajectory_image: plot_url,
        animation_data: AnimationData {
            positions,
            n: params.n,
            limit,
        },
    }))
}
