import { Moon, Sun } from "lucide-react"
import { Switch } from "@/components/ui/switch"
import { useTheme } from "@/components/theme-provider"
import { Label } from "@/components/ui/label"

const ThemeToggle = () => {
    const { theme, setTheme } = useTheme()
    const isDark = theme === "dark"
    return (
        <div className="flex items-center space-x-2 bg-secondary/50 p-1.5 px-3 rounded-full border border-border/50">
            <Sun className={`h-4 w-4 transition-all ${isDark ? "text-muted-foreground" : "text-yellow-500"}`} />

            <Switch
                id="theme-mode"
                checked={isDark}
                onCheckedChange={(checked) => setTheme(checked ? "dark" : "light")}
            />

            <Moon className={`h-4 w-4 transition-all ${isDark ? "text-blue-400" : "text-muted-foreground"}`} />

            {/* İsteğe bağlı: Etiket eklemek isterseniz */}
            {/* <Label htmlFor="theme-mode" className="sr-only">Tema Değiştir</Label> */}
        </div>
    )
}

export default ThemeToggle