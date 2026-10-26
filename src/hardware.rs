//! Sistem donanım bilgisi: sysinfo + (varsa) NVIDIA NVML.
//!
//! `HardwareInfo` bir gpui Global'i; saniyelik tick içinde hafif alanlar
//! (CPU kullanımı, RAM) güncellenir, ağır alanlar (komponent sıcaklıkları)
//! seyrek güncellenir.

use nvml_wrapper::Nvml;
use std::time::Instant;
use sysinfo::{Components, Disks, Networks, System};

#[derive(Debug, Default)]
pub struct HardwareInfo {
    pub sys: System,
    pub components: Components,
    pub disks: Disks,
    pub networks: Networks,
    pub nvml: Option<Nvml>,
    pub tick: u32,
    pub nvml_error: Option<String>,
    /// ağ hızı ölçümü için son refresh zamanı
    pub net_last: Option<Instant>,
    /// ölçülen indirme hızları: interface -> bayt/sn
    pub net_rx_speed: Vec<(String, f32)>,
    /// ölçülen yükleme hızları: interface -> bayt/sn
    pub net_tx_speed: Vec<(String, f32)>,
}

impl gpui_kit::Global for HardwareInfo {}

impl HardwareInfo {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        // komponent ve disk listeleri başta bir kez kurulur
        let components = Components::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let (nvml, nvml_error) = match Nvml::init() {
            Ok(nvml) => (Some(nvml), None),
            Err(e) => (None, Some(e.to_string())),
        };

        Self {
            sys,
            components,
            disks,
            networks,
            nvml,
            tick: 0,
            nvml_error,
            net_last: None,
            net_rx_speed: Vec::new(),
            net_tx_speed: Vec::new(),
        }
    }

    /// Saniyelik tick: CPU/RAM kullanımı her saniye, komponentler 5 saniyede bir.
    pub fn refresh_light(&mut self) {
        self.tick += 1;
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        if self.tick.is_multiple_of(5) {
            self.components.refresh(true);
        }
        if self.tick.is_multiple_of(60) {
            self.disks.refresh(true);
        }
        self.refresh_network();
    }

    /// Ağ: refresh + geçen süreye göre indirme/yükleme hızlarını ölçer.
    fn refresh_network(&mut self) {
        self.networks.refresh(true);
        let dt = match self.net_last {
            Some(last) => last.elapsed().as_secs_f32(),
            None => 0.,
        };
        self.net_last = Some(Instant::now());

        if dt <= 0. {
            return;
        }
        self.net_rx_speed = self
            .networks
            .list()
            .iter()
            .map(|(name, d)| (name.clone(), d.received() as f32 / dt))
            .collect();
        self.net_tx_speed = self
            .networks
            .list()
            .iter()
            .map(|(name, d)| (name.clone(), d.transmitted() as f32 / dt))
            .collect();
    }

    /// Toplam (indirme hızına göre) bayt/sn.
    pub fn net_download_speed_bps(&self) -> f32 {
        self.net_rx_speed.iter().map(|(_, v)| *v).sum()
    }

    /// Toplam yükleme hızı bayt/sn.
    pub fn net_upload_speed_bps(&self) -> f32 {
        self.net_tx_speed.iter().map(|(_, v)| *v).sum()
    }

    /// Açılımdan bu yana toplam indirilen/yüklenen bayt.
    pub fn net_total(&self) -> (u64, u64) {
        let mut rx = 0u64;
        let mut tx = 0u64;
        for d in self.networks.list().values() {
            rx += d.total_received();
            tx += d.total_transmitted();
        }
        (rx, tx)
    }

    // --- CPU ---

    pub fn cpu_name(&self) -> String {
        self.sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Bilinmiyor".into())
    }

    pub fn cpu_usage(&self) -> f32 {
        self.sys.global_cpu_usage()
    }

    pub fn cpu_temp(&self) -> Option<f32> {
        self.find_temp(&["cpu", "cpu package", "core", "package"])
    }

    pub fn cpu_freq_mhz(&self) -> u64 {
        self.sys.cpus().first().map(|c| c.frequency()).unwrap_or(0)
    }

    pub fn cpu_core_usage(&self) -> Vec<f32> {
        self.sys.cpus().iter().map(|c| c.cpu_usage()).collect()
    }

    // --- RAM ---

    pub fn ram_total_gib(&self) -> f32 {
        bytes_to_gib(self.sys.total_memory())
    }

    pub fn ram_used_gib(&self) -> f32 {
        bytes_to_gib(self.sys.used_memory())
    }

    pub fn ram_percent(&self) -> f32 {
        let total = self.sys.total_memory();
        if total == 0 {
            return 0.;
        }
        self.sys.used_memory() as f32 / total as f32 * 100.
    }

    // --- GPU (NVML / NVIDIA; yoksa None) ---

    pub fn gpu_info(&self) -> Option<GpuInfo> {
        let nvml = self.nvml.as_ref()?;
        let dev = nvml.device_by_index(0).ok()?;
        Some(GpuInfo {
            name: dev.name().ok().unwrap_or_else(|| "GPU".into()),
            usage: dev.utilization_rates().ok().map(|u| u.gpu as f32),
            temp: dev
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .ok()
                .map(|t| t as f32),
            fan_percent: dev.fan_speed(0).ok().map(|p| p as f32),
            memory_used_mib: dev
                .memory_info()
                .ok()
                .map(|m| m.used as f32 / 1024. / 1024.),
            memory_total_mib: dev
                .memory_info()
                .ok()
                .map(|m| m.total as f32 / 1024. / 1024.),
            power_watts: dev.power_usage().ok().map(|mw| mw as f32 / 1000.),
        })
    }

    // --- Sıcaklıklar (sysinfo komponentleri) ---

    fn find_temp(&self, needles: &[&str]) -> Option<f32> {
        for c in self.components.list() {
            let label = c.label().to_lowercase();
            if needles.iter().any(|n| label.contains(n)) {
                if let Some(t) = c.temperature() {
                    if t > 0. && t < 150. {
                        return Some(t);
                    }
                }
            }
        }
        None
    }

    /// Tüm okunabilen komponent sıcaklıkları (geçersiz olanlar atılır).
    pub fn all_temps(&self) -> Vec<(String, f32)> {
        self.components
            .list()
            .iter()
            .filter_map(|c| {
                let t = match c.temperature() {
                    Some(t) if t > 0. && t < 150. => t,
                    _ => return None,
                };
                Some((c.label().to_string(), t))
            })
            .collect()
    }

    // --- Sistem geneli ---

    pub fn os_version(&self) -> String {
        System::long_os_version().unwrap_or_else(|| "Bilinmiyor".into())
    }

    pub fn kernel_version(&self) -> String {
        System::kernel_version().unwrap_or_else(|| "—".into())
    }

    pub fn host_name(&self) -> String {
        System::host_name().unwrap_or_else(|| "—".into())
    }

    pub fn uptime_str(&self) -> String {
        let s = System::uptime();
        format!("{}h {}m", s / 3600, (s % 3600) / 60)
    }
}

pub struct GpuInfo {
    pub name: String,
    pub usage: Option<f32>,
    pub temp: Option<f32>,
    pub fan_percent: Option<f32>,
    pub memory_used_mib: Option<f32>,
    pub memory_total_mib: Option<f32>,
    pub power_watts: Option<f32>,
}

fn bytes_to_gib(b: u64) -> f32 {
    b as f32 / 1024. / 1024. / 1024.
}
