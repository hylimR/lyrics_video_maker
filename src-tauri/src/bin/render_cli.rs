//! CLI tool for rendering lyric videos
//! 
//! Usage: render_cli --input <json> --output <video> --duration <seconds>

use std::env;
use std::fs;

use std::sync::{Arc, atomic::AtomicBool};

use lyric_video_maker_lib::video::pipeline::{run_render_pipeline, RenderOptions};
use lyric_video_maker_lib::KLyricDocumentV2;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: render_cli <input_json> <output_video> [duration]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let duration_arg = args.get(3).and_then(|s| s.parse::<f64>().ok());

    println!("Reading input: {}", input_path);
    let json_content = fs::read_to_string(input_path).expect("Failed to read input file");
    
    let document: KLyricDocumentV2 = serde_json::from_str(&json_content).expect("Failed to parse KLyric JSON");

    let mut options = RenderOptions::default();
    options.width = 1920;
    options.height = 1080;
    options.fps = 30;
    
    if let Some(duration) = duration_arg {
        println!("Custom duration: {:.2}s", duration);
        options.custom_duration = Some(duration);
    }

    let cancellation_token = Arc::new(AtomicBool::new(false));

    println!("Starting render...");
    let result = run_render_pipeline(
        document,
        None, // No audio for CLI test
        output_path.to_string(),
        options,
        cancellation_token,
        |progress| {
             // Print progress every 10% or so to avoid spamming
             if (progress.percentage % 10.0) < 0.5 || progress.phase != "Rendering frames" {
                 println!("[{}] {:.1}% - {:.1} fps - ETA: {:.1}s", 
                    progress.phase, progress.percentage, progress.render_fps, progress.eta_seconds);
             }
        },
        None::<fn(Vec<u8>)>, // No preview callback
    );

    match result {
        Ok(res) => {
            println!("Render Success!");
            println!("Output: {}", res.output_path);
            println!("Time: {:.2}s", res.render_time);
            println!("Avg FPS: {:.2}", res.avg_fps);
        },
        Err(e) => {
            eprintln!("Render Failed: {}", e);
            std::process::exit(1);
        }
    }
}
