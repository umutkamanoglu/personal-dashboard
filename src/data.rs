use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::PathBuf,
};

use chrono::{Datelike, Duration, Local, Months, NaiveDate, Timelike, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
}

impl Frequency {
    pub fn unit(self) -> &'static str {
        match self {
            Frequency::Daily => "gün",
            Frequency::Weekly => "hafta",
            Frequency::Monthly => "ay",
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            1 => Frequency::Weekly,
            2 => Frequency::Monthly,
            _ => Frequency::Daily,
        }
    }
}

/// Period beginning for weekly habits (Monday of the week).
pub fn period_start(date: NaiveDate, frequency: Frequency) -> NaiveDate {
    match frequency {
        Frequency::Daily => date,
        Frequency::Weekly => date - Duration::days(date.weekday().num_days_from_monday() as i64),
        Frequency::Monthly => NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap(),
    }
}

fn period_key(date: NaiveDate, frequency: Frequency) -> String {
    let start = period_start(date, frequency);
    match frequency {
        Frequency::Monthly => format!("{}-{:02}", start.year(), start.month()),
        _ => start.format("%Y-%m-%d").to_string(),
    }
}

fn previous_period(date: NaiveDate, frequency: Frequency) -> NaiveDate {
    let start = period_start(date, frequency);
    match frequency {
        Frequency::Daily => start - Duration::days(1),
        Frequency::Weekly => start - Duration::days(7),
        Frequency::Monthly => {
            let (y, m) = if start.month() == 1 {
                (start.year() - 1, 12)
            } else {
                (start.year(), start.month() - 1)
            };
            NaiveDate::from_ymd_opt(y, m, 1).unwrap()
        }
    }
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Bugünün hafta günü indeksi: 0 = Pazartesi ... 6 = Pazar
pub fn weekday_of_today() -> u8 {
    today().weekday().num_days_from_monday() as u8
}

/// "Pazartesi, 26 Ekim 2026" gibi Türkçe tarih.
pub fn today_tr() -> String {
    let date = today();
    let weekday = match date.weekday() {
        Weekday::Mon => "Pazartesi",
        Weekday::Tue => "Salı",
        Weekday::Wed => "Çarşamba",
        Weekday::Thu => "Perşembe",
        Weekday::Fri => "Cuma",
        Weekday::Sat => "Cumartesi",
        Weekday::Sun => "Pazar",
    };
    let month = match date.month() {
        1 => "Ocak",
        2 => "Şubat",
        3 => "Mart",
        4 => "Nisan",
        5 => "Mayıs",
        6 => "Haziran",
        7 => "Temmuz",
        8 => "Ağustos",
        9 => "Eylül",
        10 => "Ekim",
        11 => "Kasım",
        _ => "Aralık",
    };
    format!("{weekday}, {} {month} {}", date.day(), date.year())
}

pub fn greeting() -> &'static str {
    match Local::now().hour() {
        0..=5 => "İyi geceler",
        6..=11 => "Günaydın",
        12..=17 => "İyi günler",
        _ => "İyi akşamlar",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub frequency: Frequency,
    pub created_at: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineItem {
    pub id: String,
    pub name: String,
    pub created_at: NaiveDate,
}

/// Serbest not.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub text: String,
    pub created_at: NaiveDate,
}

/// Haftanın seçili günlerinde hatırlatılan not ("0 = Pzt ... 6 = Paz").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub text: String,
    pub days: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PomodoroPhase {
    #[serde(rename = "work")]
    Work,
    #[serde(rename = "break")]
    Break,
}

/// Aktif Pomodoro seansı; epoch saniyeleriyle tutulur, uygulama kapansa bile devam eder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pomodoro {
    pub label: String,
    pub phase: PomodoroPhase,
    /// aktif fazın başlangıcı (unix saniye)
    pub phase_start: i64,
    pub work_min: u32,
    pub break_min: u32,
    pub rounds_done: u32,
}

/// Saat dilimi girişi: "günde hangi saatte ne var"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub id: String,
    pub name: String,
    /// "HH:MM"
    pub start: String,
    /// "HH:MM"
    pub end: String,
    /// 0 = Pazartesi ... 6 = Pazar
    pub days: Vec<u8>,
}

impl ScheduleEntry {
    /// "HH:MM" parse; geçersizse None.
    pub fn parse_time(text: &str) -> Option<String> {
        let text = text.trim();
        let (h, m) = text.split_once(':')?;
        let (h, m): (u32, u32) = (h.trim().parse().ok()?, m.trim().parse().ok()?);
        if h < 24 && m < 60 {
            Some(format!("{h:02}:{m:02}"))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardData {
    pub habits: Vec<Habit>,
    pub routine_items: Vec<RoutineItem>,
    /// kullanıcı adı (selamlamada kullanılır)
    #[serde(default = "default_name")]
    pub name: String,
    /// pomodoro varsayılan odak süresi (dk)
    #[serde(default = "default_work_min")]
    pub work_min: u32,
    /// pomodoro varsayılan mola süresi (dk)
    #[serde(default = "default_break_min")]
    pub break_min: u32,
    /// habit id -> completed period keys ("2026-10-26" / week-monday / "2026-10")
    #[serde(default)]
    pub habit_completions: HashMap<String, BTreeSet<String>>,
    /// routine item id -> completed dates
    #[serde(default)]
    pub routine_checks: HashMap<String, BTreeSet<NaiveDate>>,
    /// saat dilimleri
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
    /// notlar
    #[serde(default)]
    pub notes: Vec<Note>,
    /// hatırlatıcılar
    #[serde(default)]
    pub reminders: Vec<Reminder>,
    /// koyu tema mı (kaydedilir, açık tema varsayılan)
    #[serde(default)]
    pub dark_mode: bool,
    /// aktif pomodoro seansı (yoksa None)
    #[serde(default)]
    pub pomodoro: Option<Pomodoro>,
}

fn default_name() -> String {
    "Umut".into()
}
fn default_work_min() -> u32 {
    25
}
fn default_break_min() -> u32 {
    5
}

impl DashboardData {
    fn storage_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("personal-dashboard")
            .join("data.json")
    }

    pub fn load() -> Self {
        let path = Self::storage_path();
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(dir) = Self::storage_path().parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_vec_pretty(self) {
            let _ = fs::write(Self::storage_path(), json);
        }
    }

    fn new_id() -> String {
        format!("{}_{}", today().format("%Y%m%d"), std::process::id())
            + &format!(
                "_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0)
            )
    }

    // --- Habits ---

    pub fn add_habit(&mut self, name: String, frequency: Frequency) {
        self.habits.push(Habit {
            id: Self::new_id(),
            name,
            frequency,
            created_at: today(),
        });
        self.save();
    }

    /// Tema tercihi kaydı.
    pub fn set_dark_mode(&mut self, dark: bool) {
        self.dark_mode = dark;
        self.save();
    }

    /// Profil ve pomodoro varsayılanlarını kaydeder.
    pub fn set_settings(&mut self, name: String, work_min: u32, break_min: u32) {
        self.name = if name.trim().is_empty() {
            "Umut".into()
        } else {
            name
        };
        self.work_min = work_min.clamp(5, 120);
        self.break_min = break_min.clamp(1, 60);
        self.save();
    }

    pub fn remove_habit(&mut self, id: &str) {
        self.habits.retain(|h| h.id != id);
        self.habit_completions.remove(id);
        self.save();
    }

    pub fn habit_done(&self, habit: &Habit, date: NaiveDate) -> bool {
        self.habit_completions
            .get(&habit.id)
            .is_some_and(|set| set.contains(&period_key(date, habit.frequency)))
    }

    pub fn toggle_habit(&mut self, id: &str) {
        let Some(habit) = self.habits.iter().find(|h| h.id == id) else {
            return;
        };
        let frequency = habit.frequency;
        let key = period_key(today(), frequency);
        let set = self.habit_completions.entry(id.to_string()).or_default();
        if set.remove(&key) {
            self.save();
            return;
        }
        set.insert(key);
        self.save();
    }

    pub fn habit_streak(&self, habit: &Habit) -> u32 {
        let Some(set) = self.habit_completions.get(&habit.id) else {
            return 0;
        };
        let mut cursor = today();
        let mut streak = 0;
        if !habit_done_impl(set, habit.frequency, today()) {
            // bugün henüz yapılmadıysa seriyi önceki dönemden devam ettir
            cursor = previous_period(cursor, habit.frequency);
        }
        while habit_done_impl(set, habit.frequency, cursor) {
            streak += 1;
            cursor = previous_period(cursor, habit.frequency);
        }
        streak
    }

    /// Bugün / bu hafta / bu ay için tamamlanan günlük alışkanlık + rutin sayısı.
    pub fn today_progress(&self) -> (usize, usize) {
        let mut done = 0;
        let mut total = 0;
        for habit in self
            .habits
            .iter()
            .filter(|h| h.frequency == Frequency::Daily)
        {
            total += 1;
            if self.habit_done(habit, today()) {
                done += 1;
            }
        }
        for item in &self.routine_items {
            total += 1;
            if self.routine_done(&item.id, today()) {
                done += 1;
            }
        }
        (done, total)
    }

    /// Rutinin son `days` gün içinde yapıldığı gün sayısı.
    pub fn routine_week_count(&self, id: &str, days: u32) -> u32 {
        let Some(set) = self.routine_checks.get(id) else {
            return 0;
        };
        let cutoff = today() - Duration::days(days as i64);
        set.range(cutoff..).count() as u32
    }

    // --- Pomodoro ---

    fn now_epoch_secs() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    /// Aktif fazın toplam süresi (saniye).
    fn phase_secs(p: &Pomodoro) -> i64 {
        let mins = match p.phase {
            PomodoroPhase::Work => p.work_min,
            PomodoroPhase::Break => p.break_min,
        };
        mins as i64 * 60
    }

    /// Yeni pomodoro başlatır.
    pub fn start_pomodoro(&mut self, label: String, work_min: u32, break_min: u32) {
        self.pomodoro = Some(Pomodoro {
            label,
            phase: PomodoroPhase::Work,
            phase_start: Self::now_epoch_secs(),
            work_min,
            break_min,
            rounds_done: 0,
        });
        self.save();
    }

    pub fn stop_pomodoro(&mut self) {
        self.pomodoro = None;
        self.save();
    }

    /// Aktif fazı hemen bitirip diğerine geçer (Work bitince rounds++).
    pub fn skip_pomodoro_phase(&mut self) {
        let Some(p) = self.pomodoro.as_mut() else {
            return;
        };
        if p.phase == PomodoroPhase::Work {
            p.rounds_done += 1;
        }
        p.phase = match p.phase {
            PomodoroPhase::Work => PomodoroPhase::Break,
            PomodoroPhase::Break => PomodoroPhase::Work,
        };
        p.phase_start = Self::now_epoch_secs();
        self.save();
    }

    /// Zaman dolunca fazı otomatik geçirir (her tick'te çağrılır).
    pub fn advance_pomodoro(&mut self) -> bool {
        let Some(p) = self.pomodoro.as_mut() else {
            return false;
        };
        let expected = Self::phase_secs(p);
        if Self::now_epoch_secs() - p.phase_start < expected {
            return false;
        }
        self.skip_pomodoro_phase();
        // faz başlangıcını aşık süreyle kaydırmak yerine tam sınırda tut
        if let Some(p) = self.pomodoro.as_mut() {
            p.phase_start = Self::now_epoch_secs();
        }
        true
    }

    /// (faz, kalan saniye, etiket, tamamlanan tur); seans yoksa None.
    pub fn pomodoro_remaining(&self) -> Option<(PomodoroPhase, i64, String, u32)> {
        let p = self.pomodoro.as_ref()?;
        let expected = Self::phase_secs(p);
        let elapsed = Self::now_epoch_secs() - p.phase_start;
        Some((
            p.phase,
            (expected - elapsed).max(0),
            p.label.clone(),
            p.rounds_done,
        ))
    }

    // --- Notlar & hatırlatıcılar ---

    pub fn add_note(&mut self, text: String) {
        self.notes.push(Note {
            id: Self::new_id(),
            text,
            created_at: today(),
        });
        self.save();
    }

    pub fn remove_note(&mut self, id: &str) {
        self.notes.retain(|n| n.id != id);
        self.save();
    }

    pub fn add_reminder(&mut self, text: String, days: Vec<u8>) {
        self.reminders.push(Reminder {
            id: Self::new_id(),
            text,
            days,
        });
        self.save();
    }

    pub fn remove_reminder(&mut self, id: &str) {
        self.reminders.retain(|r| r.id != id);
        self.save();
    }

    /// Bugün hatırlatılacak olanlar.
    pub fn reminders_today(&self) -> Vec<&Reminder> {
        let wd = weekday_of_today();
        self.reminders
            .iter()
            .filter(|r| r.days.contains(&wd))
            .collect()
    }

    /// Alışkanlığın son `periods` dönemdeki durumları: (dönem başı tarihi, yapıldı mı).
    /// Eskiden yeniye sıralı. Dönem başı: Daily = gün, Weekly = haftanın herhangi bir
    /// günü (anahtar haftanın Pazartesisini verir), Monthly = ayın herhangi bir günü.
    pub fn habit_history(&self, habit: &Habit, periods: usize) -> Vec<(NaiveDate, bool)> {
        let mut out = Vec::with_capacity(periods);
        for i in 0..periods {
            let date = match habit.frequency {
                Frequency::Daily => today() - Duration::days(i as i64),
                Frequency::Weekly => today() - Duration::weeks(i as i64),
                Frequency::Monthly => today() - Months::new(i as u32),
            };
            out.push((date, self.habit_done(habit, date)));
        }
        out.reverse();
        out
    }

    /// Son `days` gün için (tarih, o gün tamamlanan günlük alışkanlık + rutin sayısı, toplam).
    /// Eskiden yeniye sıralı.
    pub fn daily_history(&self, days: usize) -> Vec<(NaiveDate, usize, usize)> {
        let daily_total = self
            .habits
            .iter()
            .filter(|h| h.frequency == Frequency::Daily)
            .count()
            + self.routine_items.len();
        let mut out = Vec::with_capacity(days);
        for i in 0..days {
            let date = today() - Duration::days((days - 1 - i) as i64);
            let mut done = 0;
            for habit in self
                .habits
                .iter()
                .filter(|h| h.frequency == Frequency::Daily)
            {
                if self.habit_done(habit, date) {
                    done += 1;
                }
            }
            for item in &self.routine_items {
                if self.routine_done(&item.id, date) {
                    done += 1;
                }
            }
            out.push((date, done, daily_total));
        }
        out
    }

    // Pomodoro test yardımı: fazın başlangıcını şimdiden `secs` saniye geriye çeker.
    #[cfg(test)]
    pub fn pomodoro_phase_start_shift(&mut self, secs: i64) {
        if let Some(p) = self.pomodoro.as_mut() {
            p.phase_start -= secs;
        }
    }

    // --- Schedule (saatler) ---

    pub fn add_schedule(&mut self, name: String, start: String, end: String, days: Vec<u8>) {
        self.schedule.push(ScheduleEntry {
            id: Self::new_id(),
            name,
            start,
            end,
            days,
        });
        self.save();
    }

    pub fn remove_schedule(&mut self, id: &str) {
        self.schedule.retain(|s| s.id != id);
        self.save();
    }

    /// Verilen güne (0..=6) denk gelen girişler, saat sırasına dizili.
    pub fn schedule_for(&self, weekday: u8) -> Vec<ScheduleEntry> {
        let mut entries: Vec<ScheduleEntry> = self
            .schedule
            .iter()
            .filter(|s| s.days.contains(&weekday))
            .cloned()
            .collect();
        entries.sort_by(|a, b| a.start.cmp(&b.start));
        entries
    }

    /// Bugünün programı.
    pub fn schedule_today(&self) -> Vec<ScheduleEntry> {
        let weekday = today().weekday().num_days_from_monday() as u8;
        self.schedule_for(weekday)
    }

    pub fn add_routine(&mut self, name: String) {
        self.routine_items.push(RoutineItem {
            id: Self::new_id(),
            name,
            created_at: today(),
        });
        self.save();
    }

    pub fn remove_routine(&mut self, id: &str) {
        self.routine_items.retain(|r| r.id != id);
        self.routine_checks.remove(id);
        self.save();
    }

    pub fn routine_done(&self, id: &str, date: NaiveDate) -> bool {
        self.routine_checks
            .get(id)
            .is_some_and(|set| set.contains(&date))
    }

    pub fn toggle_routine(&mut self, id: &str) {
        let date = today();
        let set = self.routine_checks.entry(id.to_string()).or_default();
        if !set.remove(&date) {
            set.insert(date);
        }
        self.save();
    }
}

impl gpui_kit::Global for DashboardData {}

fn habit_done_impl(set: &BTreeSet<String>, frequency: Frequency, date: NaiveDate) -> bool {
    set.contains(&period_key(date, frequency))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn habit(frequency: Frequency) -> (DashboardData, String) {
        let mut data = DashboardData::default();
        data.add_habit("Test".into(), frequency);
        let id = data.habits[0].id.clone();
        (data, id)
    }

    #[test]
    fn period_keys() {
        // 2026-10-26 Pazartesi
        assert_eq!(
            period_key(date(2026, 10, 26), Frequency::Daily),
            "2026-10-26"
        );
        assert_eq!(
            period_key(date(2026, 10, 25), Frequency::Weekly),
            "2026-10-19"
        );
        assert_eq!(
            period_key(date(2026, 10, 31), Frequency::Weekly),
            "2026-10-26"
        );
        assert_eq!(
            period_key(date(2026, 10, 26), Frequency::Monthly),
            "2026-10"
        );
    }

    #[test]
    fn weekly_key_is_monday() {
        // 2026-10-25 Pazar, aynı hafta
        assert_eq!(
            period_start(date(2026, 10, 25), Frequency::Weekly),
            date(2026, 10, 19)
        );
    }

    #[test]
    fn toggle_and_streak_daily() {
        let (mut data, id) = habit(Frequency::Daily);
        assert_eq!(data.habit_streak(&data.habits[0]), 0);
        data.toggle_habit(&id);
        // bugün işaretli, seri 1
        assert_eq!(data.habit_streak(&data.habits[0]), 1);
        // dünü de işaretleyelim
        let yesterday = previous_period(today(), Frequency::Daily);
        let key = period_key(yesterday, Frequency::Daily);
        data.habit_completions.get_mut(&id).unwrap().insert(key);
        assert_eq!(data.habit_streak(&data.habits[0]), 2);
        data.toggle_habit(&id);
        assert!(!data.habit_done(&data.habits[0], today()));
    }

    #[test]
    fn routine_dates_reset_each_day() {
        let mut data = DashboardData::default();
        data.add_routine("Spor".into());
        let id = data.routine_items[0].id.clone();
        data.toggle_routine(&id);
        assert!(data.routine_done(&id, today()));
        // yarısı başka gün
        let other = today() - Duration::days(1);
        assert!(!data.routine_done(&id, other));
    }

    #[test]
    fn remove_cleans_completions() {
        let (mut data, id) = habit(Frequency::Weekly);
        data.toggle_habit(&id);
        data.remove_habit(&id);
        assert!(data.habits.is_empty());
        assert!(data.habit_completions.is_empty());
    }

    fn entry(name: &str, start: &str, end: &str, days: Vec<u8>) -> ScheduleEntry {
        ScheduleEntry {
            id: format!("t-{name}"),
            name: name.into(),
            start: start.into(),
            end: end.into(),
            days,
        }
    }

    #[test]
    fn parse_time() {
        assert_eq!(ScheduleEntry::parse_time("9:05"), Some("09:05".into()));
        assert_eq!(ScheduleEntry::parse_time("23:59"), Some("23:59".into()));
        assert_eq!(ScheduleEntry::parse_time("24:00"), None);
        assert_eq!(ScheduleEntry::parse_time("12:60"), None);
        assert_eq!(ScheduleEntry::parse_time("abc"), None);
    }

    #[test]
    fn reminders_and_notes() {
        let mut data = DashboardData::default();
        data.add_note("market listesi".into());
        data.add_reminder("çöp çıkar".into(), vec![0, 3]);
        data.add_reminder("araba yıka".into(), vec![6]);
        // bugün pazartesi (0): çöp çıkar hatırlatılır, araba değil
        let today = data.reminders_today();
        assert_eq!(today.len(), 1);
        assert_eq!(today[0].text, "çöp çıkar");

        let note_id = data.notes[0].id.clone();
        data.remove_note(&note_id);
        assert!(data.notes.is_empty());
        let r_id = data.reminders[0].id.clone();
        data.remove_reminder(&r_id);
        assert_eq!(data.reminders.len(), 1);
    }

    #[test]
    fn pomodoro_advance_and_remaining() {
        let mut data = DashboardData::default();
        assert!(data.pomodoro_remaining().is_none());
        data.start_pomodoro("test".into(), 1, 2); // 1 dk odak
        data.pomodoro_phase_start_shift(10_000); // fazı eskiye çek
        assert_eq!(data.pomodoro_remaining().unwrap().1, 0); // süre doldu
                                                             // advance ile mola faza geçer, tur sayılır
        assert!(data.advance_pomodoro());
        let state = data.pomodoro_remaining().unwrap();
        assert_eq!(state.0, PomodoroPhase::Break);
        assert_eq!(state.3, 1);
        // manuel skip: moladan odağa
        data.skip_pomodoro_phase();
        assert_eq!(data.pomodoro_remaining().unwrap().0, PomodoroPhase::Work);
        // durdurunca None
        data.stop_pomodoro();
        assert!(data.pomodoro_remaining().is_none());
    }

    #[test]
    fn schedule_filter_and_sort() {
        let mut data = DashboardData::default();
        // 2026-10-26 Pazartesi (weekday=0)
        data.schedule = vec![
            entry("akşam", "19:00", "20:00", vec![0, 3]),
            entry("sabah", "07:00", "08:00", vec![0]),
            entry("öğlen", "12:00", "13:00", vec![6]), // sadece pazar
        ];
        let today = data.schedule_today();
        // saat sırasına dizilmiş ve sadece pazartesi olanlar
        assert_eq!(today.len(), 2);
        assert_eq!(today[0].name, "sabah");
        assert_eq!(today[1].name, "akşam");

        // diğer günler (çarşamba = 2) boş
        assert!(data.schedule_for(2).is_empty());
        // pazar (3) sadece öğlen
        assert_eq!(data.schedule_for(6).len(), 1);
        data.remove_schedule("t-akşam");
        assert_eq!(data.schedule.len(), 2);
    }

    #[test]
    fn habit_history_counts_periods() {
        let (mut data, id) = habit(Frequency::Daily);
        let habit = data.habits[0].clone();
        data.habit_completions
            .entry(id.clone())
            .or_default()
            .insert("2026-10-25".to_string()); // dün
        data.habit_completions
            .entry(id)
            .or_default()
            .insert("2026-10-23".to_string()); // iki gün önce
        let history = data.habit_history(&habit, 7);
        assert_eq!(history.len(), 7);
        // eskiden yeniye sıralı: son gün bugün, yapılmadı
        assert!(!history[6].1);
        assert!(history[5].1); // dün
        assert!(!history[4].1);
        assert!(history[3].1); // iki gün önce

        let rate = data.daily_history(7);
        assert_eq!(rate.len(), 7);
        // son gün: 0/1 (alışkanlık bugün yapılmadı)
        assert_eq!(rate[6].0, today());
        assert_eq!(rate[6].1, 0);
        assert_eq!(rate[6].2, 1);
    }

    #[test]
    fn routine_week_count() {
        let mut data = DashboardData::default();
        data.routine_checks
            .entry("t-r1".to_string())
            .or_default()
            .insert(date(2026, 10, 26));
        data.routine_checks
            .entry("t-r1".to_string())
            .or_default()
            .insert(date(2026, 9, 1)); // 30 günden eski
        assert_eq!(data.routine_week_count("t-r1", 30), 1);
        assert_eq!(data.routine_week_count("t-yok", 30), 0);
    }
}
