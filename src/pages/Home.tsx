import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Button } from "@/components/ui/button";
import {
  Activity,
  ArrowUpRight,
  Plus,
  PlayCircle,
  Heart,
  History,
  Play,
  Pause,
  SkipBack,
  SkipForward,
  Music2
} from "lucide-react";

interface SystemSummary {
  cpu_usage: number;
  ram_used: number;
  ram_total: number;
}

interface MediaInfo {
  title: string;
  artist: string;
  album_art: string;
  is_playing: boolean;
}

export default function Home() {
  const navigate = useNavigate();
  const [stats, setStats] = useState<SystemSummary | null>(null);
  const [recentWatch, setRecentWatch] = useState<any>(null);
  const [watchlistCount, setWatchlistCount] = useState(0);
  const [media, setMedia] = useState<MediaInfo | null>({
    title: "Henüz bir şey çalmıyor",
    artist: "Medya bekleniyor...",
    album_art: "",
    is_playing: false
  });

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

    const fetchMedia = async () => {
      try {
        const data: any = await invoke("get_active_media");
        if (data) setMedia(data);
      } catch (err) { /* Aktif medya yok */ }
    };

    const loadMixerData = () => {
      const history = JSON.parse(localStorage.getItem("recent_watched") || "[]");
      const watchlist = JSON.parse(localStorage.getItem("my_watchlist") || "[]");
      if (history.length > 0) setRecentWatch(history[0]);
      setWatchlistCount(watchlist.length);
    };

    fetchSummary();
    fetchMedia();
    loadMixerData();

    const interval = setInterval(() => {
      fetchSummary();
      fetchMedia();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleMediaCommand = async (command: string) => {
    try {
      await invoke("send_media_command", { command });
      setTimeout(async () => {
        const data: any = await invoke("get_active_media");
        if (data) setMedia(data);
      }, 300);
    } catch (err) { console.error(err); }
  };

  return (
    <div className="p-8 space-y-10 max-w-[1400px] mx-auto animate-in fade-in slide-in-from-bottom-4 duration-700">
      
      {/* BAŞLIK */}
      <div className="space-y-1">
        <h1 className="text-3xl font-extrabold tracking-tight">
          Merhaba, <span className="text-primary">umt</span>
        </h1>
        <p className="text-muted-foreground text-sm font-medium">
          Sisteminin genel durumu ve hızlı özetler burada.
        </p>
      </div>

      {/* GRID DÜZENİ */}
      <div className="grid gap-6 grid-cols-1 md:grid-cols-2 lg:grid-cols-4">

        {/* MEDYA KONTROL KARTI (Geniş - 2 Sütun) */}
        <Card className="lg:col-span-2 bg-sidebar/40 border-border/40 hover:border-emerald-500/30 transition-all duration-300 group shadow-none relative overflow-hidden flex flex-col justify-between min-h-[200px]">
          <CardHeader className="p-6 pb-2">
            <div className="flex justify-between items-center">
              <div className="p-2 bg-emerald-500/10 rounded-md text-emerald-500 group-hover:scale-110 transition-transform">
                <Music2 className="size-5" />
              </div>
              <span className="text-[10px] font-black text-emerald-500/60 uppercase tracking-widest animate-pulse">Şu an aktif</span>
            </div>
          </CardHeader>
          <CardContent className="p-6 pt-0 flex items-center justify-between">
            <div className="flex items-center gap-5 min-w-0">
              <div className="size-20 rounded-xl bg-zinc-800 flex items-center justify-center overflow-hidden border border-border/50 shadow-xl shrink-0">
                {media?.album_art ? (
                  <img src={media.album_art} className="w-full h-full object-cover" />
                ) : (
                  <Music2 className="size-10 text-zinc-700" />
                )}
              </div>
              <div className="min-w-0">
                <p className="text-xl font-bold truncate tracking-tight">{media?.title}</p>
                <p className="text-sm text-muted-foreground truncate">{media?.artist}</p>
              </div>
            </div>
            
            <div className="flex items-center gap-3 ml-4">
              <Button variant="ghost" size="icon" className="size-10 rounded-full" onClick={() => handleMediaCommand("prev")}>
                <SkipBack className="size-5 fill-current" />
              </Button>
              <Button
                variant="secondary"
                size="icon"
                className="size-14 rounded-full bg-emerald-500 text-white hover:bg-emerald-600 shadow-lg shadow-emerald-500/20"
                onClick={() => handleMediaCommand("toggle")}
              >
                {media?.is_playing ? <Pause className="size-7 fill-current" /> : <Play className="size-7 fill-current ml-1" />}
              </Button>
              <Button variant="ghost" size="icon" className="size-10 rounded-full" onClick={() => handleMediaCommand("next")}>
                <SkipForward className="size-5 fill-current" />
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* SİSTEM KARTI */}
        <Card className="bg-sidebar/40 border-border/40 hover:border-primary/30 transition-all duration-300 group shadow-none relative overflow-hidden">
          <CardHeader className="p-5 pb-2">
            <div className="flex justify-between items-center">
              <div className="p-2 bg-primary/10 rounded-md text-primary group-hover:scale-110 transition-transform">
                <Activity className="size-4" />
              </div>
              <Button variant="ghost" size="icon" className="size-7 rounded-full hover:bg-primary hover:text-white" onClick={() => navigate("/system")}>
                <ArrowUpRight className="size-4" />
              </Button>
            </div>
            <CardTitle className="text-base font-bold mt-3">Sistem</CardTitle>
          </CardHeader>
          <CardContent className="p-5 pt-0 space-y-5">
            <div className="space-y-2">
              <div className="flex justify-between text-[10px] font-bold uppercase tracking-widest text-muted-foreground/80">
                <span>CPU</span>
                <span className="text-foreground">%{stats?.cpu_usage.toFixed(0) || "0"}</span>
              </div>
              <Progress value={stats?.cpu_usage || 0} className="h-1.5 bg-background/50" />
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-[10px] font-bold uppercase tracking-widest text-muted-foreground/80">
                <span>RAM</span>
                <span className="text-foreground">%{stats ? ((stats.ram_used / stats.ram_total) * 100).toFixed(0) : "0"}</span>
              </div>
              <Progress value={stats ? (stats.ram_used / stats.ram_total) * 100 : 0} className="h-1.5 bg-background/50" />
            </div>
          </CardContent>
        </Card>

        {/* LİSTEM KARTI */}
        <Card className="bg-sidebar/40 border-border/40 hover:border-red-500/30 transition-all duration-300 group shadow-none relative overflow-hidden flex flex-col justify-between">
          <CardHeader className="p-5 pb-2">
            <div className="flex justify-between items-center">
              <div className="p-2 bg-red-500/10 rounded-md text-red-500 group-hover:scale-110 transition-transform">
                <Heart className="size-4" />
              </div>
            </div>
            <CardTitle className="text-base font-bold mt-3">Listem</CardTitle>
          </CardHeader>
          <CardContent className="p-5 pt-0">
            <div className="flex items-end gap-2">
              <span className="text-4xl font-black tracking-tighter">{watchlistCount}</span>
              <span className="text-[10px] font-bold text-muted-foreground uppercase mb-2">İçerik</span>
            </div>
            <div className="flex gap-1.5 mt-4">
              {[...Array(6)].map((_, i) => (
                <div key={i} className={`h-1 flex-1 rounded-full ${i < watchlistCount ? 'bg-red-500/40' : 'bg-muted/20'}`} />
              ))}
            </div>
          </CardContent>
        </Card>

        {/* SON İZLENEN KARTI (Geniş - 2 Sütun) */}
        <Card className="lg:col-span-2 bg-sidebar/40 border-border/40 hover:border-indigo-500/30 transition-all duration-300 group shadow-none relative overflow-hidden">
          <CardHeader className="p-6 pb-2">
            <div className="flex justify-between items-center">
              <div className="flex items-center gap-2 text-indigo-500">
                <History className="size-4" />
                <span className="text-[10px] font-black uppercase tracking-widest">İzleme Geçmişi</span>
              </div>
              <Button variant="ghost" size="icon" className="size-7 rounded-full hover:bg-indigo-500 hover:text-white" onClick={() => navigate("/tmdb")}>
                <ArrowUpRight className="size-4" />
              </Button>
            </div>
          </CardHeader>
          <CardContent className="p-6 pt-0">
            {recentWatch ? (
              <div className="flex items-center gap-6">
                <div className="h-24 w-16 rounded-lg overflow-hidden border border-border/40 shadow-lg shrink-0">
                  <img 
                    src={`https://image.tmdb.org/t/p/w92${recentWatch.poster_path}`} 
                    className="w-full h-full object-cover grayscale group-hover:grayscale-0 transition-all duration-500" 
                  />
                </div>
                <div className="space-y-3 min-w-0">
                  <div className="min-w-0">
                    <p className="text-lg font-bold truncate tracking-tight">{recentWatch.title || recentWatch.name}</p>
                    <p className="text-xs text-muted-foreground">Son izlediğiniz içerik burada listelenir.</p>
                  </div>
                  <Button 
                    onClick={() => navigate(`/watch/${recentWatch.media_type}/${recentWatch.id}`)} 
                    variant="secondary" 
                    size="sm" 
                    className="h-8 gap-2 px-4"
                  >
                    <PlayCircle size={14} /> İzlemeye Devam Et
                  </Button>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground italic">Henüz bir şey izlemedin.</p>
            )}
          </CardContent>
        </Card>

      </div>
    </div>
  );
}