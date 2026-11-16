#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod ipc;
mod storage;
mod tabs;

use anyhow::{Context, Result};
use app::AppState;
use ipc::UserEvent;
use std::rc::Rc;
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::{Icon, WindowBuilder};

fn main() {
    if let Err(err) = run() {
        eprintln!("Fatal error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let mut builder = WindowBuilder::new()
        .with_title("Pluto Browser")
        .with_inner_size(LogicalSize::new(1280.0, 820.0))
        .with_min_inner_size(LogicalSize::new(480.0, 360.0));

    if let Some(icon) = build_brand_icon() {
        builder = builder.with_window_icon(Some(icon));
    }

    let window = Rc::new(
        builder
            .build(&event_loop)
            .context("Failed to create main browser window")?,
    );

    let mut app_state = AppState::new(Rc::clone(&window), proxy)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    app_state.save_session();
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    app_state.handle_resize();
                }
                _ => {}
            },
            Event::UserEvent(user_event) => {
                app_state.handle_user_event(user_event);
            }
            _ => {}
        }
    });
}

fn build_brand_icon() -> Option<Icon> {
    const SIZE: u32 = 64;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);

    // Orbital ring tilt (-26 degrees)
    let angle = -0.4538_f32;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let rx = 0.40_f32;
    let ry = 0.135_f32;
    let planet_r = 0.215_f32;

    for py in 0..SIZE {
        for px in 0..SIZE {
            let x = (px as f32 + 0.5) / SIZE as f32;
            let y = (py as f32 + 0.5) / SIZE as f32;

            // Rounded square SDF centered at (0.5, 0.5), half-size 0.45, radius 0.14
            let dx = (x - 0.5).abs() - 0.31;
            let dy = (y - 0.5).abs() - 0.31;
            let ox = dx.max(0.0);
            let oy = dy.max(0.0);
            let dist = (ox * ox + oy * oy).sqrt() + dx.max(dy).min(0.0) - 0.14;
            let bg_alpha = (0.5 - dist * SIZE as f32).clamp(0.0, 1.0);

            if bg_alpha <= 0.0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }

            // Deep space navy (#0a1530) with subtle indigo Kuiper aura behind Pluto
            let radial = ((x - 0.5).powi(2) + (y - 0.5).powi(2)).sqrt();
            let aura = (1.0 - radial * 1.9).max(0.0).powi(2);
            let mut r = 10.0 + aura * 52.0;
            let mut g = 21.0 + aura * 38.0;
            let mut b = 48.0 + aura * 124.0;

            // Local rotated coordinates for the elliptical orbital ring
            let cx = x - 0.5;
            let cy = y - 0.5;
            let u = cx * cos_a - cy * sin_a;
            let v = cx * sin_a + cy * cos_a;

            let norm_sq = (u / rx).powi(2) + (v / ry).powi(2);
            let grad_mag = 2.0 * ((u / (rx * rx)).powi(2) + (v / (ry * ry)).powi(2)).sqrt().max(0.001);
            let ring_dist = ((norm_sq - 1.0) / grad_mag).abs() - 0.022;
            let ring_alpha = (0.5 - ring_dist * SIZE as f32).clamp(0.0, 1.0);

            // Back arc of the orbital ring (v < 0)
            if v < 0.0 && ring_alpha > 0.0 {
                let a = ring_alpha * 0.68;
                r = r * (1.0 - a) + 178.0 * a;
                g = g * (1.0 - a) + 156.0 * a;
                b = b * (1.0 - a) + 245.0 * a;
            }

            // Planet Pluto sphere
            let planet_dist = radial - planet_r;
            let planet_alpha = (0.5 - planet_dist * SIZE as f32).clamp(0.0, 1.0);
            if planet_alpha > 0.0 {
                let nx = (cx / planet_r).clamp(-1.0, 1.0);
                let ny = (cy / planet_r).clamp(-1.0, 1.0);
                let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();

                // Directional sun light from top-left
                let diffuse = (nx * -0.52 + ny * -0.48 + nz * 0.70).clamp(0.0, 1.0);
                let rim = (1.0 - nz).powi(3);

                // Warm twilight Pluto surface + bright nitrogen ice Tombaugh Regio highlight
                let mut pr = 64.0 + diffuse * 158.0 + rim * 65.0;
                let mut pg = 52.0 + diffuse * 130.0 + rim * 55.0;
                let mut pb = 110.0 + diffuse * 112.0 + rim * 110.0;

                // Brighter icy heart-basin accent on lower-right lit face
                let ice_d = ((nx - 0.16).powi(2) + (ny + 0.04).powi(2)).sqrt();
                let ice = (1.0 - ice_d * 1.9).max(0.0) * 0.55;
                pr = pr * (1.0 - ice) + 248.0 * ice;
                pg = pg * (1.0 - ice) + 236.0 * ice;
                pb = pb * (1.0 - ice) + 246.0 * ice;

                r = r * (1.0 - planet_alpha) + pr * planet_alpha;
                g = g * (1.0 - planet_alpha) + pg * planet_alpha;
                b = b * (1.0 - planet_alpha) + pb * planet_alpha;
            }

            // Front arc of the orbital ring (v >= 0) crossing in front of Pluto
            if v >= 0.0 && ring_alpha > 0.0 {
                let a = ring_alpha * 0.94;
                r = r * (1.0 - a) + 232.0 * a;
                g = g * (1.0 - a) + 216.0 * a;
                b = b * (1.0 - a) + 255.0 * a;
            }

            // Charon moon satellite at upper-right orbit
            let moon_d = ((x - 0.785).powi(2) + (y - 0.285).powi(2)).sqrt() - 0.046;
            let moon_alpha = (0.5 - moon_d * SIZE as f32).clamp(0.0, 1.0);
            if moon_alpha > 0.0 {
                r = r * (1.0 - moon_alpha) + 242.0 * moon_alpha;
                g = g * (1.0 - moon_alpha) + 234.0 * moon_alpha;
                b = b * (1.0 - moon_alpha) + 255.0 * moon_alpha;
            }

            rgba.push(r.clamp(0.0, 255.0) as u8);
            rgba.push(g.clamp(0.0, 255.0) as u8);
            rgba.push(b.clamp(0.0, 255.0) as u8);
            rgba.push((bg_alpha * 255.0).clamp(0.0, 255.0) as u8);
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).ok()
}

