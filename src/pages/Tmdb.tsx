import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MovieCard } from "@/components/MovieCard";
import { motion, AnimatePresence } from "framer-motion";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Search, Loader2 } from "lucide-react";

export default function DiscoverPage() {
  const [movies, setMovies] = useState<any[]>([]);
  const [series, setSeries] = useState<any[]>([]);
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [isSearching, setIsSearching] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  // 1. Sayfa ilk açıldığında popüler içerikleri getir
  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      try {
        const [movieData, seriesData]: any = await Promise.all([
          invoke("get_discover_movie"),
          invoke("get_discover_series")
        ]);
        setMovies(movieData.results || []);
        setSeries(seriesData.results || []);
      } catch (err) {
        console.error("Veri yükleme hatası:", err);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, []);

  // 2. Debounce Mekanizması: Kullanıcı yazmayı bıraktıktan 500ms sonra arama yap
  useEffect(() => {
    if (!searchQuery.trim()) {
      setSearchResults([]);
      setIsSearching(false);
      return;
    }

    const delayDebounceFn = setTimeout(() => {
      performSearch();
    }, 500);

    return () => clearTimeout(delayDebounceFn);
  }, [searchQuery]);

  const performSearch = async () => {
    setIsSearching(true);
    try {
      // Rust tarafındaki search_all metodunu çağırıyoruz
      const data: any = await invoke("search_all", { query: searchQuery });
      
      // Sadece film ve dizileri filtrele (oyuncuları listeden çıkar)
      const results = data.results?.filter(
        (item: any) => item.media_type === "movie" || item.media_type === "tv"
      ) || [];

      setSearchResults(results);
    } catch (err) {
      console.error("Arama hatası:", err);
    } finally {
      setIsSearching(false);
    }
  };

  const Section = ({ title, items }: { title: string, items: any[] }) => (
    <motion.div 
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="mb-12"
    >
      <h2 className="text-2xl font-bold mb-6 px-1 border-l-4 border-foreground ml-1">
        {title}
      </h2>
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
        {items.map((item, index) => (
          <MovieCard key={item.id} item={item} index={index} />
        ))}
      </div>
    </motion.div>
  );

  return (
    <div className="min-h-screen bg-white dark:bg-[#09090b] text-zinc-900 dark:text-zinc-100 p-8 pt-20 transition-colors duration-300">
      
      {/* Header & Arama Çubuğu */}
      <div className="mb-12 flex flex-col md:flex-row justify-between items-center gap-6">
        <div className="space-y-1">
          <h1 className="text-3xl font-extrabold tracking-tight">Dizi Film</h1>
          <p className="text-muted-foreground text-sm font-medium italic">
            Chill takılmak istediğin her an için...
          </p>
        </div>

        <div className="relative w-full md:w-96">
          <InputGroup>
            <InputGroupInput 
              placeholder="Film veya dizi ara..." 
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            <InputGroupAddon>
              {isSearching ? (
                <Loader2 className="h-4 w-4 animate-spin text-indigo-500" />
              ) : (
                <Search className="h-4 w-4" />
              )}
            </InputGroupAddon>
          </InputGroup>
        </div>
      </div>

      {/* İçerik Alanı */}
      <AnimatePresence mode="wait">
        {loading ? (
          <motion.div 
            key="loading"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex justify-center py-20 text-indigo-500 font-bold tracking-tighter"
          >
            YÜKLENİYOR...
          </motion.div>
        ) : (
          <motion.div key="content">
            {searchQuery ? (
              searchResults.length > 0 ? (
                <Section title={`"${searchQuery}" için sonuçlar`} items={searchResults} />
              ) : (
                !isSearching && (
                  <div className="text-center py-20 text-zinc-500">
                    Aradığınız kriterlere uygun sonuç bulunamadı.
                  </div>
                )
              )
            ) : (
              <>
                <Section title="Popüler Filmler" items={movies} />
                <Section title="Popüler Diziler" items={series} />
              </>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}