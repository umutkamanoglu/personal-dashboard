import { useState, useEffect, useRef } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { 
  Play, 
  Pause, 
  RotateCcw, 
  CheckCircle2, 
  Circle, 
  Plus,
  Trash2,
  Timer,
  Coffee,
  Brain
} from "lucide-react";

interface Todo {
  id: number;
  text: string;
  completed: boolean;
}

type TimerMode = 'work' | 'break';

export default function Focus() {
  const [mode, setMode] = useState<TimerMode>('work');
  const [workTime, setWorkTime] = useState(25 * 60);
  const [breakTime, setBreakTime] = useState(5 * 60);
  const [timeLeft, setTimeLeft] = useState(25 * 60);
  const [isActive, setIsActive] = useState(false);
  const [customMins, setCustomMins] = useState("");
  
  const audioRef = useRef<HTMLAudioElement | null>(null);

  const [todos, setTodos] = useState<Todo[]>(() => {
    const saved = localStorage.getItem("focus_todos");
    return saved ? JSON.parse(saved) : [];
  });
  const [newTodo, setNewTodo] = useState("");

  useEffect(() => {
    audioRef.current = new Audio("/alarm.mp3");
    const checkNotification = async () => {
      if (!(await isPermissionGranted())) await requestPermission();
    };
    checkNotification();
  }, []);

  useEffect(() => {
    let interval: any = null;
    if (isActive && timeLeft > 0) {
      interval = setInterval(() => setTimeLeft((prev) => prev - 1), 1000);
    } else if (timeLeft === 0 && isActive) {
      handlePhaseEnd();
    }
    return () => clearInterval(interval);
  }, [isActive, timeLeft]);

  useEffect(() => {
    localStorage.setItem("focus_todos", JSON.stringify(todos));
  }, [todos]);

  const handlePhaseEnd = async () => {
    setIsActive(false);
    audioRef.current?.play().catch(e => console.error("Ses çalınamadı:", e));
    const isWork = mode === 'work';
    const nextMode = isWork ? 'break' : 'work';
    const nextTime = isWork ? breakTime : workTime;

    if (await isPermissionGranted()) {
      sendNotification({
        title: isWork ? 'Çalışma bitti!' : 'Mola bitti!',
        body: isWork ? 'Şimdi mola zamanı.' : 'Tekrar odaklanmaya hazır mısın?',
      });
    }
    setMode(nextMode);
    setTimeLeft(nextTime);
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  const addTodo = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newTodo.trim()) return;
    setTodos([{ id: Date.now(), text: newTodo, completed: false }, ...todos]);
    setNewTodo("");
  };

  const toggleTodo = (id: number) => {
    setTodos(todos.map(t => t.id === id ? { ...t, completed: !t.completed } : t));
  };

  const deleteTodo = (id: number) => {
    setTodos(todos.filter(t => t.id !== id));
  };

  return (
    <div className="p-8 space-y-8 max-w-[1200px] mx-auto animate-in fade-in duration-500">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-4xl font-black tracking-tighter flex items-center gap-3">
            {mode === 'work' ? <Brain className="text-primary size-8" /> : <Coffee className="text-emerald-500 size-8" />}
            {mode === 'work' ? "ODAK MODU" : "MOLA ZAMANI"}
          </h1>
          <p className="text-muted-foreground font-medium">
            {mode === 'work' ? "Şu an derin çalışma aşamasındasın." : "Kısa bir nefes al ve tazelen."}
          </p>
        </div>
        
        <div className="flex bg-sidebar/60 p-1.5 rounded-2xl border border-border/40 backdrop-blur-md">
          <Button 
            variant={mode === 'work' ? "default" : "ghost"} 
            size="sm" 
            className="rounded-xl px-6 font-bold"
            onClick={() => { setMode('work'); setTimeLeft(workTime); setIsActive(false); }}
          >
            Odak
          </Button>
          <Button 
            variant={mode === 'break' ? "default" : "ghost"} 
            size="sm" 
            className="rounded-xl px-6 font-bold"
            onClick={() => { setMode('break'); setTimeLeft(breakTime); setIsActive(false); }}
          >
            Mola
          </Button>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-12">
        
        {/* SOL: SAYAÇ VE AYARLAR */}
        <div className="md:col-span-7 space-y-4">
          <Card className={`border-border/40 shadow-none flex flex-col justify-center items-center p-12 overflow-hidden relative min-h-[420px] transition-colors duration-500 ${mode === 'work' ? 'bg-sidebar/40' : 'bg-emerald-500/5 border-emerald-500/20'}`}>
            <div className="absolute top-0 left-0 w-full h-1.5 bg-muted/20">
              <div 
                className={`h-full transition-all duration-1000 ${mode === 'work' ? 'bg-primary' : 'bg-emerald-500'}`} 
                style={{ width: `${(timeLeft / (mode === 'work' ? workTime : breakTime)) * 100}%` }}
              />
            </div>
            
            <div className="text-[140px] font-black tracking-tighter leading-none mb-10 tabular-nums">
              {formatTime(timeLeft)}
            </div>
            
            <div className="flex gap-6 items-center">
              <Button 
                size="lg" 
                className={`w-44 h-16 text-xl font-black rounded-2xl shadow-2xl ${mode === 'work' ? 'shadow-primary/20' : 'bg-emerald-500 hover:bg-emerald-600 shadow-emerald-500/20'}`}
                onClick={() => setIsActive(!isActive)}
              >
                {isActive ? <Pause className="mr-2 size-6 fill-current" /> : <Play className="mr-2 size-6 fill-current" />}
                {isActive ? "DURAKLAT" : "BAŞLAT"}
              </Button>
              <Button 
                variant="outline" 
                size="icon" 
                className="size-16 rounded-2xl border-2"
                onClick={() => { setIsActive(false); setTimeLeft(mode === 'work' ? workTime : breakTime); }}
              >
                <RotateCcw className="size-7" />
              </Button>
            </div>
          </Card>

          <div className="grid grid-cols-2 gap-4">
            <Card className="bg-sidebar/20 border-border/40 p-5 flex flex-col gap-3">
              <span className="text-[10px] font-black uppercase text-muted-foreground">Hızlı Süreler</span>
              <div className="flex gap-2">
                {[15, 25, 50].map(m => (
                  <Button key={m} variant="secondary" size="sm" className="flex-1 font-bold" onClick={() => {
                    setIsActive(false);
                    if(mode === 'work') { setWorkTime(m*60); setTimeLeft(m*60); }
                    else { setBreakTime(m*60); setTimeLeft(m*60); }
                  }}>{m}dk</Button>
                ))}
              </div>
            </Card>

            <Card className="bg-sidebar/20 border-border/40 p-5 flex flex-col gap-3">
              <span className="text-[10px] font-black uppercase text-muted-foreground">Özel Ayar</span>
              <div className="flex gap-2">
                <Input 
                  type="number"
                  placeholder="Dakika..."
                  value={customMins}
                  onChange={(e) => setCustomMins(e.target.value)}
                  className="bg-background/40 border-none h-9 text-xs"
                />
                <Button size="sm" className="px-4 font-bold" onClick={() => {
                   const m = parseInt(customMins);
                   if(m > 0) {
                     setIsActive(false);
                     if(mode === 'work') { setWorkTime(m*60); setTimeLeft(m*60); }
                     else { setBreakTime(m*60); setTimeLeft(m*60); }
                   }
                }}>Ayarla</Button>
              </div>
            </Card>
          </div>
        </div>

        {/* SAĞ: SEANS HEDEFLERİ (TODO LIST) */}
        <Card className="md:col-span-5 bg-sidebar/40 border-border/40 shadow-none flex flex-col min-h-[500px]">
          <CardHeader>
            <CardTitle className="text-xl font-bold flex items-center gap-2">
              <CheckCircle2 className="size-5 text-primary" />
              Seans Hedefleri
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 flex-1">
            <form onSubmit={addTodo} className="flex gap-2">
              <Input 
                placeholder="Neye odaklanacaksın?" 
                value={newTodo}
                onChange={(e) => setNewTodo(e.target.value)}
                className="bg-background/50 border-border/40"
              />
              <Button type="submit" size="icon" className="shrink-0"><Plus className="size-4" /></Button>
            </form>
            
            <div className="space-y-2 overflow-y-auto max-h-[440px] pr-2 custom-scrollbar">
              {todos.length === 0 && (
                <div className="text-center py-20 space-y-3 opacity-20">
                  <Timer className="size-12 mx-auto" />
                  <p className="text-xs font-medium">Henüz bir hedef belirlemedin.</p>
                </div>
              )}
              {todos.map(todo => (
                <div 
                  key={todo.id} 
                  className="flex items-center justify-between p-3 rounded-xl bg-background/30 border border-border/20 group animate-in slide-in-from-right-2"
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <button onClick={() => toggleTodo(todo.id)}>
                      {todo.completed ? 
                        <CheckCircle2 className="size-5 text-emerald-500" /> : 
                        <Circle className="size-5 text-muted-foreground" />
                      }
                    </button>
                    <span className={`text-sm font-medium truncate ${todo.completed ? 'line-through text-muted-foreground' : ''}`}>
                      {todo.text}
                    </span>
                  </div>
                  <Button 
                    variant="ghost" 
                    size="icon" 
                    className="size-8 text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
                    onClick={() => deleteTodo(todo.id)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

      </div>
    </div>
  );
}