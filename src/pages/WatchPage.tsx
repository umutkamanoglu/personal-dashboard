import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { motion, AnimatePresence } from "framer-motion";
import { 
  Heart, 
  ChevronLeft, 
  Share2, 
  Lightbulb, 
  LightbulbOff, 
  Info, 
  Star,
  Users,
  AlertCircle
} from "lucide-react";

export default function WatchPage() {
  const { type, id } = useParams();
  const navigate = useNavigate();
  const [data, setData] = useState<any>(null);
  const [activeSource, setActiveSource] = useState(0);
  const [isLightsOff, setIsLightsOff] = useState(false);
  const [error, setError] = useState<string | null>(null); // Hata yakalama eklendi
  
  const [favorites, setFavorites] = useState<any[]>(() => {
    const saved = localStorage.getItem("my_watchlist");
    return saved ? JSON.parse(saved) : [];
  });

  const isFavorite = favorites.some((f) => f.id.toString() === id?.toString());

  useEffect(() => {
    // Siyah ekran sorununu önlemek için id kontrolü
    if (!id) return;

    invoke("get_item_details", { id: parseInt(id), mediaType: type })
      .then((res: any) => {
        setData(res);
        addToHistory(res);
      })
      .catch((err) => {
        console.error(err);
        setError("Veri yüklenirken bir sorun oluştu.");
      });
      
    window.scrollTo(0, 0);
  }, [id, type]);

  const addToHistory = (item: any) => {
    const history = JSON.parse(localStorage.getItem("recent_watched") || "[]");
    const filtered = history.filter((h: any) => h.id !== item.id);
    const updated = [{ ...item, media_type: type, watchedAt: new Date() }, ...filtered].slice(0, 10);
    localStorage.setItem("recent_watched", JSON.stringify(updated));
  };

  const toggleFavorite = () => {
    const updated = isFavorite 
      ? favorites.filter((f) => f.id.toString() !== id?.toString()) 
      : [{ ...data, media_type: type }, ...favorites];
    setFavorites(updated);
    localStorage.setItem("my_watchlist", JSON.stringify(updated));
  };

  const sources = [
    { name: "Server 1", url: type === "movie" ? `https://vidsrc.me/embed/movie?tmdb=${id}` : `https://vidsrc.me/embed/tv?tmdb=${id}&sea=1&epi=1` },
    { name: "Server 2", url: type === "movie" ? `https://www.2embed.cc/embed/${id}` : `https://www.2embed.cc/embedtv/${id}&s=1&e=1` },
    { name: "Server 3", url: type === "movie" ? `https://vidapi.ru/api/movie/${id}` : `https://vidapi.ru/api/tv/${id}/1/1` },
  ];

  // Hata Ekranı
  if (error) return (
    <div className="h-screen bg-[#09090b] flex flex-col items-center justify-center gap-4 text-white p-6">
      <AlertCircle size={48} className="text-red-500" />
      <p className="text-zinc-400 font-bold uppercase tracking-widest text-center">{error}</p>
      <button onClick={() => navigate(-1)} className="px-6 py-2 bg-indigo-600 rounded-full font-black">Geri Dön</button>
    </div>
  );

  // Yükleme Ekranı
  if (!data) return (
    <div className="h-screen bg-[#09090b] flex flex-col items-center justify-center gap-4 text-white">
      <div className="w-12 h-12 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
      <p className="text-zinc-500 text-[10px] font-black tracking-[0.3em] uppercase animate-pulse">Kaynaklar Hazırlanıyor</p>
    </div>
  );

  return (
    <div className={`min-h-screen transition-all duration-700 pb-20 ${isLightsOff ? 'bg-black' : 'bg-white dark:bg-[#09090b]'}`}>
      
      {/* Dinamik Blur Arka Plan */}
      {!isLightsOff && data.backdrop_path && (
        <div className="absolute top-0 left-0 w-full h-[60vh] overflow-hidden pointer-events-none opacity-40">
          <img 
            src={`https://image.tmdb.org/t/p/original${data.backdrop_path}`}
            className="w-full h-full object-cover blur-[100px] scale-110"
            alt=""
          />
        </div>
      )}

      {/* Header */}
      <AnimatePresence>
        {!isLightsOff && (
          <motion.div 
            initial={{ y: -50, opacity: 0 }} 
            animate={{ y: 0, opacity: 1 }} 
            exit={{ y: -50, opacity: 0 }}
            className="p-4 md:px-8 flex rounded-b-3xl flex-col md:flex-row items-center justify-between bg-white/70 dark:bg-zinc-900/70 backdrop-blur-3xl sticky top-0 z-[60] border-b border-zinc-200 dark:border-zinc-800 gap-4"
          >
            <div className="flex items-center gap-4 w-full md:w-auto">
              <button onClick={() => navigate(-1)} className="p-2.5 hover:bg-zinc-200 dark:hover:bg-zinc-800 rounded-2xl transition-all cursor-pointer border border-zinc-200 dark:border-zinc-700">
                <ChevronLeft size={20} />
              </button>
              <div className="flex flex-col flex-1 min-w-0">
                <h1 className="font-black text-lg leading-tight truncate max-w-[250px] md:max-w-md uppercase tracking-tighter">
                  {data.title || data.name}
                </h1>
                <span className="text-[10px] text-indigo-500 font-black uppercase tracking-widest">{type} Modu Aktif</span>
              </div>
            </div>

            {/* Server Seçimi - Resimdeki Stil */}
            <div className="flex bg-zinc-200/50 dark:bg-zinc-800/50 p-1 rounded-2xl gap-1 border border-zinc-300 dark:border-zinc-700">
              {sources.map((src, idx) => (
                <button
                  key={idx}
                  onClick={() => setActiveSource(idx)}
                  className={`px-5 py-2 rounded-xl text-xs font-black transition-all duration-300 ${
                    activeSource === idx 
                    ? "bg-indigo-600 text-white shadow-lg shadow-indigo-500/40" 
                    : "text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-200"
                  }`}
                >
                  {src.name}
                </button>
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <div className="max-w-7xl mx-auto p-4 md:p-8 relative z-10">
        {/* Üst Kontroller */}
        <div className="flex justify-between items-center mb-6">
          <div className="flex gap-2">
             <button 
              onClick={() => setIsLightsOff(!isLightsOff)}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-[10px] font-black tracking-widest uppercase transition-all ${isLightsOff ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-500/50' : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-500 hover:bg-indigo-500/10 hover:text-indigo-500'}`}
            >
              {isLightsOff ? <Lightbulb size={14} /> : <LightbulbOff size={14} />}
              {isLightsOff ? 'Işıkları Aç' : 'Sinema Modu'}
            </button>
          </div>
          
          <div className="flex gap-2">
            <button 
              onClick={() => {
                navigator.clipboard.writeText(`${data.title || data.name} izliyorum!`);
                alert("Kopyalandı!");
              }} 
              className="p-3 bg-zinc-100 dark:bg-zinc-800 text-zinc-500 hover:text-indigo-500 rounded-xl transition-all border border-transparent hover:border-indigo-500/30"
            >
              <Share2 size={18} />
            </button>
            <button 
              onClick={toggleFavorite}
              className={`p-3 rounded-xl transition-all ${isFavorite ? 'bg-red-500 text-white shadow-lg shadow-red-500/30' : 'bg-zinc-100 dark:bg-zinc-800 text-zinc-500 hover:text-red-500'}`}
            >
              <Heart size={18} className={isFavorite ? "fill-current" : ""} />
            </button>
          </div>
        </div>

        {/* Video Oynatıcı */}
        <motion.section 
          layout
          className={`relative aspect-video w-full rounded-[2.5rem] overflow-hidden bg-black shadow-2xl transition-all duration-700 ${isLightsOff ? 'ring-8 ring-indigo-600/10 scale-[1.03] z-[70]' : 'border border-zinc-200 dark:border-zinc-800'}`}
        >
          <iframe 
            src={sources[activeSource].url} 
            className="w-full h-full" 
            allowFullScreen 
            frameBorder="0"
          ></iframe>
        </motion.section>

        {!isLightsOff && (
          <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }} className="grid grid-cols-1 lg:grid-cols-3 gap-12 pt-12">
            <div className="lg:col-span-2 space-y-12">
              <div className="space-y-6">
                <div className="flex flex-wrap gap-3">
                  <div className="flex items-center gap-2 px-4 py-1.5 bg-yellow-500/10 text-yellow-600 dark:text-yellow-500 rounded-xl text-sm font-black border border-yellow-500/20">
                    <Star size={16} className="fill-current" />
                    {data.vote_average?.toFixed(1)}
                  </div>
                  {data.genres?.map((g: any) => (
                    <span key={g.id} className="px-4 py-1.5 bg-zinc-100 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400 rounded-xl text-[10px] font-black uppercase tracking-widest border border-zinc-200 dark:border-zinc-700">
                      {g.name}
                    </span>
                  ))}
                </div>
                <h2 className="text-5xl md:text-7xl font-black tracking-tighter leading-none uppercase">
                  {data.title || data.name}
                </h2>
                <p className="text-zinc-600 dark:text-zinc-400 text-xl leading-relaxed font-medium max-w-3xl">
                  {data.overview || "Özet bulunamadı."}
                </p>
              </div>

              {/* Oyuncular */}
              <div>
                <div className="flex items-center gap-3 mb-8">
                  <div className="w-1 h-6 bg-indigo-500 rounded-full" />
                  <h3 className="text-2xl font-black tracking-tight uppercase">Oyuncu Kadrosu</h3>
                </div>
                <div className="flex gap-6 overflow-x-auto pb-6 scrollbar-hide">
                  {data.credits?.cast?.slice(0, 12).map((actor: any) => (
                    <div key={actor.id} className="flex-shrink-0 w-32 group">
                      <div className="aspect-square rounded-[2rem] overflow-hidden border-4 border-transparent group-hover:border-indigo-500 transition-all duration-500 shadow-xl mb-4">
                        <img 
                          src={actor.profile_path ? `https://image.tmdb.org/t/p/w185${actor.profile_path}` : "https://via.placeholder.com/185x185?text=?"} 
                          className="w-full h-full object-cover transition-all duration-500 scale-105 group-hover:scale-110"
                          alt={actor.name}
                        />
                      </div>
                      <p className="text-sm font-black truncate">{actor.name}</p>
                      <p className="text-[11px] text-zinc-500 font-bold truncate uppercase tracking-tighter">{actor.character}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            {/* Yan Panel */}
            <aside>
              <div className="bg-zinc-100/50 dark:bg-zinc-900/50 p-8 rounded-[3rem] border border-zinc-200 dark:border-zinc-800 sticky top-32">
                <div className="space-y-8">
                  <InfoItem label="YIL" value={(data.release_date || data.first_air_date)?.split("-")[0]} />
                  <InfoItem label="ORİJİNAL DİL" value={data.original_language?.toUpperCase()} />
                  <InfoItem label="SÜRE" value={data.runtime ? `${data.runtime} DK` : `${data.number_of_seasons} SEZON`} />
                  <InfoItem label="POPÜLERLİK" value={`%${Math.round(data.popularity / 100)}`} />
                </div>
              </div>
            </aside>
          </motion.div>
        )}
      </div>
    </div>
  );
}

function InfoItem({ label, value }: { label: string, value: string }) {
  return (
    <div className="flex flex-col border-b border-zinc-200 dark:border-zinc-800 pb-4 last:border-0">
      <span className="text-[10px] text-zinc-400 font-black mb-1 tracking-[0.2em]">{label}</span>
      <span className="text-base font-black dark:text-zinc-100">{value || "---"}</span>
    </div>
  );
}