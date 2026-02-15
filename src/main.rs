use std::time::Duration;

use clap::Parser;
use chrono::Utc;
use cron::Schedule;
use rodio::{Decoder, OutputStream, buffer::SamplesBuffer, source::{self, Source}};

fn main() {
    let cli =  Config::parse();
    let audio_stream = match cli.selected_audio {
        None => AudioStream::new(include_bytes!("../assets/brvhrtz-stab-f-01-brvhrtz-224599.raw")),
        Some(file_path) => todo!()
    };

    let default_cron = "0/20 * * * * *";
    let cron: Schedule = cli.cron
        .as_deref()
        .and_then(|s| Schedule::try_from(s).ok())
        .unwrap_or_else(|| Schedule::try_from(default_cron).expect("Default value, should never fail"));

    loop {
        let now = Utc::now();
        if let Some(next) = cron.upcoming(Utc).next() {
            // we do not want to spawn a million thread if the cron is too fast
            let minimum_duration = Duration::from_secs(60);
            let duration = next.signed_duration_since(now).to_std()
                .unwrap_or(Duration::ZERO)
                .max(minimum_duration);
            println!("posture check in {:?}", duration);
            std::thread::sleep(duration);
            let value = audio_stream.clone();
            std::thread::spawn(move || value.play_audio());

        }
    }

}

#[derive(Debug, Clone)]
struct AudioStream {
    audio: Vec<f32>,
    curr_pos: usize
}

impl Source for AudioStream{
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        2
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        44100
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(Duration::new(1, 6))
    }
}

impl Iterator for AudioStream {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let val = Some(self.audio[self.curr_pos]);
        self.curr_pos += 1;
        val
    }
}

impl AudioStream {
    fn new(data: &[u8]) -> Self {
        let processed_data: Vec<f32> = data.chunks_exact(4)
            .map(|slice| {
                //log!("{:#x} {:#x} {:#x} {:#x} ", slice[0], slice[1], slice[2], slice[3]);
                f32::from_ne_bytes([slice[0], slice[1], slice[2], slice[3]])
            })
            .collect();

        AudioStream{
            audio: processed_data,
            curr_pos: 0
        }
    }

    pub fn play_audio(&self){
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().expect("Audio stream not found");
        let decoder = SamplesBuffer::new(2, 44100, self.audio.clone());
        stream_handle.mixer().add(decoder);
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Config {
    /// Which audio to play, currently not implemented
    selected_audio: Option<String>,

    /// Cron string for the effect
    cron: Option<String>
}

