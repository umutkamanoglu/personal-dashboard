import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MovieCard } from "@/components/MovieCard";
import { motion } from "framer-motion";
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from "@/components/ui/input-group";
import { Search } from "lucide-react";

export default function DiscoverPage() {
  const [movies, setMovies] = useState<any[]>([]);
  const [series, setSeries] = useState<any[]>([]);
  const [searchResults, setSearchResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");

  const fetchData = async () => {
    setLoading(true);
    try {
      const [movieData, seriesData]: any = await Promise.all([
        invoke("get_discover_movie", { params: "&sort_by=popularity.desc" }),
        invoke("get_discover_series", { params: "&sort_by=popularity.desc" })
      ]);
      setMovies(movieData.results || []);
      setSeries(seriesData.results || []);
    } catch (err) {
      console.error("Veri çekme hatası:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) {
      setSearchResults([]);
      return;
    }
    setLoading(true);
    try {
      // Arama için genel discovery kullanabilir veya search endpoint'i ekleyebilirsin
      const data: any = await invoke("get_discover_movie", {
        params: `&query=${encodeURIComponent(searchQuery)}`
      });
      setSearchResults(data.results || []);
    } catch (err) { console.error(err); }
    finally { setLoading(false); }
  };

  const Section = ({ title, items }: { title: string, items: any[] }) => (
    <div className="mb-12">
      <h2 className="text-2xl font-bold mb-6 px-1 border-l-4 border-foreground ml-1">{title}</h2>
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6">
        {items.map((item, index) => (
          <MovieCard key={item.id} item={item} index={index} />
        ))}
      </div>
    </div>
  );

  return (
    <div className="min-h-screen bg-white dark:bg-[#09090b] text-zinc-900 dark:text-zinc-100 p-8 pt-20 transition-colors duration-300">

      {/* Search Header */}
      <div className="mb-12 flex flex-col md:flex-row justify-between items-center gap-6">
        <div className="space-y-1">
          <h1 className="text-3xl font-extrabold tracking-tight">
            Dizi Film
          </h1>
          <p className="text-muted-foreground text-sm font-medium">
            Chill takılmak istediğin her an için...
          </p>
        </div>
        <form onSubmit={handleSearch} className="relative w-full md:w-96">
          {/* <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Film veya dizi ara..."
            className="w-full px-5 py-3 bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-2xl outline-none outline-none transition-all"
          /> */}
          <InputGroup>
            <InputGroupInput placeholder="Film veya dizi ara..." name="watchsearch" />
            <InputGroupAddon>
              <Search />
            </InputGroupAddon>
          </InputGroup>
        </form>
      </div>

      {loading ? (
        <div className="flex justify-center py-20 animate-pulse text-indigo-500">Yükleniyor...</div>
      ) : (
        <>
          {searchQuery ? (
            <Section title="Arama Sonuçları" items={searchResults} />
          ) : (
            <>
              <Section title="Popüler Filmler" items={movies} />
              <Section title="Popüler Diziler" items={series} />
            </>
          )}
        </>
      )}
    </div>
  );
}