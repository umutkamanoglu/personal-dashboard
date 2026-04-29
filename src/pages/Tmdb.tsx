import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MovieCard } from "@/components/MovieCard";
import { motion, AnimatePresence } from "framer-motion";
import { Search, Loader2, Plus, Heart, BookmarkCheck, Dices, Tv, Sparkles } from "lucide-react";
import { useNavigate } from "react-router-dom";

export default function DiscoverPage() {
  const navigate = useNavigate();
  const [movies, setMovies] = useState<any[]>([]);
  const [series, setSeries] = useState<any[]>([]);
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [isSearching, setIsSearching] = useState(false);
  const [isRandomLoading, setIsRandomLoading] = useState(false); // Rastgele yükleme durumu
  const [searchQuery, setSearchQuery] = useState("");
  const [genres, setGenres] = useState<any[]>([]);
  const [selectedGenre, setSelectedGenre] = useState<number | null>(null);

  const [moviePage, setMoviePage] = useState(1);
  const [seriesPage, setSeriesPage] = useState(1);
  const [moreLoading, setMoreLoading] = useState({ movies: false, series: false });
  
  const [favorites, setFavorites] = useState<any[]>(() => {
    const saved = localStorage.getItem("my_watchlist");
    return saved ? JSON.parse(saved) : [];
  });

  // TAMAMEN RASTGELE SEÇİM FONKSİYONU
  const handleRandomPlay = async (type: 'movie' | 'tv') => {
    setIsRandomLoading(true);
    try {
      // 1 ile 500 arasında rastgele bir sayfa seç (TMDB sınırı)
      const randomPage = Math.floor(Math.random() * 500) + 1;
      
      const data: any = await invoke(type === 'movie' ? "get_discover_movie" : "get_discover_series", { 
        page: randomPage, 
        genreId: selectedGenre 
      });

      if (data?.results?.length > 0) {
        const randomIndex = Math.floor(Math.random() * data.results.length);
        const randomItem = data.results[randomIndex];
        navigate(`/watch/${type}/${randomItem.id}`);
      }
    } catch (err) {
      console.error("Random selection error:", err);
    } finally {
      setIsRandomLoading(false);
    }
  };

  const performSearch = useCallback(async (query: string) => {
    if (!query.trim()) {
      setSearchResults([]);
      setIsSearching(false);
      return;
    }
    setIsSearching(true);
    try {
      const data: any = await invoke("search_all", { query });
      setSearchResults(data.results || []);
    } catch (err) {
      console.error("Search Error:", err);
    } finally {
      setIsSearching(false);
    }
  }, []);

  useEffect(() => {
    const timeoutId = setTimeout(() => {
      performSearch(searchQuery);
    }, 400);
    return () => clearTimeout(timeoutId);
  }, [searchQuery, performSearch]);

  const fetchData = async (genreId: number | null = null) => {
    setLoading(true);
    setMoviePage(1);
    setSeriesPage(1);
    try {
      const [movieData, seriesData, genreData]: any = await Promise.all([
        invoke("get_discover_movie", { page: 1, genreId }),
        invoke("get_discover_series", { page: 1, genreId }),
        invoke("get_genres", { mediaType: "movie" })
      ]);
      setMovies(movieData.results || []);
      setSeries(seriesData.results || []);
      setGenres(genreData.genres || []);
    } catch (err) {
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const loadMore = async (type: 'movie' | 'tv') => {
    const isMovie = type === 'movie';
    const nextPage = isMovie ? moviePage + 1 : seriesPage + 1;
    setMoreLoading(prev => ({ ...prev, [isMovie ? 'movies' : 'series']: true }));

    try {
      const data: any = await invoke(isMovie ? "get_discover_movie" : "get_discover_series", { 
        page: nextPage, 
        genreId: selectedGenre 
      });
      
      if (isMovie) {
        setMovies(prev => [...prev, ...data.results]);
        setMoviePage(nextPage);
      } else {
        setSeries(prev => [...prev, ...data.results]);
        setSeriesPage(nextPage);
      }
    } catch (err) {
      console.error(err);
    } finally {
      setMoreLoading(prev => ({ ...prev, [isMovie ? 'movies' : 'series']: false }));
    }
  };

  useEffect(() => { fetchData(selectedGenre); }, [selectedGenre]);

  const toggleFavorite = (e: React.MouseEvent, item: any) => {
    e.stopPropagation();
    const isFav = favorites.some(f => f.id === item.id);
    const updated = isFav ? favorites.filter(f => f.id !== item.id) : [item, ...favorites];
    setFavorites(updated);
    localStorage.setItem("my_watchlist", JSON.stringify(updated));
  };

  return (
    <div className="min-h-screen bg-background text-foreground p-6 md:p-12 pt-20">
      
      {/* Header Section */}
      <div className="mb-10 flex flex-col md:flex-row justify-between items-end gap-6">
        <div className="flex flex-col">
          <h1 className="text-6xl font-black italic tracking-tighter bg-gradient-to-br from-foreground to-muted-foreground bg-clip-text text-transparent">
            MİXER
          </h1>
          <p className="text-muted-foreground text-sm font-medium mt-2">Kişiselleştirilmiş İzleme Listesi</p>
        </div>
        
        <div className="relative w-full md:w-80 lg:w-[400px]">
          <div className="relative group">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-indigo-500 transition-colors" />
            <input 
              placeholder="Film, dizi veya tür ara..." 
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full h-12 pl-12 pr-4 bg-muted/40 border border-border/50 rounded-2xl focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:bg-muted/80 transition-all text-sm"
            />
            {isSearching && <Loader2 className="absolute right-4 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-indigo-500" />}
          </div>
        </div>
      </div>

      {/* Kategori Barı */}
      <div className="flex gap-3 overflow-x-auto pb-4 mb-4 no-scrollbar">
        <button 
          onClick={() => setSelectedGenre(null)}
          className={`px-6 py-2.5 rounded-xl text-[11px] font-bold transition-all ${!selectedGenre ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-500/30' : 'bg-muted/60 hover:bg-muted'}`}
        >
          TÜMÜ
        </button>
        {genres.map((g) => (
          <button 
            key={g.id}
            onClick={() => setSelectedGenre(g.id)}
            className={`px-6 cursor-pointer py-2.5 rounded-xl text-[11px] font-bold transition-all whitespace-nowrap ${selectedGenre === g.id ? 'bg-indigo-600 text-white shadow-lg shadow-indigo-500/30' : 'bg-muted/60 hover:bg-muted'}`}
          >
            {g.name.toUpperCase()}
          </button>
        ))}
      </div>

      {/* GELİŞMİŞ RASTGELE SEÇİM BUTONLARI */}
      {!searchQuery && (
        <div className="flex flex-wrap gap-4 mb-10">
          <button 
            disabled={isRandomLoading}
            onClick={() => handleRandomPlay('movie')}
            className="flex items-center gap-3 px-6 py-3 bg-indigo-500/10 hover:bg-indigo-500/20 text-indigo-500 border border-indigo-500/20 rounded-2xl transition-all group cursor-pointer disabled:opacity-50"
          >
            {isRandomLoading ? <Loader2 className="animate-spin" size={20} /> : <Dices size={20} className="group-hover:rotate-180 transition-transform duration-500" />}
            <div className="flex flex-col items-start leading-tight">
              <span className="text-[10px] opacity-60 font-bold uppercase tracking-widest">Şanslı Hisset</span>
              <span className="text-sm font-black uppercase">Rastgele Film</span>
            </div>
          </button>
          
          <button 
            disabled={isRandomLoading}
            onClick={() => handleRandomPlay('tv')}
            className="flex items-center gap-3 px-6 py-3 bg-fuchsia-500/10 hover:bg-fuchsia-500/20 text-fuchsia-500 border border-fuchsia-500/20 rounded-2xl transition-all group cursor-pointer disabled:opacity-50"
          >
            {isRandomLoading ? <Loader2 className="animate-spin" size={20} /> : <Tv size={20} className="group-hover:scale-110 transition-transform" />}
            <div className="flex flex-col items-start leading-tight">
              <span className="text-[10px] opacity-60 font-bold uppercase tracking-widest">Ne İzlesem?</span>
              <span className="text-sm font-black uppercase">Rastgele Dizi</span>
            </div>
          </button>
        </div>
      )}

      {/* İÇERİK ALANI */}
      <AnimatePresence mode="wait">
        {searchQuery ? (
          <Section key="search" title={`"${searchQuery}" için sonuçlar`} items={searchResults} toggleFavorite={toggleFavorite} favs={favorites} />
        ) : (
          <motion.div key="main" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
            
            {/* FAVORİLER (LİSTEM) SECTION */}
            {favorites.length > 0 && (
              <div className="mb-16 p-8 rounded-[2.5rem] bg-gradient-to-r from-indigo-500/10 via-transparent to-transparent border border-indigo-500/20 relative overflow-hidden">
                <div className="absolute top-0 right-0 p-10 opacity-10 pointer-events-none">
                   <Sparkles size={120} className="text-indigo-500" />
                </div>
                <div className="flex items-center gap-3 mb-8">
                  <div className="p-2 bg-indigo-500 rounded-lg shadow-lg shadow-indigo-500/20">
                    <Heart size={20} className="fill-white text-white" />
                  </div>
                  <h2 className="text-3xl font-black tracking-tight">Listem</h2>
                </div>
                <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6 relative z-10">
                  {favorites.slice(0, 6).map((item) => (
                    <FavoriteCard key={item.id} item={item} onToggle={(e) => toggleFavorite(e, item)} />
                  ))}
                </div>
              </div>
            )}

            <Section 
              title="Popüler Filmler" 
              items={movies} 
              toggleFavorite={toggleFavorite} 
              favs={favorites}
              onLoadMore={() => loadMore('movie')}
              isMoreLoading={moreLoading.movies}
            />
            
            <Section 
              title="Popüler Diziler" 
              items={series} 
              toggleFavorite={toggleFavorite} 
              favs={favorites} 
              onLoadMore={() => loadMore('tv')}
              isMoreLoading={moreLoading.series}
            />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

// Alt Bileşen: Standart Section
function Section({ title, items, toggleFavorite, favs, onLoadMore, isMoreLoading }: any) {
  return (
    <div className="mb-12">
      <div className="flex items-center gap-4 mb-8">
        <div className="h-8 w-1.5 bg-indigo-500 rounded-full shadow-[0_0_10px_rgba(99,102,241,0.5)]" />
        <h2 className="text-2xl font-bold tracking-tight">{title}</h2>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-6">
        {items.map((item: any, idx: number) => (
          <div key={`${item.id}-${idx}`} className="relative group">
            <MovieCard item={item} index={idx}/>
            <button 
              onClick={(e) => toggleFavorite(e, item)}
              className="absolute top-3 right-3 p-2.5 bg-black/40 backdrop-blur-md rounded-xl border border-white/10 opacity-0 group-hover:opacity-100 transition-all hover:scale-110 z-30 cursor-pointer"
            >
              <Heart size={16} className={favs.some((f: any) => f.id === item.id) ? "fill-red-500 text-red-500" : "text-white"} />
            </button>
          </div>
        ))}
      </div>
      
      {onLoadMore && items.length > 0 && (
        <div className="flex justify-center mt-10">
          <button 
            onClick={onLoadMore}
            disabled={isMoreLoading}
            className="flex items-center gap-2 px-8 py-3 bg-muted/50 hover:bg-muted text-foreground rounded-2xl text-xs font-bold transition-all border border-border/50 disabled:opacity-50 cursor-pointer"
          >
            {isMoreLoading ? <Loader2 className="animate-spin" size={16} /> : <Plus size={16} />}
            DAHA FAZLA GÖRÜNTÜLE
          </button>
        </div>
      )}
    </div>
  );
}

// Alt Bileşen: Favori Kartı
function FavoriteCard({ item, onToggle }: any) {
  return (
    <motion.div whileHover={{ y: -10 }} className="relative aspect-[2/3] rounded-2xl overflow-hidden shadow-2xl border-2 border-indigo-500/30 group cursor-pointer">
      <img 
        src={`https://image.tmdb.org/t/p/w500${item.poster_path}`} 
        className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110" 
      />
      <div className="absolute inset-0 bg-gradient-to-t from-black via-transparent" />
      <button 
        onClick={onToggle}
        className="absolute top-2 right-2 p-2 bg-red-500 rounded-lg shadow-lg z-10 cursor-pointer hover:scale-110 transition-transform"
      >
        <Heart size={14} className="fill-white text-white" />
      </button>
      <div className="absolute bottom-3 left-3 right-3">
        <p className="text-[10px] font-bold text-white truncate uppercase">{item.title || item.name}</p>
      </div>
    </motion.div>
  );
}