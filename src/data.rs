use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::PathBuf,
};

use chrono::{Datelike, Duration, Local, NaiveDate, Timelike, Weekday};
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
    /// habit id -> completed period keys ("2026-10-26" / week-monday / "2026-10")
    #[serde(default)]
    pub habit_completions: HashMap<String, BTreeSet<String>>,
    /// routine item id -> completed dates
    #[serde(default)]
    pub routine_checks: HashMap<String, BTreeSet<NaiveDate>>,
    /// saat dilimleri
    #[serde(default)]
    pub schedule: Vec<ScheduleEntry>,
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
}
