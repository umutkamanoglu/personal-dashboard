import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { motion } from "framer-motion";

export default function WatchPage() {
  const { type, id } = useParams();
  const navigate = useNavigate();
  const [data, setData] = useState<any>(null);
  const [activeSource, setActiveSource] = useState(0);

  // Kaynak Listesi - İsimler direkt sağlayıcı olarak güncellendi
  const sources = [
    { name: "Vidsrc", url: type === "movie" ? `https://vidsrc.me/embed/movie?tmdb=${id}` : `https://vidsrc.me/embed/tv?tmdb=${id}&sea=1&epi=1` },
    { name: "2Embed", url: type === "movie" ? `https://www.2embed.cc/embed/${id}` : `https://www.2embed.cc/embedtv/${id}&s=1&e=1` },
    { name: "Vidapi", url: type === "movie" ? `https://vidapi.ru/api/movie/${id}` : `https://vidapi.ru/api/tv/${id}/1/1` },
  ];

  useEffect(() => {
    invoke("get_item_details", { id: parseInt(id!), mediaType: type })
      .then((res: any) => setData(res))
      .catch(console.error);
    window.scrollTo(0, 0);
  }, [id, type]);

  if (!data) return (
    <div className="h-screen bg-zinc-950 flex flex-col items-center justify-center gap-4 text-white">
      <div className="w-10 h-10 border-4 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
      <p className="text-zinc-500 text-sm tracking-widest uppercase">İçerik Hazırlanıyor</p>
    </div>
  );

  return (
    <div className="min-h-screen bg-white dark:bg-[#09090b] text-zinc-900 dark:text-zinc-100 transition-colors duration-300 pb-20">
      
      {/* Header & Source Selection */}
      <div className="p-4 md:px-8 flex rounded-2xl flex-col md:flex-row items-center justify-between bg-white/80 dark:bg-zinc-900/80 backdrop-blur-xl sticky top-0 z-50 border-b border-zinc-200 dark:border-zinc-800 gap-4">
        <div className="flex items-center gap-4 w-full md:w-auto">
          <button onClick={() => navigate(-1)} className="p-2.5 hover:bg-zinc-200 dark:hover:bg-zinc-800 rounded-full transition-all">
            <span className="text-xl leading-none">←</span>
          </button>
          <div className="flex flex-col">
            <h1 className="font-bold text-lg leading-tight truncate max-w-[200px] md:max-w-md">
              {data.title || data.name}
            </h1>
            <span className="text-[10px] text-zinc-500 font-bold uppercase tracking-widest">{type} izleniyor</span>
          </div>
        </div>

        {/* Görseldeki gibi oval buton yapısı */}
        <div className="flex bg-zinc-100 dark:bg-zinc-800/50 p-1.5 rounded-full gap-1 border border-zinc-200 dark:border-zinc-800">
          {sources.map((src, idx) => (
            <button
              key={idx}
              onClick={() => setActiveSource(idx)}
              className={`px-6 cursor-pointer py-2 rounded-full text-xs font-black transition-all duration-300 ${
                activeSource === idx 
                ? "bg-background text-foregrund shadow-lg" 
                : "text-zinc-500 hover:text-zinc-700 dark:hover:text-zinc-300"
              }`}
            >
              {src.name}
            </button>
          ))}
        </div>
      </div>

      <div className="max-w-7xl mx-auto p-4 md:p-8 space-y-12">
        {/* Video Player Section */}
        <motion.section 
          initial={{ opacity: 0, scale: 0.98 }}
          animate={{ opacity: 1, scale: 1 }}
          className="aspect-video w-full rounded-[2rem] overflow-hidden bg-black shadow-2xl border border-zinc-200 dark:border-zinc-800 ring-1 ring-black/5"
        >
          <iframe 
            src={sources[activeSource].url} 
            className="w-full h-full" 
            allowFullScreen 
            frameBorder="0"
          ></iframe>
        </motion.section>

        {/* Detaylar */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-12 pt-4">
          <div className="lg:col-span-2 space-y-10">
            <motion.div initial={{ opacity: 0, y: 20 }} animate={{ opacity: 1, y: 0 }}>
              <div className="flex items-center gap-4 mb-4 text-sm font-bold text-yellow-500 bg-yellow-500/10 w-fit px-3 py-1 rounded-lg">
                ★ {data.vote_average?.toFixed(1)} <span className="text-zinc-400 font-normal">| TMDB Skoru</span>
              </div>
              <h2 className="text-4xl md:text-6xl font-black tracking-tighter mb-6">
                {data.title || data.name}
              </h2>
              <p className="text-zinc-800 dark:text-zinc-300 text-xl leading-relaxed font-medium">
                {data.overview || "Bu yapım için henüz bir özet eklenmemiş."}
              </p>
            </motion.div>

            {/* Oyuncu Kadrosu */}
            <div>
              <h3 className="text-xl font-black mb-8 tracking-tight uppercase border-b-2 border-indigo-500 w-fit">Kadro</h3>
              <div className="flex gap-6 overflow-x-auto pb-6 scrollbar-hide">
                {data.credits?.cast?.slice(0, 12).map((actor: any) => (
                  <div key={actor.id} className="flex-shrink-0 w-28 group text-center">
                    <div className="w-24 h-24 mx-auto rounded-full overflow-hidden border-4 border-transparent group-hover:border-indigo-500 transition-all duration-300 shadow-xl mb-3">
                      <img 
                        src={actor.profile_path ? `https://image.tmdb.org/t/p/w185${actor.profile_path}` : "https://via.placeholder.com/185x185?text=?"} 
                        className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
                        alt={actor.name}
                      />
                    </div>
                    <p className="text-xs font-black truncate px-1">{actor.name}</p>
                    <p className="text-[10px] text-zinc-500 font-medium truncate">{actor.character}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Yan Panel */}
          <aside className="space-y-6">
            <div className="bg-zinc-50 dark:bg-zinc-900/40 p-8 rounded-[2.5rem] border border-zinc-200 dark:border-zinc-800 shadow-inner">
              <h4 className="text-[10px] font-black uppercase tracking-[0.2em] text-indigo-500 mb-6">Meta Veriler</h4>
              <div className="space-y-5">
                <InfoItem label="Tür" value={data.genres?.map((g: any) => g.name).slice(0, 2).join(", ")} />
                <InfoItem label="Yıl" value={(data.release_date || data.first_air_date)?.split("-")[0]} />
                <InfoItem label="Dil" value={data.original_language?.toUpperCase()} />
                <InfoItem label="Süre" value={data.runtime ? `${data.runtime} dk` : `${data.number_of_seasons} Sezon`} />
              </div>
            </div>
          </aside>
        </div>
      </div>
    </div>
  );
}

function InfoItem({ label, value }: { label: string, value: string }) {
  return (
    <div className="flex flex-col">
      <span className="text-[10px] text-zinc-500 font-bold uppercase mb-1">{label}</span>
      <span className="text-sm font-black">{value || "Belirtilmemiş"}</span>
    </div>
  );
}