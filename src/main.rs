use std::path::PathBuf;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use flac_bound::{FlacEncoder, FlacEncoderState};

#[derive(Debug)]
enum Event {
    Buf(Vec<i32>),
    Finish,
}

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Could not get default device");

    match device.name() {
        Ok(name) => {
            println!("Attaching too: {}", name)
        }
        Err(e) => {
            eprintln!("Could not get name of attached device: Error: {}", e)
        }
    }

    // Get input audio device.
    let config = device.default_input_config().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let tx_1 = tx.clone();
    let stream = device
        // build an input audio stream.
        .build_input_stream(
            &config.config(),
            // callback for audio, data is 32 bit floats in range [-1, 1] interleaved.
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                let mut buf = Vec::with_capacity(data.len() + 1);
                for &i in data {
                    buf.push((i * 32_767_f32) as i32);
                }
                // I get this audio buffer from another thread,
                // and libflac isn't threadsafe, send data back over channel.
                tx.send(Event::Buf(buf)).unwrap();
            },
            // currently prints nothing since it never errors.
            move |err| {
                eprintln!("{}", err);
            },
        )
        .unwrap();

    // Start the above stream with the attached callback.
    stream.play().unwrap();

    // Spawn a thread with the flac encoder
    let t = std::thread::spawn(move || {
        let mut enc = FlacEncoder::new()
            .unwrap()
            .verify(true)
            .channels(2)
            .bits_per_sample(16)
            .sample_rate(config.sample_rate().0)
            .init_file(&PathBuf::from("flac.flac"))
            .unwrap();
        while let Ok(event) = rx.recv() {
            // for every buffer in the channel, encode the data.
            match event {
                Event::Buf(v) => {
                    enc.process_interleaved(&v, (v.len() / usize::from(config.channels())) as u32)
                        .unwrap();
                }
                Event::Finish => break,
            }
        }
        println!("all done");
        enc.finish().unwrap();
    });
    std::thread::sleep_ms(1000 * 10);
    tx_1.send(Event::Finish).unwrap();
    t.join().unwrap();
}
