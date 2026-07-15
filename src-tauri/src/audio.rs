//! 錄音與 WAV 封裝（規格第 4 節）。
//! cpal 抓麥克風到記憶體 buffer（混為單聲道），停止時降取樣到 16kHz 並用 hound 封裝成 WAV bytes，
//! 全程不落地。Recorder 持有 cpal::Stream（!Send），僅在 controller 執行緒內使用。

use anyhow::{anyhow, bail, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

const TARGET_RATE: u32 = 16_000;
const MAX_WAV_BYTES: usize = 25 * 1024 * 1024; // Groq 25MB 上限

pub struct Recorder {
    stream: cpal::Stream,
    buf: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

/// 開始錄音：開啟預設輸入裝置的串流，PCM 持續寫入記憶體 buffer。
/// `level` 由本函式的音訊 callback 持續更新為當前音量（f32 bits），供波形疊加視窗讀取。
pub fn start(level: Arc<AtomicU32>) -> Result<Recorder> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow!("找不到麥克風輸入裝置"))?;
    let supported = device
        .default_input_config()
        .map_err(|e| anyhow!("取得輸入設定失敗: {e}"))?;
    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels() as usize;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();

    let buf = Arc::new(Mutex::new(Vec::<f32>::new()));
    let buf_cb = buf.clone();
    let level_cb = level.clone();
    let err_fn = |e| eprintln!("[audio] 串流錯誤: {e}");

    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _: &_| push_mono(&buf_cb, data, channels, &level_cb),
            err_fn,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _: &_| {
                let f: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                push_mono(&buf_cb, &f, channels, &level_cb);
            },
            err_fn,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _: &_| {
                let f: Vec<f32> = data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0).collect();
                push_mono(&buf_cb, &f, channels, &level_cb);
            },
            err_fn,
            None,
        ),
        other => bail!("不支援的取樣格式: {other:?}"),
    }
    .map_err(|e| anyhow!("建立輸入串流失敗: {e}"))?;

    stream.play().map_err(|e| anyhow!("啟動串流失敗: {e}"))?;
    println!("[audio] 開始錄音（{sample_rate} Hz, {channels} ch → 16kHz mono）");
    Ok(Recorder {
        stream,
        buf,
        sample_rate,
    })
}

impl Recorder {
    /// 停止錄音並回傳 16kHz 單聲道 16-bit WAV bytes（記憶體內）。
    pub fn stop_to_wav(self) -> Result<Vec<u8>> {
        let Recorder {
            stream,
            buf,
            sample_rate,
        } = self;
        drop(stream); // 停止擷取

        let samples = std::mem::take(&mut *buf.lock().unwrap());
        if samples.is_empty() {
            bail!("沒有錄到任何音訊");
        }
        let resampled = resample_linear(&samples, sample_rate, TARGET_RATE);
        let wav = encode_wav_16k(&resampled)?;
        if wav.len() > MAX_WAV_BYTES {
            bail!(
                "錄音超過 25MB 上限（{} bytes），請縮短單次錄音時間",
                wav.len()
            );
        }
        println!("[audio] 停止錄音，WAV {} bytes", wav.len());
        Ok(wav)
    }
}

/// 把交錯的多聲道資料混成單聲道後追加到 buffer，並更新當前音量（RMS，含衰減平滑）。
fn push_mono(buf: &Arc<Mutex<Vec<f32>>>, data: &[f32], channels: usize, level: &AtomicU32) {
    let mut b = buf.lock().unwrap();
    let start = b.len();
    if channels <= 1 {
        b.extend_from_slice(data);
    } else {
        for frame in data.chunks(channels) {
            let sum: f32 = frame.iter().sum();
            b.push(sum / channels as f32);
        }
    }
    // 以本批新樣本算 RMS，再與前值取衰減最大值，讓波形不會瞬間歸零。
    let new = &b[start..];
    if !new.is_empty() {
        let sum_sq: f32 = new.iter().map(|s| s * s).sum();
        let rms = (sum_sq / new.len() as f32).sqrt();
        let prev = f32::from_bits(level.load(Ordering::Relaxed));
        let smoothed = rms.max(prev * 0.80);
        level.store(smoothed.to_bits(), Ordering::Relaxed);
    }
}

/// 線性內插降/升取樣。MVP 用簡單內插即可（規格未要求高品質重採樣）。
fn resample_linear(input: &[f32], in_rate: u32, out_rate: u32) -> Vec<f32> {
    if in_rate == out_rate || input.is_empty() {
        return input.to_vec();
    }
    let ratio = out_rate as f64 / in_rate as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src.floor() as usize;
        let frac = (src - idx as f64) as f32;
        let a = input.get(idx).copied().unwrap_or(0.0);
        let b = input.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

/// f32 [-1,1] → 16-bit PCM mono 16kHz WAV bytes。
fn encode_wav_16k(samples: &[f32]) -> Result<Vec<u8>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|e| anyhow!("WAV 初始化失敗: {e}"))?;
        for &s in samples {
            let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer
                .write_sample(v)
                .map_err(|e| anyhow!("寫入 WAV 失敗: {e}"))?;
        }
        writer.finalize().map_err(|e| anyhow!("WAV finalize 失敗: {e}"))?;
    }
    Ok(cursor.into_inner())
}
