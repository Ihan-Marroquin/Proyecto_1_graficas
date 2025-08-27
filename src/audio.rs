use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::sync::Mutex;

pub struct AudioManager {
    stream: OutputStream,   
    music_sink: Sink,      
    sfx_master_volume: f32,  
    sfx_sinks: Mutex<Vec<Sink>>,
}

impl AudioManager {
    pub fn new_loop(music_path: &str, music_volume: f32, sfx_volume: f32) -> Self {
        let stream = OutputStreamBuilder::open_default_stream()
            .expect("AudioManager: no se pudo inicializar el stream de audio por defecto");

        let music_sink = Sink::connect_new(&stream.mixer());

        match File::open(music_path) {
            Ok(file) => {
                let buf = BufReader::new(file);
                match Decoder::new_looped(buf) {
                    Ok(decoder) => {
                        music_sink.append(decoder);
                    }
                    Err(e) => {
                        eprintln!("AudioManager: error decodificando '{}': {}", music_path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("AudioManager: no se pudo abrir '{}': {}", music_path, e);
            }
        }

        music_sink.set_volume(music_volume.clamp(0.0, 1.0));
        music_sink.play();

        AudioManager {
            stream,
            music_sink,
            sfx_master_volume: sfx_volume.clamp(0.0, 1.0),
            sfx_sinks: Mutex::new(Vec::new()),
        }
    }

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_sink.set_volume(volume.clamp(0.0, 1.0));
    }

    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn play_sfx(&self, path: &str, vol: f32) {
        match File::open(path) {
            Ok(file) => {
                let buf = BufReader::new(file);
                match Decoder::new(buf) {
                    Ok(decoder) => {
                        let sink = Sink::connect_new(&self.stream.mixer());
                        let final_vol = (vol.clamp(0.0, 1.0) * self.sfx_master_volume).clamp(0.0, 1.0);
                        sink.append(decoder);
                        sink.set_volume(final_vol);

                        eprintln!("AudioManager: playing SFX '{}' vol={} final_vol={}", path, vol, final_vol);

                        if let Ok(mut vec) = self.sfx_sinks.lock() {
                            vec.push(sink);
                            vec.retain(|s| !s.empty());
                        }
                    }
                    Err(e) => {
                        eprintln!("AudioManager: error decodificando SFX '{}': {}", path, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("AudioManager: no se pudo abrir SFX '{}' : {}", path, e);
            }
        }
    }

    pub fn pause_music(&self) {
        self.music_sink.pause();
    }

    pub fn resume_music(&self) {
        self.music_sink.play();
    }

    pub fn stop_music(&self) {
        self.music_sink.stop();
    }
}
