use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    current_file: Option<PathBuf>,
}

impl AudioManager {
    pub fn new() -> Option<Self> {
        let (stream, stream_handle) = OutputStream::try_default().ok()?;
        let sink = Sink::try_new(&stream_handle).ok()?;

        Some(Self {
            _stream: stream,
            stream_handle,
            sink,
            current_file: None,
        })
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref().to_path_buf();

        // Stop current playback
        self.sink.stop();

        // recreate sink to clear queue and ensure clean state
        if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
            self.sink = new_sink;
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader)?;

        self.sink.append(source);
        self.sink.pause(); // Start paused

        self.current_file = Some(path);

        Ok(())
    }

    pub fn play(&self) {
        if !self.sink.empty() {
            self.sink.play();
        }
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn stop(&self) {
        self.sink.pause();
        // To truly "stop" and reset to 0 without clearing the queue, we'd seek to 0.
        // rodio sinks don't really have a "stop and rewind" without clearing.
        // But we usually just want to pause and reset position.
        let _ = self.seek(Duration::from_secs(0));
    }

    pub fn seek(&self, time: Duration) -> anyhow::Result<()> {
        // rodio's seek might be tricky depending on the source.
        // For WAV/MP3 decoded via rodio, it usually works.
        self.sink
            .try_seek(time)
            .map_err(|e| anyhow::anyhow!("Seek failed: {}", e))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    #[allow(dead_code)]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}

impl std::fmt::Debug for AudioManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioManager")
            .field("current_file", &self.current_file)
            .finish()
    }
}




