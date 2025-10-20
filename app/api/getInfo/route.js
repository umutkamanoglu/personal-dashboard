// app/api/system-info/route.ts
import { NextResponse } from "next/server"
import si from "systeminformation"
import { exec } from "child_process"
import { promisify } from "util"

const execPromise = promisify(exec)

// Cache mekanizması - yavaş değişen veriler için
let cachedStaticData = null
let lastCacheTime = 0
const CACHE_DURATION = 60000 // 60 saniye

// NVIDIA GPU bilgilerini al
async function getNvidiaGPUInfo() {
    try {
        const { stdout } = await execPromise(
            "nvidia-smi --query-gpu=temperature.gpu,utilization.gpu,memory.used,memory.total --format=csv,noheader,nounits"
        )

        const values = stdout
            .trim()
            .split(",")
            .map(v => v.trim())
        return {
            temperature: values[0] || "N/A",
            usage: values[1] || "N/A",
            memoryUsed: values[2] ? (parseInt(values[2]) / 1024).toFixed(2) : "N/A",
            memoryTotal: values[3] ? (parseInt(values[3]) / 1024).toFixed(2) : "N/A"
        }
    } catch (error) {
        return null
    }
}

// CPU sıcaklığını al - basit tahmin yöntemi
async function getCPUTempEstimate(cpuLoad) {
    // Yöntem 1: WMI ile dene
    try {
        const {
            stdout
        } = await execPromise(
            "wmic /namespace:\\\\root\\wmi PATH MSAcpi_ThermalZoneTemperature get CurrentTemperature /value",
            { timeout: 1500 }
        )

        const match = stdout.match(/CurrentTemperature=(\d+)/)
        if (match) {
            const temp = parseInt(match[1])
            const celsius = temp / 10 - 273.15
            if (celsius > 0 && celsius < 120) {
                return Math.round(celsius)
            }
        }
    } catch (error) {
        // Hata varsa devam et
    }

    // Yöntem 2: CPU yüküne göre tahmin et
    // Modern Intel/AMD CPU'lar idle'da ~35-45°C, yük altında ~60-80°C
    const baseTemp = 40
    const estimatedTemp = baseTemp + cpuLoad * 0.4
    return `~${Math.round(estimatedTemp)}°C`
}

// Statik verileri al (nadiren değişen)
async function getStaticData() {
    const now = Date.now()

    // Cache kontrolü
    if (cachedStaticData && now - lastCacheTime < CACHE_DURATION) {
        return cachedStaticData
    }

    const [cpu, osInfo, system, graphics] = await Promise.all([
        si.cpu(),
        si.osInfo(),
        si.system(),
        si.graphics()
    ])

    cachedStaticData = { cpu, osInfo, system, graphics }
    lastCacheTime = now

    return cachedStaticData
}

export async function GET() {
    try {
        const startTime = Date.now()

        // Statik ve dinamik verileri paralel olarak al
        const [
            staticData,
            cpuLoad,
            cpuTemp,
            mem,
            disk,
            networkStats,
            battery,
            nvidiaInfo
        ] = await Promise.all([
            getStaticData(),
            si.currentLoad(),
            si.cpuTemperature(),
            si.mem(),
            si.fsSize(),
            si.networkStats(),
            si.battery(),
            getNvidiaGPUInfo()
        ])

        const { cpu, osInfo, system, graphics } = staticData

        // CPU bilgileri
        const cpuLoadValue = Math.round(cpuLoad.currentLoad)
        const cpuTempValue =
            cpuTemp.main || (await getCPUTempEstimate(cpuLoadValue))

        const cpuInfo = {
            model: cpu.manufacturer + " " + cpu.brand,
            cores: cpu.cores,
            physicalCores: cpu.physicalCores,
            speed: cpu.speed + " GHz",
            usage: cpuLoadValue + "%",
            temperature:
                typeof cpuTempValue === "number" ? cpuTempValue + "°C" : cpuTempValue,
            temperatureMax: cpuTemp.max ? cpuTemp.max + "°C" : "N/A"
        }

        // RAM bilgileri
        const ramInfo = {
            total: (mem.total / 1024 ** 3).toFixed(2) + " GB",
            used: (mem.used / 1024 ** 3).toFixed(2) + " GB",
            free: (mem.free / 1024 ** 3).toFixed(2) + " GB",
            usagePercentage: Math.round((mem.used / mem.total) * 100) + "%"
        }

        // Disk bilgileri - sadece local diskler
        const diskInfo = disk
            .filter(d => d.type !== "cd" && d.type !== "removable")
            .map(d => ({
                name: d.fs,
                type: d.type,
                size: (d.size / 1024 ** 3).toFixed(2) + " GB",
                used: (d.used / 1024 ** 3).toFixed(2) + " GB",
                available: (d.available / 1024 ** 3).toFixed(2) + " GB",
                usagePercentage: Math.round(d.use) + "%",
                mount: d.mount
            }))

        // Ağ bilgileri - sadece aktif interface
        const networkInfo = networkStats
            .filter(net => net.operstate === "up")
            .map(net => ({
                interface: net.iface,
                download: (net.rx_sec / 1024 ** 2).toFixed(2) + " MB/s",
                upload: (net.tx_sec / 1024 ** 2).toFixed(2) + " MB/s",
                downloadTotal: (net.rx_bytes / 1024 ** 3).toFixed(2) + " GB",
                uploadTotal: (net.tx_bytes / 1024 ** 3).toFixed(2) + " GB"
            }))

        // GPU bilgileri - NVIDIA için gerçek veriler
        const gpuInfo = graphics.controllers.map((gpu, index) => {
            const isNvidia = gpu.vendor?.toLowerCase().includes("nvidia")

            return {
                model: gpu.model || gpu.name,
                vram: gpu.vram ? (gpu.vram / 1024).toFixed(2) + " GB" : "N/A",
                vramUsed:
                    isNvidia && nvidiaInfo
                        ? nvidiaInfo.memoryUsed + " GB"
                        : gpu.memoryUsed
                            ? (gpu.memoryUsed / 1024).toFixed(2) + " GB"
                            : "N/A",
                usage:
                    isNvidia && nvidiaInfo
                        ? nvidiaInfo.usage + "%"
                        : gpu.utilizationGpu
                            ? gpu.utilizationGpu + "%"
                            : "N/A",
                temperature:
                    isNvidia && nvidiaInfo
                        ? nvidiaInfo.temperature + "°C"
                        : gpu.temperatureGpu || "N/A",
                vendor: gpu.vendor
            }
        })

        // Ekstra bilgiler - minimal
        const extraInfo = {
            os: {
                platform: osInfo.platform,
                distro: osInfo.distro,
                hostname: osInfo.hostname
            },
            system: {
                manufacturer: system.manufacturer,
                model: system.model
            },
            battery: {
                hasBattery: battery.hasBattery,
                isCharging: battery.isCharging,
                percent: battery.percent + "%"
            }
        }

        const responseTime = Date.now() - startTime

        const systemData = {
            timestamp: new Date().toISOString(),
            responseTime: responseTime + "ms",
            cpu: cpuInfo,
            ram: ramInfo,
            disks: diskInfo,
            network: networkInfo,
            gpu: gpuInfo,
            extra: extraInfo
        }

        return NextResponse.json(systemData)
    } catch (error) {
        console.error("Sistem bilgileri alınırken hata:", error)
        return NextResponse.json(
            { error: "Sistem bilgileri alınamadı" },
            { status: 500 }
        )
    }
}

export const dynamic = "force-dynamic"
export const revalidate = 0
