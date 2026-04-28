import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { 
  Cpu, Zap, HardDrive, Activity, Monitor, Clock, ArrowDown, ArrowUp 
} from "lucide-react";

// 1. EKSİK OLAN INTERFACE TANIMI
interface SystemData {
  cpu_usage: number;
  ram_used: number;
  ram_total: number;
  gpu_name: string | null;
  gpu_usage: number | null;
  gpu_temp: number | null;
  gpu_mem_used: number | null;
  disks: [string, number, number][];
  net_in: number;
  net_out: number;
  os: string;
  uptime: number;
}

export default function System() {
  const [data, setData] = useState<SystemData | null>(null);

  // 2. EKSİK OLAN FORMAT FONKSİYONLARI
  const formatBytes = (bytes: number) => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  };

  const formatUptimeShort = (s: number) => {
    const d = Math.floor(s / (3600 * 24));
    const h = Math.floor((s % (3600 * 24)) / 3600);
    const m = Math.floor((s % 3600) / 60);
    return `${d}g ${h}s ${m}d`;
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        // DİKKAT: Rust tarafındaki #[tauri::command] isminin tam olarak bu olduğundan emin ol
        const stats: SystemData = await invoke("get_system_stats");
        setData(stats);
      } catch (err) { 
        console.error("Rust'tan veri çekilemedi:", err); 
      }
    };
    fetchData();
    const interval = setInterval(fetchData, 2000);
    return () => clearInterval(interval);
  }, []);

  // Veri gelene kadar bir yükleniyor ekranı göster (Null hatasını engeller)
  if (!data) return (
    <div className="flex h-full items-center justify-center text-muted-foreground animate-pulse">
      Sistem verileri yükleniyor...
    </div>
  );

  return (
    <div className="p-6 space-y-8 max-w-[1600px] mx-auto animate-in fade-in duration-500">
      {/* Üst Başlık */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent">
            Sistem İzleyici
          </h1>
          <p className="text-muted-foreground text-sm">Donanım kaynaklarının anlık durumu.</p>
        </div>
        <Badge variant="secondary" className="py-1.5 px-4 bg-sidebar border-border/50 text-xs font-medium flex gap-2 items-center rounded-full shadow-sm">
          <Monitor className="size-3.5 text-primary" /> {data.os}
        </Badge>
      </div>

      {/* Ana Kartlar */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        <StatCard 
          title="İşlemci (CPU)" 
          value={`%${data.cpu_usage.toFixed(1)}`} 
          icon={<Cpu className="size-4 text-orange-500" />}
          progress={data.cpu_usage}
          footer="Anlık yüklenme oranı"
        />

        <StatCard 
          title="Bellek (RAM)" 
          value={formatBytes(data.ram_used)}
          subValue={`/ ${formatBytes(data.ram_total)}`}
          icon={<Activity className="size-4 text-blue-500" />}
          progress={(data.ram_used / data.ram_total) * 100}
          footer="Kullanılan toplam bellek"
        />

        <StatCard 
          title="Ekran Kartı (GPU)" 
          value={data.gpu_usage !== null ? `%${data.gpu_usage}` : "N/A"} 
          icon={<Zap className="size-4 text-yellow-500" />}
          progress={data.gpu_usage || 0}
          footer={data.gpu_name || "Bağlı cihaz yok"}
          extra={data.gpu_temp ? `${data.gpu_temp}°C` : ""}
        />

        <StatCard 
          title="Çalışma Süresi" 
          value={formatUptimeShort(data.uptime)} 
          icon={<Clock className="size-4 text-green-500" />}
          footer="Son açılıştan beri"
        />
      </div>

      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-12">
        {/* Depolama */}
        <Card className="lg:col-span-8 bg-sidebar/50 border-border/40 shadow-none backdrop-blur-sm">
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2 font-semibold text-foreground">
              <HardDrive className="size-5 text-primary" /> Sürücü Durumu
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-6">
            {data.disks.map(([point, total, available], idx) => {
              const used = total - available;
              const percent = (used / total) * 100;
              return (
                <div key={idx} className="group">
                  <div className="flex justify-between items-end mb-2">
                    <div className="flex flex-col text-foreground">
                      <span className="text-sm font-bold">{point} Sürücüsü</span>
                      <span className="text-[11px] text-muted-foreground uppercase tracking-wider font-medium">
                         {formatBytes(used)} / {formatBytes(total)}
                      </span>
                    </div>
                    <span className={`text-sm font-bold ${percent > 90 ? 'text-destructive' : 'text-primary'}`}>
                      %{percent.toFixed(1)}
                    </span>
                  </div>
                  <Progress value={percent} className="h-1.5 bg-background/50" />
                </div>
              );
            })}
          </CardContent>
        </Card>

        {/* Ağ Trafiği */}
        <Card className="lg:col-span-4 bg-sidebar/50 border-border/40 shadow-none backdrop-blur-sm">
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2 font-semibold text-foreground">
              <Activity className="size-5 text-primary" /> Ağ Akışı
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col justify-around h-[200px]">
            <div className="flex items-center justify-between p-4 rounded-xl bg-background/30 border border-border/20">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-blue-500/10 rounded-lg text-blue-500"><ArrowDown className="size-5" /></div>
                <span className="text-sm font-medium text-muted-foreground uppercase tracking-tight">Gelen</span>
              </div>
              <span className="text-lg font-mono font-bold text-foreground">{formatBytes(data.net_in)}</span>
            </div>
            <div className="flex items-center justify-between p-4 rounded-xl bg-background/30 border border-border/20">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-green-500/10 rounded-lg text-green-500"><ArrowUp className="size-5" /></div>
                <span className="text-sm font-medium text-muted-foreground uppercase tracking-tight">Giden</span>
              </div>
              <span className="text-lg font-mono font-bold text-foreground">{formatBytes(data.net_out)}</span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function StatCard({ title, value, subValue, icon, progress, footer, extra }: any) {
  return (
    <Card className="bg-sidebar border-border/40 hover:border-primary/30 transition-colors shadow-none overflow-hidden relative group">
      <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
        <CardTitle className="text-[11px] uppercase tracking-[0.1em] font-bold text-muted-foreground/80">
          {title}
        </CardTitle>
        <div className="p-2 bg-background/50 rounded-lg group-hover:scale-110 transition-transform">
          {icon}
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-baseline gap-1 text-foreground">
          <span className="text-2xl font-bold tracking-tight font-mono">{value}</span>
          {subValue && <span className="text-xs text-muted-foreground font-medium">{subValue}</span>}
        </div>
        {progress !== undefined && (
          <div className="mt-4 mb-2">
            <Progress value={progress} className="h-1 bg-background/50" />
          </div>
        )}
        <div className="flex justify-between items-center mt-2">
          <p className="text-[10px] text-muted-foreground font-medium truncate max-w-[80%]">{footer}</p>
          {extra && <Badge variant="outline" className="text-[9px] h-4 px-1 rounded-sm border-orange-500/20 text-orange-500">{extra}</Badge>}
        </div>
      </CardContent>
    </Card>
  );
}