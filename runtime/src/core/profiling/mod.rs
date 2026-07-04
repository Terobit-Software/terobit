use std::collections::HashMap;
use std::time::{Duration, Instant};

/// rolling stats for one profiled label, plus any extra named values attached to it
pub struct ProfilerEntry {
    pub last_ms: f32,
    pub avg_ms: f32,
    pub min_ms: f32,
    pub max_ms: f32,
    /// arbitrary extra data for this label (instance counts, draw calls, whatever),
    /// keyed by name so a label can carry more than one number
    pub values: HashMap<String, f64>,
}

impl ProfilerEntry {
    fn new() -> Self {
        Self { last_ms: 0.0, avg_ms: 0.0, min_ms: f32::MAX, max_ms: 0.0, values: HashMap::new() }
    }

    fn record(&mut self, ms: f32) {
        self.last_ms = ms;
        self.min_ms = self.min_ms.min(ms);
        self.max_ms = self.max_ms.max(ms);

        // exponential moving average, smooths noisy per-frame timings without keeping a history buffer
        const SMOOTHING: f32 = 0.1;
        self.avg_ms += (ms - self.avg_ms) * SMOOTHING;
    }
}

/// general purpose start/stop timer registry. label a section, start it, stop it,
/// read back its stats. also holds arbitrary named "extra values" per label (counts,
/// sizes, anything worth tracking alongside timing). one instance lives on `AppState`
/// and profiles engine internals (physics, rendering); `RuntimeContext` exposes the
/// same instance so game code can profile its own sections the exact same way
pub struct Profiler {
    entries: HashMap<String, ProfilerEntry>,
    /// insertion order, so the hud displays sections in a stable, meaningful order
    order: Vec<String>,
    active: HashMap<String, Instant>,
}

impl Profiler {
    pub fn new() -> Self {
        Self { entries: HashMap::new(), order: Vec::new(), active: HashMap::new() }
    }

    fn entry_mut(&mut self, label: &str) -> &mut ProfilerEntry {
        if !self.entries.contains_key(label) {
            self.order.push(label.to_string());
        }
        self.entries.entry(label.to_string()).or_insert_with(ProfilerEntry::new)
    }

    /// starts timing a labeled section. call `stop` with the same label to record it
    pub fn start(&mut self, label: &str) {
        self.active.insert(label.to_string(), Instant::now());
    }

    /// stops timing a labeled section and records its duration. no-op if `start`
    /// wasn't called first, a profiler should never be able to crash the app
    pub fn stop(&mut self, label: &str) {
        let Some(started) = self.active.remove(label) else { return };
        self.record_duration(label, started.elapsed());
    }

    /// records a duration measured some other way (e.g. a local `Instant` when
    /// interleaving with borrows that don't play nice with `start`/`stop`)
    pub fn record_duration(&mut self, label: &str, duration: Duration) {
        self.entry_mut(label).record(duration.as_secs_f32() * 1000.0);
    }

    /// times a scope for as long as the returned guard lives
    pub fn scope(&mut self, label: &str) -> ProfilerScope<'_> {
        self.start(label);
        ProfilerScope { profiler: self, label: label.to_string() }
    }

    /// attaches an arbitrary named value to a label, independent of timing
    pub fn record_value(&mut self, label: &str, key: &str, value: f64) {
        self.entry_mut(label).values.insert(key.to_string(), value);
    }

    /// all entries in the order their labels were first seen
    pub fn entries(&self) -> impl Iterator<Item = (&str, &ProfilerEntry)> {
        self.order.iter().filter_map(|label| self.entries.get(label).map(|e| (label.as_str(), e)))
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII timing guard returned by `Profiler::scope`, stops the timer on drop
pub struct ProfilerScope<'a> {
    profiler: &'a mut Profiler,
    label: String,
}

impl Drop for ProfilerScope<'_> {
    fn drop(&mut self) {
        self.profiler.stop(&self.label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn stop_without_start_is_noop() {
        let mut p = Profiler::new();
        p.stop("nope");
        assert_eq!(p.entries().count(), 0);
    }

    #[test]
    fn start_stop_records_an_entry() {
        let mut p = Profiler::new();
        p.start("a");
        sleep(Duration::from_millis(1));
        p.stop("a");

        let (_, entry) = p.entries().next().unwrap();
        assert!(entry.last_ms > 0.0);
    }

    #[test]
    fn scope_guard_records_on_drop() {
        let mut p = Profiler::new();
        {
            let _scope = p.scope("b");
            sleep(Duration::from_millis(1));
        }

        let (_, entry) = p.entries().next().unwrap();
        assert!(entry.last_ms > 0.0);
    }

    #[test]
    fn record_value_attaches_without_timing() {
        let mut p = Profiler::new();
        p.record_value("c", "count", 42.0);

        let (_, entry) = p.entries().next().unwrap();
        assert_eq!(entry.values.get("count"), Some(&42.0));
    }

    #[test]
    fn entries_preserve_insertion_order() {
        let mut p = Profiler::new();
        p.record_value("first", "x", 1.0);
        p.record_value("second", "x", 2.0);
        p.record_value("first", "y", 3.0);

        let labels: Vec<&str> = p.entries().map(|(label, _)| label).collect();
        assert_eq!(labels, vec!["first", "second"]);
    }

    #[test]
    fn record_duration_matches_start_stop_shape() {
        let mut p = Profiler::new();
        p.record_duration("d", Duration::from_millis(5));

        let (_, entry) = p.entries().next().unwrap();
        assert!((entry.last_ms - 5.0).abs() < 0.5);
    }
}
