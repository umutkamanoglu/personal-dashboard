import { motion } from "framer-motion";
import { useNavigate } from "react-router-dom";

interface MovieCardProps {
  item: any;
  index: number;
}

export const MovieCard = ({ item, index }: MovieCardProps) => {
  const navigate = useNavigate();

  const imageUrl = item.poster_path 
    ? `https://image.tmdb.org/t/p/w500${item.poster_path}`
    : "/placeholder-poster.png";

  // Tıklama işleyicisi: Rust tarafında enjekte ettiğimiz media_type'ı kullanıyoruz
  const handleCardClick = () => {
    if (item.media_type && item.id) {
      navigate(`/watch/${item.media_type}/${item.id}`);
    }
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, delay: index * 0.02 }}
      whileHover={{ scale: 1.05, y: -5 }}
      onClick={handleCardClick}
      className="relative group overflow-hidden rounded-2xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 cursor-pointer shadow-lg transition-colors duration-300"
    >
      {/* Resim Alanı */}
      <div className="aspect-[2/3] w-full overflow-hidden bg-zinc-200 dark:bg-zinc-800">
        <img 
          src={imageUrl} 
          alt={item.title || item.name}
          className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
        />
      </div>

      {/* Hover Detay Katmanı */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4 z-10">
        <h3 className="text-white font-bold text-sm truncate">
          {item.title || item.name}
        </h3>
        
        <div className="flex items-center gap-2 mt-1">
          <span className="text-yellow-500 text-xs font-bold">★ {item.vote_average?.toFixed(1)}</span>
          <span className="text-zinc-300 text-[10px]">
            {item.release_date?.split('-')[0] || item.first_air_date?.split('-')[0]}
          </span>
          <span className="text-[10px] bg-indigo-600/80 text-white px-1.5 py-0.5 rounded uppercase font-medium">
            {item.media_type}
          </span>
        </div>

        <p className="text-zinc-200 text-[10px] line-clamp-2 mt-2 leading-relaxed italic">
          {item.overview || "Açıklama bulunmuyor."}
        </p>

        <div className="mt-3 py-1.5 bg-white/10 backdrop-blur-sm rounded-lg text-center text-[10px] text-white font-semibold border border-white/10 uppercase tracking-wider">
          Hemen İzle
        </div>
      </div>
    </motion.div>
  );
};