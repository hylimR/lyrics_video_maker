use anyhow::Result;
use klyric_renderer::{
    model::KLyricDocumentV2,
    renderer::Renderer,
    text::TextRenderer,
};
use skia_safe::{Canvas, Color};
use std::io::{self, BufRead};
use serde::Deserialize;
use std::sync::mpsc;
use std::thread;

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd", content = "payload")]
enum Command {
    LoadDoc(KLyricDocumentV2),
    SetTime(f64),
    Resize { w: u32, h: u32 },
}

pub struct PreviewApp {
    renderer: Option<Renderer>,
    doc: Option<KLyricDocumentV2>,
    time: f64,
    rx: mpsc::Receiver<Command>,
}

impl PreviewApp {
    pub fn new() -> Self {
        // Spawn stdin reader thread
        let (tx, rx) = mpsc::channel();
        
        thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut line = String::new();
            
            while handle.read_line(&mut line).unwrap_or(0) > 0 {
                if let Ok(cmd) = serde_json::from_str::<Command>(&line) {
                    let _ = tx.send(cmd);
                }
                line.clear();
            }
        });

        Self {
            renderer: None, // Initialized on first resize/render
            doc: None,
            time: 0.0,
            rx,
        }
    }

    pub fn poll_stdin(&mut self) -> bool {
        let mut needs_redraw = false;
        while let Ok(cmd) = self.rx.try_recv() {
            match cmd {
                Command::LoadDoc(doc) => {
                    self.doc = Some(doc);
                    needs_redraw = true;
                },
                Command::SetTime(t) => {
                    self.time = t;
                    needs_redraw = true;
                },
                Command::Resize { w, h } => {
                     // Renderer needs to be recreated if size changes?
                     // Or just updated. klyric-renderer's Renderer might need recreation.
                     // The native Renderer struct usually takes w/h in constructor.
                     self.renderer = Some(Renderer::new(w, h));
                     needs_redraw = true;
                }
            }
        }
        needs_redraw
    }

    pub fn render(&mut self, canvas: &Canvas) {
        canvas.clear(Color::BLACK);
        
        if let Some(renderer) = &mut self.renderer {
            if let Some(doc) = &self.doc {
                if let Ok(_) = renderer.render_to_canvas(canvas, doc, self.time) {
                     // Success
                }
            }
        }
    }
}
