use crate::imports::*;
use std::sync::OnceLock;
use crate::runtime::try_runtime;

/// Repaint performance configuration
/// Can be adjusted based on different usage scenarios and device performance
#[derive(Debug, Clone)]
pub struct RepaintConfig {
    /// Target frame rate
    pub target_fps: u64,
    /// Minimum repaint interval (milliseconds)
    pub min_repaint_interval_ms: u64,
    /// Maximum repaint delay (milliseconds)
    pub max_repaint_delay_ms: u64,
    /// Batch update threshold
    pub batch_update_threshold: usize,
    /// Whether to enable smart repaint
    pub smart_repaint_enabled: bool,
    /// Whether to enable batch updates
    pub batch_update_enabled: bool,
}

impl Default for RepaintConfig {
    fn default() -> Self {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Desktop version: high performance configuration
                Self {
                    target_fps: 60,
                    min_repaint_interval_ms: 16, // About 60 FPS
                    max_repaint_delay_ms: 50,    // Maximum 50ms delay
                    batch_update_threshold: 3,   // Batch repaint after 3 updates
                    smart_repaint_enabled: true,
                    batch_update_enabled: true,
                }
            } else {
                // Web version: balanced performance configuration
                Self {
                    target_fps: 30,
                    min_repaint_interval_ms: 33, // About 30 FPS
                    max_repaint_delay_ms: 100,   // Maximum 100ms delay
                    batch_update_threshold: 5,   // Batch repaint after 5 updates
                    smart_repaint_enabled: true,
                    batch_update_enabled: true,
                }
            }
        }
    }
}

impl RepaintConfig {
    /// High performance mode configuration
    pub fn high_performance() -> Self {
        Self {
            target_fps: 120,
            min_repaint_interval_ms: 8,  // About 120 FPS
            max_repaint_delay_ms: 25,    // Maximum 25ms delay
            batch_update_threshold: 2,   // Batch repaint after 2 updates
            smart_repaint_enabled: true,
            batch_update_enabled: true,
        }
    }

    /// Power saving mode configuration
    pub fn power_saving() -> Self {
        Self {
            target_fps: 30,
            min_repaint_interval_ms: 33, // About 30 FPS
            max_repaint_delay_ms: 200,   // Maximum 200ms delay
            batch_update_threshold: 10,  // Batch repaint after 10 updates
            smart_repaint_enabled: true,
            batch_update_enabled: true,
        }
    }

    /// Debug mode configuration
    pub fn debug() -> Self {
        Self {
            target_fps: 60,
            min_repaint_interval_ms: 16,
            max_repaint_delay_ms: 50,
            batch_update_threshold: 1,   // Immediate repaint when debugging
            smart_repaint_enabled: false, // Disable smart repaint when debugging
            batch_update_enabled: false,  // Disable batch updates when debugging
        }
    }

    /// Calculate repaint interval
    pub fn repaint_interval_ms(&self) -> u64 {
        1000 / self.target_fps
    }
}

// Global repaint configuration
static REPAINT_CONFIG: OnceLock<Arc<Mutex<RepaintConfig>>> = OnceLock::new();

/// Get global repaint configuration
pub fn get_repaint_config() -> Arc<Mutex<RepaintConfig>> {
    REPAINT_CONFIG
        .get_or_init(|| Arc::new(Mutex::new(RepaintConfig::default())))
        .clone()
}

/// Set global repaint configuration
pub fn set_repaint_config(config: RepaintConfig) {
    if let Some(existing_config) = REPAINT_CONFIG.get() {
        if let Ok(mut config_mutex) = existing_config.lock() {
            *config_mutex = config;
        }
    } else {
        let _ = REPAINT_CONFIG.set(Arc::new(Mutex::new(config)));
    }
}

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub const TARGET_FPS: u64 = 60;
    } else {
        pub const TARGET_FPS: u64 = 30;
    }
}

pub const REPAINT_INTERVAL_MILLIS: u64 = 1000 / TARGET_FPS;

// Smart repaint configuration - get from global configuration
pub fn smart_repaint_enabled() -> bool {
    if let Ok(config) = get_repaint_config().lock() {
        config.smart_repaint_enabled
    } else {
        true // Default enabled
    }
}

pub fn min_repaint_interval_millis() -> u64 {
    if let Ok(config) = get_repaint_config().lock() {
        config.min_repaint_interval_ms
    } else {
        16 // Default 16ms
    }
}

pub fn max_repaint_delay_millis() -> u64 {
    if let Ok(config) = get_repaint_config().lock() {
        config.max_repaint_delay_ms
    } else {
        100 // Default 100ms
    }
}

pub fn batch_update_threshold() -> usize {
    if let Ok(config) = get_repaint_config().lock() {
        config.batch_update_threshold
    } else {
        3 // Default 3
    }
}

// Smart repaint configuration constants
pub const SMART_REPAINT_ENABLED: bool = true;
pub const MIN_REPAINT_INTERVAL_MILLIS: u64 = 16; // Minimum repaint interval (about 60 FPS)
pub const MAX_REPAINT_DELAY_MILLIS: u64 = 100;   // Maximum repaint delay
pub const BATCH_UPDATE_THRESHOLD: usize = 3;     // Batch update threshold

pub enum RepaintServiceEvents {
    Exit,
}

pub struct RepaintService {
    pub application_events: ApplicationEventsChannel,
    pub service_events: Channel<RepaintServiceEvents>,
    pub task_ctl: Channel<()>,
    pub repaint: Arc<AtomicBool>,
    // Smart repaint control
    pub last_repaint: Arc<Mutex<Instant>>,
    pub pending_repaint: Arc<AtomicBool>,
    pub repaint_timer: Arc<Mutex<Option<tokio::time::Sleep>>>,
    // Batch update control
    pub pending_updates: Arc<AtomicUsize>,
    pub batch_timer: Arc<Mutex<Option<tokio::time::Sleep>>>,
}

impl RepaintService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        Self {
            application_events,
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            repaint: Arc::new(AtomicBool::new(false)),
            last_repaint: Arc::new(Mutex::new(Instant::now())),
            pending_repaint: Arc::new(AtomicBool::new(false)),
            repaint_timer: Arc::new(Mutex::new(None)),
            pending_updates: Arc::new(AtomicUsize::new(0)),
            batch_timer: Arc::new(Mutex::new(None)),
        }
    }

    pub fn trigger(&self) {
        self.repaint.store(true, Ordering::SeqCst);
    }

    pub fn clear(&self) {
        self.repaint.store(false, Ordering::SeqCst);
    }

    /// Batch update repaint - collect multiple updates then repaint once
    pub fn batch_update(&self) {
        if !batch_update_enabled() {
            self.smart_trigger();
            return;
        }

        let current_updates = self.pending_updates.fetch_add(1, Ordering::SeqCst) + 1;
        let threshold = batch_update_threshold();
        
        if current_updates >= threshold {
            // Reached threshold, repaint immediately
            self.force_repaint();
            self.pending_updates.store(0, Ordering::SeqCst);
        } else {
            // Use more efficient delay mechanism to avoid frequent async task creation
            self.schedule_delayed_repaint();
        }
    }

    /// Optimized delayed repaint scheduling
    fn schedule_delayed_repaint(&self) {
        // Check if a delayed repaint is already scheduled
        if self.pending_repaint.load(Ordering::SeqCst) {
            return; // Avoid duplicate scheduling
        }

        let pending_updates = self.pending_updates.clone();
        let repaint_service = self.clone();
        let max_delay = max_repaint_delay_millis();
        
        // Mark as pending repaint
        self.pending_repaint.store(true, Ordering::SeqCst);
        
        // Use shorter delay for better responsiveness
        let delay = (max_delay / 4).max(10); // Minimum 10ms delay
        
        // Cancel previous batch timer (if any)
        if let Ok(mut timer_guard) = self.batch_timer.lock() {
            timer_guard.take();
        }
        
        // Create new delayed task
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            
            // Check if repaint is still needed
            if pending_updates.load(Ordering::SeqCst) > 0 {
                repaint_service.force_repaint();
                pending_updates.store(0, Ordering::SeqCst);
            }
        });
    }

    /// Optimized smart repaint trigger
    pub fn smart_trigger(&self) {
        if !smart_repaint_enabled() {
            self.trigger();
            return;
        }

        let now = Instant::now();
        let min_interval = min_repaint_interval_millis();
        
        // More efficient lock strategy
        let should_delay = {
            if let Ok(last_repaint) = self.last_repaint.try_lock() {
                let elapsed = now.duration_since(*last_repaint).as_millis() as u64;
                elapsed < min_interval
            } else {
                // If lock fails, trigger repaint immediately
                false
            }
        };

        if should_delay {
            // Delay repaint
            self.pending_repaint.store(true, Ordering::SeqCst);
            
            // Calculate delay time
            let delay = {
                if let Ok(last_repaint) = self.last_repaint.lock() {
                    let elapsed = now.duration_since(*last_repaint).as_millis() as u64;
                    min_interval.saturating_sub(elapsed)
                } else {
                    min_interval
                }
            };
            
            // Use more efficient delay mechanism
            self.schedule_smart_delayed_repaint(delay, now);
        } else {
            // Trigger repaint immediately
            self.trigger();
            if let Ok(mut last_repaint) = self.last_repaint.try_lock() {
                *last_repaint = now;
            }
        }
    }

    /// Optimized smart delayed repaint
    fn schedule_smart_delayed_repaint(&self, delay: u64, now: Instant) {
        let pending_repaint = self.pending_repaint.clone();
        let last_repaint = self.last_repaint.clone();
        let repaint_service = self.clone();
        
        // Cancel previous timer
        if let Ok(mut timer_guard) = self.repaint_timer.lock() {
            timer_guard.take();
        }
        
        // Create delayed task
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            
            if pending_repaint.load(Ordering::SeqCst) {
                pending_repaint.store(false, Ordering::SeqCst);
                repaint_service.trigger();
                
                // Update last repaint time
                if let Ok(mut last_repaint_guard) = last_repaint.lock() {
                    *last_repaint_guard = now;
                }
            }
        });
    }

    /// Force repaint immediately (for critical updates)
    pub fn force_repaint(&self) {
        self.trigger();
        *self.last_repaint.lock().unwrap() = Instant::now();
        self.pending_repaint.store(false, Ordering::SeqCst);
        self.pending_updates.store(0, Ordering::SeqCst);
    }

    /// Check if repaint is needed
    pub fn needs_repaint(&self) -> bool {
        self.repaint.load(Ordering::SeqCst) || 
        self.pending_repaint.load(Ordering::SeqCst) ||
        self.pending_updates.load(Ordering::SeqCst) > 0
    }

    /// Get current repaint statistics
    pub fn get_stats(&self) -> RepaintStats {
        RepaintStats {
            pending_repaint: self.pending_repaint.load(Ordering::SeqCst),
            pending_updates: self.pending_updates.load(Ordering::SeqCst),
            last_repaint_ago: self.last_repaint.lock()
                .map(|last| last.elapsed().as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

/// Repaint statistics info
#[derive(Debug, Clone)]
pub struct RepaintStats {
    pub pending_repaint: bool,
    pub pending_updates: usize,
    pub last_repaint_ago: u64,
}

// Helper function
fn batch_update_enabled() -> bool {
    if let Ok(config) = get_repaint_config().lock() {
        config.batch_update_enabled
    } else {
        true // Default enabled
    }
}


impl Clone for RepaintService {
    fn clone(&self) -> Self {
        Self {
            application_events: self.application_events.clone(),
            service_events: Channel::unbounded(),
            task_ctl: Channel::oneshot(),
            repaint: self.repaint.clone(),
            last_repaint: self.last_repaint.clone(),
            pending_repaint: self.pending_repaint.clone(),
            repaint_timer: self.repaint_timer.clone(),
            pending_updates: self.pending_updates.clone(),
            batch_timer: self.batch_timer.clone(),
        }
    }
}

#[async_trait]
impl Service for RepaintService {
    fn name(&self) -> &'static str {
        "repaint-service"
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        let _application_events_sender = self.application_events.sender.clone();
        let interval = task::interval(Duration::from_millis(REPAINT_INTERVAL_MILLIS));
        pin_mut!(interval);

        loop {
            select! {
                _ = interval.next().fuse() => {
                    if self.repaint.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                        runtime().egui_ctx().request_repaint();
                    }
                },
                msg = self.as_ref().service_events.receiver.recv().fuse() => {
                    if let Ok(event) = msg {
                        match event {
                            RepaintServiceEvents::Exit => {
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.task_ctl.send(()).await.unwrap();
        Ok(())
    }

    fn terminate(self: Arc<Self>) {
        self.service_events
            .sender
            .try_send(RepaintServiceEvents::Exit)
            .unwrap();
    }

    async fn join(self: Arc<Self>) -> Result<()> {
        self.task_ctl.recv().await.unwrap();
        Ok(())
    }
}
/// Smart repaint configuration options
#[derive(Debug, Clone)]
pub struct SmartRepaintConfig {
    pub enabled: bool,
    pub min_interval_ms: u64,
    pub max_delay_ms: u64,
    pub batch_size: usize,
}

impl Default for SmartRepaintConfig {
    fn default() -> Self {
        Self {
            enabled: SMART_REPAINT_ENABLED,
            min_interval_ms: MIN_REPAINT_INTERVAL_MILLIS,
            max_delay_ms: MAX_REPAINT_DELAY_MILLIS,
            batch_size: 10,
        }
    }
}

/// Global smart repaint configuration
pub static SMART_REPAINT_CONFIG: OnceLock<Arc<Mutex<SmartRepaintConfig>>> = OnceLock::new();

/// Get global smart repaint configuration
pub fn get_smart_repaint_config() -> Arc<Mutex<SmartRepaintConfig>> {
    SMART_REPAINT_CONFIG
        .get_or_init(|| Arc::new(Mutex::new(SmartRepaintConfig::default())))
        .clone()
}

/// Smart repaint utility functions
pub mod utils {
    use super::*;

    /// Smart repaint request - automatically merges multiple calls within a short time
    pub fn smart_request_repaint() {
        if let Some(runtime) = try_runtime() {
            runtime.request_repaint();
        }
    }

    /// Force repaint request - execute immediately without merging
    pub fn force_request_repaint() {
        if let Some(runtime) = try_runtime() {
            runtime.force_repaint();
        }
    }

    /// Batch repaint request - collect multiple updates and repaint once
    pub fn batch_request_repaint<F>(updates: F) 
    where 
        F: FnOnce() + Send + 'static 
    {
        if let Some(runtime) = try_runtime() {
            // Execute all updates
            updates();
            // Then trigger repaint
            runtime.request_repaint();
        }
    }

    /// Delayed repaint request - repaint after a specified delay
    pub fn delayed_request_repaint(delay_ms: u64) {
        if let Some(runtime) = try_runtime() {
            let runtime_clone = runtime.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                runtime_clone.request_repaint();
            });
        }
    }
}

/// Repaint priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RepaintPriority {
    Low,      // Low priority - can be merged and delayed
    Normal,   // Normal priority - smart merge
    High,     // High priority - repaint immediately
    Critical, // Critical priority - force repaint immediately
}

impl RepaintService {
    /// Trigger repaint according to priority
    pub fn trigger_with_priority(&self, priority: RepaintPriority) {
        match priority {
            RepaintPriority::Low => {
                // Low priority: use batch update
                self.batch_update();
            }
            RepaintPriority::Normal => {
                // Normal priority: smart repaint
                self.smart_trigger();
            }
            RepaintPriority::High => {
                // High priority: repaint immediately
                self.trigger();
            }
            RepaintPriority::Critical => {
                // Critical priority: force repaint
                self.force_repaint();
            }
        }
    }

    /// Request repaint (compatibility method)
    pub fn request_repaint(&self) {
        self.smart_trigger();
    }
}

/// Repaint performance monitor
pub struct RepaintMonitor {
    total_repaints: Arc<AtomicUsize>,
    total_updates: Arc<AtomicUsize>,
    start_time: Instant,
    config: Arc<Mutex<RepaintConfig>>,
}

impl RepaintMonitor {
    pub fn new() -> Self {
        Self {
            total_repaints: Arc::new(AtomicUsize::new(0)),
            total_updates: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            config: get_repaint_config(),
        }
    }

    /// Record repaint event
    pub fn record_repaint(&self) {
        self.total_repaints.fetch_add(1, Ordering::SeqCst);
    }

    /// Record update event
    pub fn record_update(&self) {
        self.total_updates.fetch_add(1, Ordering::SeqCst);
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let runtime = self.start_time.elapsed();
        let repaints = self.total_repaints.load(Ordering::SeqCst);
        let updates = self.total_updates.load(Ordering::SeqCst);
        
        PerformanceStats {
            runtime_seconds: runtime.as_secs_f64(),
            total_repaints: repaints,
            total_updates: updates,
            repaints_per_second: if runtime.as_secs() > 0 { 
                repaints as f64 / runtime.as_secs() as f64 
            } else { 
                0.0 
            },
            updates_per_second: if runtime.as_secs() > 0 { 
                updates as f64 / runtime.as_secs() as f64 
            } else { 
                0.0 
            },
            efficiency_ratio: if updates > 0 { 
                repaints as f64 / updates as f64 
            } else { 
                0.0 
            },
        }
    }

    /// Reset statistics
    pub fn reset(&mut self) {
        self.total_repaints.store(0, Ordering::SeqCst);
        self.total_updates.store(0, Ordering::SeqCst);
        self.start_time = Instant::now();
    }

    /// Print performance report
    pub fn print_report(&self) {
        let stats = self.get_performance_stats();
        println!("=== Repaint Performance Report ===");
        println!("Runtime: {:.2} seconds", stats.runtime_seconds);
        println!("Total repaints: {}", stats.total_repaints);
        println!("Total updates: {}", stats.total_updates);
        println!("Repaint frequency: {:.2} /sec", stats.repaints_per_second);
        println!("Update frequency: {:.2} /sec", stats.updates_per_second);
        println!("Efficiency ratio: {:.2} (repaints/updates)", stats.efficiency_ratio);
        
        if let Ok(config) = self.config.lock() {
            println!("Current config:");
            println!("  Target FPS: {}", config.target_fps);
            println!("  Min repaint interval: {}ms", config.min_repaint_interval_ms);
            println!("  Batch update threshold: {}", config.batch_update_threshold);
        }
        println!("==================");
    }
}

/// Performance statistics info
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub runtime_seconds: f64,
    pub total_repaints: usize,
    pub total_updates: usize,
    pub repaints_per_second: f64,
    pub updates_per_second: f64,
    pub efficiency_ratio: f64,
}

// Global performance monitor
static REPAINT_MONITOR: OnceLock<Arc<RepaintMonitor>> = OnceLock::new();

/// Get global repaint performance monitor
pub fn get_repaint_monitor() -> Arc<RepaintMonitor> {
    REPAINT_MONITOR
        .get_or_init(|| Arc::new(RepaintMonitor::new()))
        .clone()
}

/// Record repaint event (for performance monitoring)
pub fn record_repaint_event() {
    if let Some(monitor) = REPAINT_MONITOR.get() {
        monitor.record_repaint();
    }
}

/// Record update event (for performance monitoring)
pub fn record_update_event() {
    if let Some(monitor) = REPAINT_MONITOR.get() {
        monitor.record_update();
    }
}

/// Print performance report
pub fn print_performance_report() {
    if let Some(monitor) = REPAINT_MONITOR.get() {
        monitor.print_report();
    }
}

/// Repaint optimization suggestion generator
pub struct RepaintOptimizer {
    monitor: Arc<RepaintMonitor>,
}

impl RepaintOptimizer {
    pub fn new() -> Self {
        Self {
            monitor: get_repaint_monitor(),
        }
    }

    /// Analyze current performance and generate optimization suggestions
    pub fn analyze_and_suggest(&self) -> Vec<OptimizationSuggestion> {
        let stats = self.monitor.get_performance_stats();
        let mut suggestions = Vec::new();

        // Analyze repaint frequency
        if stats.repaints_per_second > 60.0 {
            suggestions.push(OptimizationSuggestion {
                category: "Repaint frequency too high".to_string(),
                description: "Current repaint frequency exceeds 60FPS, which may cause performance waste".to_string(),
                suggestion: "Consider increasing the minimum repaint interval or enabling batch updates".to_string(),
                priority: SuggestionPriority::High,
            });
        }

        // Analyze efficiency ratio
        if stats.efficiency_ratio > 0.5 {
            suggestions.push(OptimizationSuggestion {
                category: "Low repaint efficiency".to_string(),
                description: format!("Repaint/update ratio: {:.2}, too many unnecessary repaints", stats.efficiency_ratio),
                suggestion: "Optimize update logic, reduce duplicate updates, use batch updates".to_string(),
                priority: SuggestionPriority::Medium,
            });
        }

        // Analyze update frequency
        if stats.updates_per_second > 100.0 {
            suggestions.push(OptimizationSuggestion {
                category: "Update frequency too high".to_string(),
                description: "Update frequency is too high, which may cause performance issues".to_string(),
                suggestion: "Consider using debounce or throttling to reduce update frequency".to_string(),
                priority: SuggestionPriority::Medium,
            });
        }

        // Positive feedback if no issues
        if suggestions.is_empty() {
            suggestions.push(OptimizationSuggestion {
                category: "Good performance".to_string(),
                description: "Current repaint performance is good".to_string(),
                suggestion: "Keep the current optimization strategy".to_string(),
                priority: SuggestionPriority::Low,
            });
        }

        suggestions
    }

    /// Automatically adjust configuration parameters
    pub fn auto_tune_config(&self) -> RepaintConfig {
        let stats = self.monitor.get_performance_stats();
        let mut config = RepaintConfig::default();

        // Auto adjust based on performance
        if stats.efficiency_ratio > 0.7 {
            // Low efficiency, increase batch threshold
            config.batch_update_threshold = (config.batch_update_threshold * 3) / 2;
            config.max_repaint_delay_ms = (config.max_repaint_delay_ms * 3) / 2;
        } else if stats.efficiency_ratio < 0.2 {
            // High efficiency, reduce delay
            config.batch_update_threshold = (config.batch_update_threshold * 2) / 3;
            config.max_repaint_delay_ms = (config.max_repaint_delay_ms * 2) / 3;
        }

        // Adjust based on repaint frequency
        if stats.repaints_per_second > 60.0 {
            config.min_repaint_interval_ms = (config.min_repaint_interval_ms * 3) / 2;
        } else if stats.repaints_per_second < 20.0 {
            config.min_repaint_interval_ms = (config.min_repaint_interval_ms * 2) / 3;
        }

        config
    }
}

/// Optimization suggestion
#[derive(Debug, Clone)]
pub struct OptimizationSuggestion {
    pub category: String,
    pub description: String,
    pub suggestion: String,
    pub priority: SuggestionPriority,
}

/// Suggestion priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
}

impl OptimizationSuggestion {
    pub fn print(&self) {
        let priority_symbol = match self.priority {
            SuggestionPriority::High => "🔴",
            SuggestionPriority::Medium => "🟡",
            SuggestionPriority::Low => "🟢",
        };
        
        println!("{} {}: {}", priority_symbol, self.category, self.description);
        println!("   Suggestion: {}", self.suggestion);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_smart_repaint_merging() {
        let repaint_service = RepaintService::new(
            ApplicationEventsChannel::new(),
            &Settings::default(),
        );

        // Simulate rapid consecutive repaint requests
        let start = Instant::now();
        
        // Trigger multiple consecutive repaints
        for _ in 0..10 {
            repaint_service.smart_trigger();
        }
        
        // Wait a short period
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        // Verify repaint flag status
        assert!(repaint_service.repaint.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_batch_update() {
        let repaint_service = RepaintService::new(
            ApplicationEventsChannel::new(),
            &Settings::default(),
        );

        // Simulate batch updates
        for _ in 0..batch_update_threshold() {
            repaint_service.batch_update();
        }
        
        // Verify repaint is triggered after batch update
        assert!(repaint_service.repaint.load(Ordering::SeqCst));
        assert_eq!(repaint_service.pending_updates.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn test_priority_repaint() {
        let repaint_service = RepaintService::new(
            ApplicationEventsChannel::new(),
            &Settings::default(),
        );

        // Test repaints with different priorities
        repaint_service.trigger_with_priority(RepaintPriority::Low);
        repaint_service.trigger_with_priority(RepaintPriority::Normal);
        repaint_service.trigger_with_priority(RepaintPriority::High);
        repaint_service.trigger_with_priority(RepaintPriority::Critical);

        // Wait for async operations to complete
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Verify repaint flag status
        assert!(repaint_service.repaint.load(Ordering::SeqCst));
    }

    #[test]
    fn test_needs_repaint() {
        let repaint_service = RepaintService::new(
            ApplicationEventsChannel::new(),
            &Settings::default(),
        );

        // Initially does not need repaint
        assert!(!repaint_service.needs_repaint());

        // After triggering, repaint is needed
        repaint_service.trigger();
        assert!(repaint_service.needs_repaint());

        // After clearing, repaint is not needed
        repaint_service.clear();
        assert!(!repaint_service.needs_repaint());
    }

    #[test]
    fn test_repaint_config() {
        // Test default config
        let default_config = RepaintConfig::default();
        assert!(default_config.smart_repaint_enabled);
        assert!(default_config.batch_update_enabled);
        
        // Test high performance config
        let high_perf_config = RepaintConfig::high_performance();
        assert_eq!(high_perf_config.target_fps, 120);
        assert_eq!(high_perf_config.min_repaint_interval_ms, 8);
        
        // Test power saving config
        let power_save_config = RepaintConfig::power_saving();
        assert_eq!(power_save_config.target_fps, 30);
        assert_eq!(power_save_config.batch_update_threshold, 10);
        
        // Test debug config
        let debug_config = RepaintConfig::debug();
        assert!(!debug_config.smart_repaint_enabled);
        assert!(!debug_config.batch_update_enabled);
    }

    #[test]
    fn test_config_management() {
        // Test setting and getting config
        let test_config = RepaintConfig {
            target_fps: 45,
            min_repaint_interval_ms: 22,
            max_repaint_delay_ms: 75,
            batch_update_threshold: 4,
            smart_repaint_enabled: true,
            batch_update_enabled: true,
        };
        
        set_repaint_config(test_config.clone());
        
        let retrieved_config = get_repaint_config();
        if let Ok(config) = retrieved_config.lock() {
            assert_eq!(config.target_fps, 45);
            assert_eq!(config.min_repaint_interval_ms, 22);
            assert_eq!(config.batch_update_threshold, 4);
        }
    }

    #[test]
    fn test_performance_monitoring() {
        let monitor = RepaintMonitor::new();
        
        // Record some events
        monitor.record_repaint();
        monitor.record_repaint();
        monitor.record_update();
        monitor.record_update();
        monitor.record_update();
        
        // Get statistics
        let stats = monitor.get_performance_stats();
        assert_eq!(stats.total_repaints, 2);
        assert_eq!(stats.total_updates, 3);
        assert!(stats.efficiency_ratio > 0.0);
    }

    #[test]
    fn test_optimization_suggestions() {
        let optimizer = RepaintOptimizer::new();
        let suggestions = optimizer.analyze_and_suggest();
        
        // Should have at least one suggestion
        assert!(!suggestions.is_empty());
        
        // Print suggestions (for debugging)
        for suggestion in &suggestions {
            suggestion.print();
        }
    }

    #[test]
    fn test_auto_tune_config() {
        let optimizer = RepaintOptimizer::new();
        let tuned_config = optimizer.auto_tune_config();
        
        // Verify config parameters fall within reasonable ranges
        assert!(tuned_config.target_fps >= 30 && tuned_config.target_fps <= 120);
        assert!(tuned_config.min_repaint_interval_ms >= 8 && tuned_config.min_repaint_interval_ms <= 100);
        assert!(tuned_config.batch_update_threshold >= 1 && tuned_config.batch_update_threshold <= 20);
    }

    #[test]
    fn test_repaint_stats() {
        let repaint_service = RepaintService::new(
            ApplicationEventsChannel::new(),
            &Settings::default(),
        );
        
        // Get initial stats
        let initial_stats = repaint_service.get_stats();
        assert!(!initial_stats.pending_repaint);
        assert_eq!(initial_stats.pending_updates, 0);
        
        // Trigger some updates
        repaint_service.batch_update();
        repaint_service.batch_update();
        
        // Get updated stats
        let updated_stats = repaint_service.get_stats();
        assert_eq!(updated_stats.pending_updates, 2);
    }

    #[tokio::test]
    async fn test_integration_scenarios() {
        let repaint_service = RepaintService::new(
            ApplicationEventsChannel::new(),
            &Settings::default(),
        );
        
        // Scenario 1: Rapid consecutive updates
        for i in 0..10 {
            if i % 3 == 0 {
                repaint_service.batch_update();
            } else {
                repaint_service.smart_trigger();
            }
        }
        
        // Wait for async operations
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Verify final state
        let stats = repaint_service.get_stats();
        println!("Integration test results: {:?}", stats);
        
        // There should be at least one repaint
        assert!(repaint_service.repaint.load(Ordering::SeqCst) || stats.pending_updates > 0);
    }
}
