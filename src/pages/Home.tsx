import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import { Cpu, Activity, ArrowUpRight, LayoutDashboard, Plus } from "lucide-react";

interface SystemSummary {
  cpu_usage: number;
  ram_used: number;
  ram_total: number;
}

export default function Home() {
  const navigate = useNavigate();
  const [stats, setStats] = useState<SystemSummary | null>(null);

  useEffect(() => {
    const fetchSummary = async () => {
      try {
        const data: any = await invoke("get_system_stats");
        setStats({
          cpu_usage: data.cpu_usage,
          ram_used: data.ram_used,
          ram_total: data.ram_total,
        });
      } catch (err) { console.error(err); }
    };
    fetchSummary();
    const interval = setInterval(fetchSummary, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    // "animate-in fade-in slide-in-from-bottom-4 duration-700" ile açılış animasyonu eklendi
    <div className="p-8 space-y-10 max-w-[1400px] mx-auto animate-in fade-in slide-in-from-bottom-4 duration-700">
      
      {/* Karşılama Bölümü */}
      <div className="space-y-1">
        <h1 className="text-3xl font-extrabold tracking-tight">
          Merhaba, <span className="text-primary">umt</span>
        </h1>
        <p className="text-muted-foreground text-sm font-medium">
          Sisteminin genel durumu ve hızlı özetler burada.
        </p>
      </div>

      <div className="grid gap-5 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        
        {/* KÜÇÜLTÜLMÜŞ SİSTEM KARTI */}
        <Card className="bg-sidebar/40 border-border/40 hover:border-primary/30 transition-all duration-300 group shadow-none relative overflow-hidden">
          <CardHeader className="p-4 pb-2">
            <div className="flex justify-between items-center">
              <div className="p-2 bg-primary/10 rounded-md text-primary group-hover:scale-110 transition-transform">
                <Activity className="size-4" />
              </div>
              <Button 
                variant="ghost" 
                size="icon" 
                className="size-7 rounded-full hover:bg-primary hover:text-primary-foreground"
                onClick={() => navigate("/system")}
              >
                <ArrowUpRight className="size-4" />
              </Button>
            </div>
            <CardTitle className="text-base font-bold mt-2">Sistem</CardTitle>
          </CardHeader>

          <CardContent className="p-4 pt-0 space-y-4">
            {/* CPU Mini */}
            <div className="space-y-1.5">
              <div className="flex justify-between text-[10px] font-bold uppercase tracking-widest text-muted-foreground/80">
                <span>İşlemci</span>
                <span className="text-foreground">%{stats?.cpu_usage.toFixed(0) || "0"}</span>
              </div>
              <Progress value={stats?.cpu_usage || 0} className="h-1 bg-background/50" />
            </div>

            {/* RAM Mini */}
            <div className="space-y-1.5">
              <div className="flex justify-between text-[10px] font-bold uppercase tracking-widest text-muted-foreground/80">
                <span>Bellek</span>
                <span className="text-foreground">
                  %{stats ? ((stats.ram_used / stats.ram_total) * 100).toFixed(0) : "0"}
                </span>
              </div>
              <Progress value={stats ? (stats.ram_used / stats.ram_total) * 100 : 0} className="h-1 bg-background/50" />
            </div>
          </CardContent>
        </Card>

        {/* YENİ ÖZELLİK EKLE - DAHA KÜÇÜK VE SADE */}
        <Card className="border-dashed bg-transparent border-border/40 hover:border-border/80 transition-colors flex flex-col items-center justify-center p-4 group cursor-pointer shadow-none">
          <div className="size-8 rounded-full border border-dashed border-border flex items-center justify-center mb-2 group-hover:bg-sidebar transition-colors">
            <Plus className="size-4 text-muted-foreground" />
          </div>
          <span className="text-xs font-semibold text-muted-foreground">Yeni Ekle</span>
        </Card>

      </div>
    </div>
  );
}