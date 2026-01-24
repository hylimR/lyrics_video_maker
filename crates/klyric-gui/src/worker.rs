use iced::widget::image;

// use iced::futures::SinkExt; // Removed
use klyric_renderer::model::KLyricDocumentV2;
use klyric_renderer::renderer::Renderer;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::sync::mpsc as tokio_mpsc;



pub enum RenderingRequest {
    Render {
        doc: Arc<KLyricDocumentV2>,
        time: f64,
        width: u32,
        height: u32,
    }
}

pub enum RenderingResponse {
    FrameRendered(image::Handle),
    Error(String),
}

#[derive(Clone, Debug)]
pub struct RenderWorker {
    tx: tokio_mpsc::UnboundedSender<RenderingRequest>,
}

impl RenderWorker {
    pub fn request_frame(&self, doc: Arc<KLyricDocumentV2>, time: f64, width: u32, height: u32) {
        let _ = self.tx.send(RenderingRequest::Render { 
            doc,
            time, 
            width, 
            height 
        });
    }
}

#[derive(Clone)]
pub struct WorkerConnection {
    worker: RenderWorker,
    receiver: Arc<Mutex<Option<tokio_mpsc::UnboundedReceiver<RenderingResponse>>>>,
}

pub fn spawn() -> WorkerConnection {
    let (req_tx, mut req_rx) = tokio_mpsc::unbounded_channel::<RenderingRequest>();
    let (res_tx, res_rx) = tokio_mpsc::unbounded_channel();

    thread::spawn(move || {
        let mut renderer: Option<Renderer> = None;
        let mut last_size = (0, 0);

        while let Some(msg) = req_rx.blocking_recv() {
            match msg {
                RenderingRequest::Render { doc, time, width, height } => {
                    if renderer.is_none() || last_size != (width, height) {
                        renderer = Some(Renderer::new(width, height));
                        last_size = (width, height);
                    }

                    if let Some(r) = renderer.as_mut() {
                        match r.render_frame(&doc, time) {
                            Ok(pixels) => {
                                let handle = image::Handle::from_rgba(width, height, pixels);
                                let _ = res_tx.send(RenderingResponse::FrameRendered(handle));
                            }
                            Err(e) => {
                                let _ = res_tx.send(RenderingResponse::Error(e.to_string()));
                            }
                        }
                    }
                }
            }
        }
    });

    WorkerConnection {
        worker: RenderWorker { tx: req_tx },
        receiver: Arc::new(Mutex::new(Some(res_rx))),
    }
}

// Subscription removed in favor of polling in app.rs
// pub fn subscription(...) ... 

impl WorkerConnection {
    pub fn get_worker(&self) -> RenderWorker {
        self.worker.clone()
    }
    
    pub fn try_recv(&self) -> Result<RenderingResponse, tokio_mpsc::error::TryRecvError> {
        let mut guard = self.receiver.lock().expect("Lock poisoned");
        if let Some(rx) = guard.as_mut() {
            rx.try_recv()
        } else {
            Err(tokio_mpsc::error::TryRecvError::Disconnected)
        }
    }
}
