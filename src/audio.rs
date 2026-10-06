use sdl3::AudioSubsystem;
use sdl3::audio::{AudioCallback, AudioFormat, AudioSpec, AudioStream, AudioStreamWithCallback};

pub struct Beeper {
    stream: AudioStreamWithCallback<SquareWave>,
    is_active: bool,
}

impl Beeper {
    pub fn new(audio_subsystem: &AudioSubsystem) -> Self {
        let source_freq = 44_100;
        let source_spec = AudioSpec {
            freq: Some(source_freq),
            channels: Some(1),
            format: Some(AudioFormat::f32_sys()),
        };
        let stream = audio_subsystem
            .open_playback_stream(
                &source_spec,
                SquareWave {
                    phase_inc: 440.0 / source_freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                },
            )
            .expect("Failed to open audio stream");

        // SDL opens the playback stream paused.
        Self {
            stream,
            is_active: false,
        }
    }

    pub fn set_active(&mut self, is_active: bool) {
        if is_active == self.is_active {
            return;
        }

        if is_active {
            self.stream.resume().expect("Failed to start playback");
        } else {
            self.stream.pause().expect("Failed to pause playback");
        }
        self.is_active = is_active;
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback<f32> for SquareWave {
    fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
        let mut out = Vec::with_capacity(requested as usize);

        for _ in 0..requested {
            out.push(if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            });
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }

        stream
            .put_data_f32(&out)
            .expect("Failed to queue audio samples");
    }
}
