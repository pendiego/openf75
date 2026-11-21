//! Audio-reactive key lighting: loopback capture of the default output, 15 log bands, keys drawn as bars.

use std::collections::VecDeque;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

use crate::device::Device;
use crate::layout;
use crate::proto;

pub type Shared = Arc<Mutex<Option<Device>>>;

const N: usize = 2048;
const BANDS: usize = 15;
const RING_CAP: usize = 2048;
const FRAME: Duration = Duration::from_millis(33);

pub struct AudioViz {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
    dev: Shared,
}

impl AudioViz {
    /// Starts streaming the default output's spectrum to the keyboard LEDs.
    pub fn start(dev: Shared, gain: f32) -> Result<AudioViz> {
        {
            let g = dev
                .lock()
                .map_err(|_| anyhow!("estado do teclado corrompido"))?;
            g.as_ref()
                .ok_or_else(|| anyhow!("teclado não conectado"))?
                .writable()?;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::channel::<Result<(), String>>();
        let (s, d) = (stop.clone(), dev.clone());
        let thread = thread::spawn(move || run(d, gain, s, tx));
        match rx.recv() {
            Ok(Ok(())) => Ok(AudioViz { stop, thread, dev }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(anyhow!(e))
            }
            Err(_) => Err(anyhow!("thread de áudio encerrou")),
        }
    }
    /// Stops the loop and restores the keyboard's configured LED effect.
    pub fn stop(self) -> Result<()> {
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.thread.join();
        let g = self
            .dev
            .lock()
            .map_err(|_| anyhow!("estado do teclado corrompido"))?;
        if let Some(d) = g.as_ref() {
            d.set_led(&d.led()?)?;
        }
        Ok(())
    }
}

/// Owns the capture stream and the render loop; reports startup success or failure first.
fn run(dev: Shared, gain: f32, stop: Arc<AtomicBool>, tx: mpsc::Sender<Result<(), String>>) {
    let ring = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(RING_CAP)));
    let stream = match capture(ring.clone()) {
        Ok((s, rate)) => {
            let _ = tx.send(Ok(()));
            (s, rate)
        }
        Err(e) => {
            let _ = tx.send(Err(e.to_string()));
            return;
        }
    };
    let (_stream, rate) = stream;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(N);
    let window: Vec<f32> = (0..N)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / N as f32).cos())
        .collect();
    let bin_hz = rate / N as f32;
    let keys: Vec<_> = layout::KEYS.iter().filter(|k| k.rect != [0; 4]).collect();
    let mut level = [0f32; BANDS];

    while !stop.load(Ordering::SeqCst) {
        let mut buf: Vec<Complex<f32>> = {
            let r = match ring.lock() {
                Ok(r) => r,
                Err(_) => break,
            };
            let skip = r.len().saturating_sub(N);
            let pad = N - (r.len() - skip);
            (0..N)
                .map(|i| {
                    let s = if i < pad { 0.0 } else { r[skip + i - pad] };
                    Complex::new(s * window[i], 0.0)
                })
                .collect()
        };
        fft.process(&mut buf);

        for b in 0..BANDS {
            let lo = 60.0 * 200f32.powf(b as f32 / BANDS as f32);
            let hi = 60.0 * 200f32.powf((b + 1) as f32 / BANDS as f32);
            let mut peak = 0f32;
            for k in 1..N / 2 {
                let f = k as f32 * bin_hz;
                if f >= lo && f < hi {
                    peak = peak.max(buf[k].norm());
                }
            }
            if peak == 0.0 {
                let k = ((lo * hi).sqrt() / bin_hz)
                    .round()
                    .clamp(1.0, (N / 2 - 1) as f32) as usize;
                peak = buf[k].norm();
            }
            let m = peak * 2.0 / N as f32;
            let db = 20.0 * (m + 1e-9).log10();
            let lvl = ((db + 60.0) / 60.0 * gain).clamp(0.0, 1.0);
            level[b] = lvl.max(level[b] * 0.85);
        }

        let frame_keys: Vec<(u8, [u8; 3])> = keys
            .iter()
            .map(|k| {
                let band = ((((k.rect[0] + k.rect[2]) as f32 / 2.0) - 30.0) / 600.0 * BANDS as f32)
                    .clamp(0.0, (BANDS - 1) as f32) as usize;
                let y = k.rect[1];
                let row = match y {
                    0..=60 => 5,
                    61..=100 => 4,
                    101..=140 => 3,
                    141..=180 => 2,
                    181..=215 => 1,
                    _ => 0,
                };
                let colour = match row {
                    0 | 1 => [0, 255, 0],
                    2 | 3 => [255, 255, 0],
                    _ => [255, 0, 0],
                };
                let lit = level[band] >= (row + 1) as f32 / 6.0;
                (k.matrix, if lit { colour } else { [0, 0, 0] })
            })
            .collect();
        let frame = proto::realtime_frame(&frame_keys);

        let g = match dev.lock() {
            Ok(g) => g,
            Err(_) => break,
        };
        let Some(d) = g.as_ref() else { break };
        if d.realtime(&frame).is_err() {
            break;
        }
        drop(g);
        thread::sleep(FRAME);
    }
}

/// Opens a capture of the system output and returns the stream and its sample rate.
fn capture(ring: Arc<Mutex<VecDeque<f32>>>) -> Result<(cpal::Stream, f32)> {
    let (dev, cfg) = source(&cpal::default_host())?;
    let rate = cfg.sample_rate().0 as f32;
    let channels = cfg.channels() as usize;
    let err = |e| eprintln!("áudio: {e}");
    let stream = match cfg.sample_format() {
        cpal::SampleFormat::F32 => build::<f32>(&dev, &cfg.into(), channels, ring, err)?,
        cpal::SampleFormat::I16 => build::<i16>(&dev, &cfg.into(), channels, ring, err)?,
        cpal::SampleFormat::U16 => build::<u16>(&dev, &cfg.into(), channels, ring, err)?,
        f => return Err(anyhow!("formato de áudio não suportado: {f:?}")),
    };
    stream.play()?;
    Ok((stream, rate))
}

/// WASAPI loopback: the default output device, opened as an input stream.
#[cfg(windows)]
fn source(host: &cpal::Host) -> Result<(cpal::Device, cpal::SupportedStreamConfig)> {
    let out = host
        .default_output_device()
        .ok_or_else(|| anyhow!("nenhuma saída de áudio padrão"))?;
    let cfg = out.default_output_config()?;
    Ok((out, cfg))
}

/// ALSA via PulseAudio/PipeWire: prefer a sink monitor, else the default input routed to it.
#[cfg(target_os = "linux")]
fn source(host: &cpal::Host) -> Result<(cpal::Device, cpal::SupportedStreamConfig)> {
    route_default_input_to_monitor();
    let dev = host
        .input_devices()?
        .find(|d| d.name().is_ok_and(|n| n.to_lowercase().contains("monitor")))
        .or_else(|| host.default_input_device())
        .ok_or_else(|| anyhow!("nenhuma entrada de áudio padrão"))?;
    let cfg = dev.default_input_config()?;
    Ok((dev, cfg))
}

/// CoreAudio cannot capture system output; a virtual loopback input (BlackHole and similar) is required.
#[cfg(target_os = "macos")]
fn source(host: &cpal::Host) -> Result<(cpal::Device, cpal::SupportedStreamConfig)> {
    let dev = host
        .input_devices()?
        .find(|d| {
            d.name().is_ok_and(|n| {
                let n = n.to_lowercase();
                ["blackhole", "loopback", "soundflower"].iter().any(|k| n.contains(k))
            })
        })
        .ok_or_else(|| {
            anyhow!(
                "o macOS não captura o som do sistema: instale um dispositivo de loopback como o BlackHole e direcione a saída para ele"
            )
        })?;
    let cfg = dev.default_input_config()?;
    Ok((dev, cfg))
}

/// Points the ALSA default input at the default sink monitor, unless the user already set these.
/// Runs once; set_var can race only with C getenv calls on other threads, a residual risk accepted here.
#[cfg(target_os = "linux")]
fn route_default_input_to_monitor() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("PULSE_SOURCE").is_none() {
            std::env::set_var("PULSE_SOURCE", "@DEFAULT_MONITOR@");
        }
        if std::env::var_os("PIPEWIRE_PROPS").is_none() {
            std::env::set_var("PIPEWIRE_PROPS", "{ stream.capture.sink = true }");
        }
    });
}

fn build<T>(
    dev: &cpal::Device,
    cfg: &cpal::StreamConfig,
    channels: usize,
    ring: Arc<Mutex<VecDeque<f32>>>,
    err: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let s = dev.build_input_stream(
        cfg,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            let Ok(mut r) = ring.lock() else { return };
            for frame in data.chunks(channels) {
                let mono =
                    frame.iter().map(|&x| x.to_sample::<f32>()).sum::<f32>() / channels as f32;
                if r.len() == RING_CAP {
                    r.pop_front();
                }
                r.push_back(mono);
            }
        },
        err,
        None,
    )?;
    Ok(s)
}
