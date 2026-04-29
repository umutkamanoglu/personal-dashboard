use lazy_static::lazy_static;
use nvml_wrapper::Nvml;
use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Disks, Networks, System};

// Global bir ağ nesnesi oluşturuyoruz ki veriler her istekte sıfırlanmasın
lazy_static! {
    static ref GLOBAL_NETWORKS: Mutex<Networks> = Mutex::new(Networks::new_with_refreshed_list());
}

#[derive(Serialize)]
pub struct SystemData {
    pub cpu_usage: f32,
    pub ram_used: u64,
    pub ram_total: u64,
    pub gpu_name: Option<String>,
    pub gpu_usage: Option<u32>,
    pub gpu_temp: Option<u32>,
    pub gpu_mem_used: Option<u64>,
    pub disks: Vec<(String, u64, u64)>,
    pub net_in: u64,
    pub net_out: u64,
    pub os: String,
    pub uptime: u64,
}

pub fn get_full_system_info() -> SystemData {
    let mut sys = System::new_all();
    sys.refresh_all();

    // --- AĞ VERİSİ HESAPLAMA ---
    let mut net_in = 0;
    let mut net_out = 0;

    if let Ok(mut networks) = GLOBAL_NETWORKS.lock() {
        networks.refresh(); // Sadece verileri tazele, listeyi değil
        for (_, data) in networks.iter() {
            net_in += data.received();
            net_out += data.transmitted();
        }
    }

    // --- GPU BÖLÜMÜ ---
    let mut gpu_name = None;
    let mut gpu_usage = None;
    let mut gpu_temp = None;
    let mut gpu_mem_used = None;

    if let Ok(nvml) = Nvml::init() {
        if let Ok(device) = nvml.device_by_index(0) {
            gpu_name = device.name().ok();
            gpu_usage = device.utilization_rates().ok().map(|u| u.gpu);
            gpu_temp = device
                .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                .ok();
            gpu_mem_used = device.memory_info().ok().map(|m| m.used);
        }
    }

    let disks = Disks::new_with_refreshed_list();

    SystemData {
        cpu_usage: sys.global_cpu_info().cpu_usage(),
        ram_used: sys.used_memory(),
        ram_total: sys.total_memory(),
        gpu_name,
        gpu_usage,
        gpu_temp,
        gpu_mem_used,
        disks: disks
            .iter()
            .map(|d| {
                (
                    d.mount_point().to_string_lossy().into(),
                    d.total_space(),
                    d.available_space(),
                )
            })
            .collect(),
        net_in,
        net_out,
        os: System::long_os_version().unwrap_or_default(),
        uptime: System::uptime(),
    }
}
