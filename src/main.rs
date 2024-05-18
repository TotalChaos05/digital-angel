use async_timers::{OneshotTimer, PeriodicTimer};
use rodio::Sink;
use std::fmt::Error;
use std::io::BufReader;
use tokio;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::Duration;
#[tokio::main]
async fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    sink.set_volume(0.15);

    let (tx, rx) = channel(32);
    let join_handle = tokio::task::spawn(heart(rx, sink));
    tx.send(60).await.unwrap();
    loop {
        let mut line = String::new();
        let b1 = std::io::stdin().read_line(&mut line).unwrap();

        let t: String = line.chars().filter(|c| c.is_digit(10)).collect();
        if t != "" {tx.send(t.trim().parse().unwrap()).await.unwrap();}

    }
}

async fn heart(mut rx: Receiver<u32>, mut sink: Sink) -> Result<(), Error> {
    let mut bpm: u32 = 60;
    let mut primary = PeriodicTimer::stopped();
    let mut beat:u8 = 0;
    primary.start(Duration::from_millis(bpm as u64));
    let mut stop_delay = OneshotTimer::expired();
    loop {
        tokio::select! {
                    i = rx.recv() => {
                        bpm = i.unwrap().clamp(0,250);
                        primary.stop();
                        primary.start(Duration::from_millis(((60000/bpm)/3) as u64));

                    }
                    _ = primary.tick() => {
                        beat = (beat+1)%3;
                        let file = match beat {
                            1 => std::fs::File::open("s1.wav").unwrap(),
                            2 => std::fs::File::open("s2.wav").unwrap(),
                            _ => std::fs::File::open("empty.wav").unwrap()
                        };

                        sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                    }
                    _ = stop_delay.tick() => {
                        primary.stop();
                        return Ok(())
                    }
                }
    }
}
