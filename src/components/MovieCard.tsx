import { motion } from "framer-motion";

export const MovieCard = ({ item, index }: { item: any; index: number }) => {
  const imageUrl = item.poster_path 
    ? `https://image.tmdb.org/t/p/w500${item.poster_path}`
    : "/placeholder-poster.png";

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4, delay: index * 0.02 }}
      whileHover={{ scale: 1.05, y: -5 }}
      className="relative group overflow-hidden rounded-2xl bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 cursor-pointer shadow-lg transition-colors duration-300"
    >
      <div className="aspect-[2/3] w-full overflow-hidden">
        <img 
          src={imageUrl} 
          alt={item.title || item.name}
          className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110"
        />
      </div>

      <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex flex-col justify-end p-4">
        <h3 className="text-white font-bold text-sm truncate">{item.title || item.name}</h3>
        <div className="flex items-center gap-2 mt-1">
          <span className="text-yellow-500 text-xs font-bold">★ {item.vote_average?.toFixed(1)}</span>
          <span className="text-zinc-300 text-[10px]">
            {item.release_date?.split('-')[0] || item.first_air_date?.split('-')[0]}
          </span>
        </div>
        <p className="text-zinc-200 text-[10px] line-clamp-2 mt-2 leading-relaxed">{item.overview}</p>
      </div>
    </motion.div>
  );
};