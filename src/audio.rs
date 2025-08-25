use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, source::Source};

pub struct AudioManager {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioManager {
    pub fn new_loop(path: &str, volume: f32) -> Self {
        let stream = OutputStreamBuilder::open_default_stream()
            .expect("AudioManager: no se pudo inicializar el stream de audio por defecto");

        let sink = Sink::connect_new(&stream.mixer());

        let file = File::open(path)
            .unwrap_or_else(|_| panic!("AudioManager: no se pudo abrir '{}'", path));
        let buf = BufReader::new(file);

        let decoder = Decoder::new_looped(buf)
            .expect("AudioManager: error al decodificar el audio (formato no soportado?)");

        sink.append(decoder);
        sink.set_volume(volume.clamp(0.0, 1.0));
        sink.play();

        AudioManager {
            _stream: stream,
            sink,
        }
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume.clamp(0.0, 1.0));
    }
    pub fn pause(&self) { self.sink.pause(); }
    pub fn play(&self) { self.sink.play(); }
    pub fn stop(&self) { self.sink.stop(); }
}
