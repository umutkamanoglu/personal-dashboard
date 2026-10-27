//! CAVA tarzı ses görselleştirici: WASAPI loopback ile sistem sesi yakalanır,
//! FFT ile frekans bantlarına bölünür, bar yükseklikleri gpui'de çizilir.
//!
//! `VisualizerState` bir gpui Global'i: arka plan iş parçacığı sürekli
//! yakalama + FFT yapıp `bars`'ı güncelller (0..=1 normalized); kart tikte
//! okuyup çizer.

use std::f32::consts::PI;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use rustfft::{num_complex::Complex, FftPlanner};

/// FFT pencere boyutu (örnek; 48 kHz'de ~43 ms).
const FFT_SIZE: usize = 2048;
/// Kaç frekans bandı hesaplanır (kart kendi sayısını seçer).
pub const MAX_BANDS: usize = 64;
/// Arka plan döngüsünün uyanma aralığı.
const POLL_MS: u64 = 15;

#[derive(Default)]
pub struct VisualizerState {
    /// 0..=1 normalized band seviyeleri; ses yoksa hepsi 0
    pub bars: Arc<Mutex<Vec<f32>>>,
    /// barlar güncel mi değil, interpolasyon için gösterim kopyası
    pub display: Arc<Mutex<Vec<f32>>>,
    /// akıcı tick'te sessizlik bildirimini seyreltmek için sayaç
    pub ticks: AtomicUsize,
    /// son hatada arka plan iş parçacığının yazdığı bilgi
    pub error: Arc<Mutex<Option<String>>>,
}

impl gpui_kit::Global for VisualizerState {}

impl VisualizerState {
    /// Barların anlık kopyası (renderken kilit tutmamak için).
    pub fn snapshot(&self, count: usize) -> Vec<f32> {
        let all = self
            .bars
            .lock()
            .map(|b| b.clone())
            .unwrap_or_else(|_| vec![0.; MAX_BANDS]);
        let count = count.min(all.len());
        if count == 0 {
            return Vec::new();
        }
        let src = all.len();
        (0..count).map(|i| all[i * src / count]).collect()
    }

    pub fn error_text(&self) -> Option<String> {
        self.error.lock().ok().and_then(|e| e.clone())
    }

    /// Sessizlikte akıcı tick'i seyreltir: 8 tikte 1 true döner (~4 Hz).
    pub fn slow_tick(&self) -> bool {
        self.ticks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .is_multiple_of(8)
    }

    /// Çizim zeminini yumuşatan interpolasyonlu kopya (~60 fps tik için
    /// ayarlı): hedefe yukarı hızlı (0.40), aşağı akıcı (0.16) yakınsama;
    /// ham veri 15 ms'de değişse çizim her karede ara değer üretir.
    pub fn display_snapshot(&self, count: usize) -> Vec<f32> {
        let target = self.snapshot(count);
        let mut disp = match self.display.lock() {
            Ok(mut d) => {
                if d.len() != target.len() {
                    *d = vec![0.; target.len()];
                }
                for (i, t) in target.iter().enumerate() {
                    let cur = d[i];
                    let k = if *t > cur { 0.40 } else { 0.16 };
                    d[i] += (t - cur) * k;
                }
                d.clone()
            }
            Err(_) => self.snapshot(count),
        };
        // köşe durumu: uzunluk yine tutmadıysa hedefin kendisi
        if disp.len() != target.len() {
            disp = target;
        }
        disp
    }
}

/// Uygulama açılışında çağrılır: arka plan yakalama iş parçacığını başlatır.
pub fn start() -> VisualizerState {
    let state = VisualizerState {
        bars: Arc::new(Mutex::new(vec![0.; MAX_BANDS])),
        display: Arc::new(Mutex::new(vec![0.; MAX_BANDS])),
        ticks: AtomicUsize::new(0),
        error: Arc::new(Mutex::new(None)),
    };
    let bars = state.bars.clone();
    let err = state.error.clone();
    let _ = std::thread::Builder::new()
        .name("audio-visualizer".into())
        .spawn(move || {
            loop {
                match capture_loop(bars.clone(), err.clone()) {
                    Ok(()) => break,
                    Err(e) => {
                        let msg = e.to_string();
                        if let Ok(mut slot) = err.lock() {
                            *slot = Some(msg.clone());
                        }
                        // cihaz değişikliği vs. olabilir; biraz bekleyip tekrar dene
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        if let Ok(mut slot) = err.lock() {
                            *slot = None;
                        }
                    }
                }
            }
        });
    state
}

/// Ham FFT sonucunu band'a çevirir (logaritmik 30 Hz – 16 kHz).
fn magnitudes_to_bands(bin_mags: &[f32], sample_rate: f32, out: &mut [f32]) {
    const F_MIN: f32 = 30.;
    const F_MAX: f32 = 16_000.;
    let half = bin_mags.len() as f32;
    if half <= 0. {
        return;
    }
    let bin_hz = sample_rate / 2. / half;
    let n = out.len();
    if n == 0 {
        return;
    }
    for (i, slot) in out.iter_mut().enumerate() {
        let lo = F_MIN * (F_MAX / F_MIN).powf(i as f32 / n as f32);
        let hi = F_MIN * (F_MAX / F_MIN).powf((i + 1) as f32 / n as f32);
        let mut b0 = ((lo / bin_hz).floor() as usize).max(1);
        let b1 = (((hi / bin_hz).ceil() as usize) + 1).min(bin_mags.len());
        if b1 <= b0 {
            b0 = b1.saturating_sub(1);
        }
        let mut sum = 0.;
        let mut cnt = 0usize;
        for bin in bin_mags.iter().take(b1).skip(b0) {
            sum += *bin;
            cnt += 1;
        }
        let avg = if cnt > 0 { sum / cnt as f32 } else { 0. };
        // dB benzeri ölçek: -70 dB -> 0, +10 dB -> 1
        let db = 20. * (avg + 1e-9).log10();
        *slot = ((db + 70.) / 80.).clamp(0., 1.);
    }
}

fn hann(i: usize, n: usize) -> f32 {
    0.5 - 0.5 * ((2. * PI * i as f32) / (n as f32 - 1.)).cos()
}

/// WASAPI loopback yakalama. Cihaz kalkarsa hata döner; üst katman yeniden dener.
fn capture_loop(
    bars: Arc<Mutex<Vec<f32>>>,
    err: Arc<Mutex<Option<String>>>,
) -> windows::core::Result<()> {
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator,
        AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, WAVEFORMATEXTENSIBLE,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    };

    // windows crate bu sabitleri üretmiyor; burada tanımlıyoruz
    const WAVE_FORMAT_EXTENSIBLE: u16 = 0xFFFE;
    let clsid_mmdevice = windows::core::GUID::from_u128(0xbcde_0395_e52f_467c_8e3d_c457_9291_692e);
    // SubFormat GUID'i: KSDATAFORMAT_SUBTYPE_IEEE_FLOAT
    let ieee_float = windows::core::GUID::from_u128(0x0000_0003_0000_0010_8000_00aa_0038_9b71);

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;

        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&clsid_mmdevice, None, CLSCTX_ALL)?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

        let fmt_ptr = client.GetMixFormat()?;
        let fmt = &*fmt_ptr;
        let channels = fmt.nChannels as usize;
        let sample_rate = fmt.nSamplesPerSec as f32;
        let bits = fmt.wBitsPerSample;

        // WAVEFORMATEXTENSIBLE packed; alanları hizasız oku
        let tag = std::ptr::addr_of!(fmt.wFormatTag).read_unaligned();
        let is_float = if tag == WAVE_FORMAT_EXTENSIBLE {
            let ext = fmt_ptr as *const WAVEFORMATEXTENSIBLE;
            std::ptr::addr_of!((*ext).SubFormat).read_unaligned() == ieee_float
        } else {
            tag == 3 // WAVE_FORMAT_IEEE_FLOAT
        };
        if !is_float || bits != 32 {
            let msg =
                format!("mix format desteklenmiyor (tag={tag:#x}, bits={bits}, float={is_float})");
            if let Ok(mut s) = err.lock() {
                *s = Some(msg);
            }
            return Ok(()); // sessizce çık; barlar 0 kalır
        }

        client.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_LOOPBACK,
            0,
            0,
            fmt_ptr,
            None,
        )?;
        let capture: IAudioCaptureClient = client.GetService()?;
        client.Start()?;

        // FFT planı
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let mut scratch: Vec<Complex<f32>> = vec![Complex::new(0., 0.); FFT_SIZE];
        let mut bands = vec![0f32; MAX_BANDS];
        let mut write_buf = vec![0f32; MAX_BANDS];
        let mags_len = FFT_SIZE / 2;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(POLL_MS));

            // ring: her yakalamada mono (kanal ortalaması) örnekleri ekler;
            // FFT_SIZE'dan fazlası baştan atılır
            let mut frames_mono: Vec<f32> = Vec::new();
            loop {
                let available = capture.GetNextPacketSize()?;
                if available == 0 {
                    break;
                }
                let mut data: *mut u8 = std::ptr::null_mut();
                let mut frames: u32 = 0;
                let mut flags: u32 = 0;
                let mut pos: u64 = 0;
                let mut qpc: u64 = 0;
                capture.GetBuffer(
                    &mut data,
                    &mut frames,
                    &mut flags,
                    Some(&mut pos),
                    Some(&mut qpc),
                )?;
                if frames > 0 {
                    let bytes = std::slice::from_raw_parts(data, frames as usize * channels * 4);
                    for f in 0..frames as usize {
                        let mut sum = 0f32;
                        for c in 0..channels {
                            let off = (f * channels + c) * 4;
                            let v = f32::from_le_bytes([
                                bytes[off],
                                bytes[off + 1],
                                bytes[off + 2],
                                bytes[off + 3],
                            ]);
                            sum += v;
                        }
                        frames_mono.push(sum / channels as f32);
                    }
                }
                capture.ReleaseBuffer(frames)?;
            }

            if frames_mono.is_empty() {
                // sessizlik: barlar yavaşça sönsün
                decay(&mut bands, 0.94);
            } else {
                // son FFT_SIZE örneği
                let start = frames_mono.len().saturating_sub(FFT_SIZE);
                let chunk = &frames_mono[start..];
                for (i, s) in chunk.iter().take(FFT_SIZE).enumerate() {
                    scratch[i] = Complex::new(*s * hann(i, FFT_SIZE), 0.);
                }
                for s in scratch.iter_mut().skip(chunk.len().min(FFT_SIZE)) {
                    *s = Complex::new(0., 0.);
                }
                fft.process(&mut scratch);
                let mut mags32 = vec![0f32; mags_len];
                for (mi, s) in scratch.iter().take(mags_len).enumerate() {
                    mags32[mi] = s.norm();
                }
                // normalize
                let max_mag = mags32.iter().cloned().fold(0., f32::max).max(1e-6);
                for m in mags32.iter_mut() {
                    *m /= max_mag;
                }
                magnitudes_to_bands(&mags32, sample_rate, &mut write_buf);
                // smoothing: hızlı saldırı, yumuşak sönüş
                for i in 0..MAX_BANDS {
                    bands[i] = write_buf[i].max(bands[i] * 0.88);
                }
            }

            if let Ok(mut lock) = bars.lock() {
                *lock = bands.clone();
            }
        }
    }
}

fn decay(bands: &mut [f32], factor: f32) {
    for b in bands.iter_mut() {
        *b *= factor;
    }
}
