use crate::imports::*;
use tondi_rpc_core::{GetSystemInfoResponse, GetMetricsRequest};
use std::sync::Arc;
use tondi_metrics_core::{Metric, Metrics, MetricsSnapshot};
#[allow(unused_imports)]
use tondi_wallet_core::rpc::{NotificationMode, Rpc, RpcCtl, WrpcEncoding};
use tokio::time::{Duration, interval};

#[allow(clippy::identity_op)]
pub const MAX_METRICS_SAMPLES: usize = 60 * 60 * 24 * 1; // 1 day

pub struct MetricsService {
    pub application_events: ApplicationEventsChannel,
    pub task_ctl: Channel<()>,
    pub metrics: Arc<Metrics>,
    pub metrics_data: Mutex<HashMap<Metric, Vec<PlotPoint>>>,
    pub samples_since_connection: Arc<AtomicUsize>,
    pub rpc_api: Mutex<Option<Arc<dyn RpcApi>>>,
    pub metrics_update_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl MetricsService {
    pub fn new(application_events: ApplicationEventsChannel, _settings: &Settings) -> Self {
        let metrics = Arc::new(Metrics::default());
        let metrics_data = Metric::into_iter()
            .map(|metric| (metric, Vec::new()))
            .collect::<HashMap<Metric, Vec<_>>>();

        Self {
            application_events,
            task_ctl: Channel::oneshot(),
            metrics,
            metrics_data: Mutex::new(metrics_data),
            samples_since_connection: Arc::new(AtomicUsize::new(0)),
            rpc_api: Mutex::new(None),
            metrics_update_task: Mutex::new(None),
        }
    }

    pub fn rpc_api(&self) -> Option<Arc<dyn RpcApi>> {
        self.rpc_api.lock().unwrap().clone()
    }

    pub fn metrics_data(&self) -> MutexGuard<'_, HashMap<Metric, Vec<PlotPoint>>> {
        self.metrics_data.lock().unwrap()
    }

    pub fn metrics(&self) -> &Arc<Metrics> {
        &self.metrics
    }

    pub fn reset_metrics_data(&self) -> Result<()> {
        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::into_iter() {
            metrics_data.insert(metric, Vec::with_capacity(MAX_METRICS_SAMPLES));
        }
        Ok(())
    }

    pub fn ingest_metrics_snapshot(&self, snapshot: Box<MetricsSnapshot>) -> Result<()> {
        let timestamp = snapshot.unixtime_millis;
        let mut metrics_data = self.metrics_data.lock().unwrap();
        for metric in Metric::into_iter() {
            let dest = metrics_data.get_mut(&metric).unwrap();
            if dest.is_empty() {
                if snapshot.duration_millis < 0.0 {
                    continue;
                }
                let y = snapshot.get(&metric);
                // 使用当前时间戳作为基准，向前填充历史数据
                // 每个数据点间隔1秒
                let mut fill_timestamp = timestamp - (MAX_METRICS_SAMPLES - 1) as f64;
                for _ in 0..(MAX_METRICS_SAMPLES - 1) {
                    dest.push(PlotPoint { x: fill_timestamp, y });
                    fill_timestamp += 1.0; // 1秒间隔
                }
            }
            if dest.len() > MAX_METRICS_SAMPLES {
                dest.drain(0..dest.len() - MAX_METRICS_SAMPLES);
            }

            let y = snapshot.get(&metric);
            if y.is_finite() {
                dest.push(PlotPoint { x: timestamp, y });
            } else {
                dest.push(PlotPoint {
                    x: timestamp,
                    y: 0.0,
                });
            }
        }

        if snapshot.node_cpu_cores > 0.0 {
            self.application_events
                .sender
                .try_send(crate::events::Events::MempoolSize {
                    mempool_size: snapshot.get(&Metric::NetworkMempoolSize) as usize,
                })
                .unwrap();

            self.application_events
                .sender
                .try_send(crate::events::Events::Metrics { snapshot })
                .unwrap();
        }

        self.samples_since_connection.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    pub fn samples_since_connection(&self) -> usize {
        self.samples_since_connection.load(Ordering::SeqCst)
    }

    /// Manually start metrics update loop
    /// Because tondi_metrics_core::Metrics may not work correctly, we implement it manually
    async fn start_manual_metrics_update_loop(self: Arc<Self>) -> Result<()> {
        let this = self.clone();
        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1)); // Update once per second
            
            loop {
                interval.tick().await;
                
                // Check if RPC API is available
                if let Some(rpc_api) = this.rpc_api() {
                    // Try to get metrics data
                    let request = GetMetricsRequest {
                        bandwidth_metrics: false,
                        connection_metrics: true,
                        consensus_metrics: true,
                        process_metrics: false,
                        storage_metrics: false,
                        custom_metrics: false,
                    };
                    match rpc_api.get_metrics_call(None, request).await {
                        Ok(metrics_response) => {
                            println!("[METRICS] Successfully got metrics from gRPC: {:?}", metrics_response);
                            
                            // Convert RPC metrics to MetricsSnapshot
                            if let Some(consensus_metrics) = metrics_response.consensus_metrics {
                                // Create simulated MetricsSnapshot
                                let snapshot = this.create_metrics_snapshot_from_rpc(consensus_metrics, metrics_response.connection_metrics);
                                
                                // Process metrics snapshot
                                if let Err(err) = this.ingest_metrics_snapshot(Box::new(snapshot)) {
                                    println!("[METRICS] Error ingesting metrics snapshot: {}", err);
                                }
                            }
                        }
                        Err(e) => {
                            println!("[METRICS] Failed to get metrics from gRPC: {}", e);
                        }
                    }
                } else {
                    println!("[METRICS] No RPC API available, skipping metrics update");
                }
            }
        });
        
        // Store task handle
        self.metrics_update_task.lock().unwrap().replace(handle);
        
        Ok(())
    }

    /// Create MetricsSnapshot from RPC metrics
    fn create_metrics_snapshot_from_rpc(&self, consensus_metrics: tondi_rpc_core::ConsensusMetrics, connection_metrics: Option<tondi_rpc_core::ConnectionMetrics>) -> MetricsSnapshot {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as f64; // 使用秒为单位，与egui_plot期望一致
        
        // Create complete MetricsSnapshot with all necessary fields
        let mut snapshot = MetricsSnapshot::default();
        
        // Basic time information
        snapshot.unixtime_millis = now * 1000.0; // 转换为毫秒，保持与MetricsSnapshot的兼容性
        snapshot.duration_millis = 1000.0; // 1 second update interval
        
        // Network related metrics
        snapshot.network_difficulty = consensus_metrics.network_difficulty;
        snapshot.network_mempool_size = consensus_metrics.network_mempool_size as f64;
        snapshot.network_past_median_time = consensus_metrics.network_past_median_time as f64;
        snapshot.network_tip_hashes_count = consensus_metrics.network_tip_hashes_count as f64;
        snapshot.network_virtual_daa_score = consensus_metrics.network_virtual_daa_score as f64;
        snapshot.network_virtual_parent_hashes_count = consensus_metrics.network_virtual_parent_hashes_count as f64;
        
        // Calculate TPS: based on recent block processing
        let recent_blocks = consensus_metrics.node_chain_blocks_processed_count;
        let recent_transactions = consensus_metrics.node_transactions_processed_count;
        snapshot.network_transactions_per_second = if recent_blocks > 0 {
            (recent_transactions as f64) / (recent_blocks as f64).max(1.0)
        } else {
            0.0
        };
        
        // Node activity metrics - 使用connection_metrics中的active_peers来设置PEERS指标
        if let Some(conn_metrics) = connection_metrics {
            // PEERS指标：使用active_peers + borsh_live_connections + json_live_connections
            snapshot.node_active_peers = (conn_metrics.active_peers + conn_metrics.borsh_live_connections + conn_metrics.json_live_connections) as f64;
        } else {
            // 如果没有connection_metrics，使用mempool_size作为fallback
            snapshot.node_active_peers = consensus_metrics.network_mempool_size as f64;
        }
        
        // Node processing statistics
        snapshot.node_blocks_submitted_count = consensus_metrics.node_blocks_submitted_count as f64;
        snapshot.node_bodies_processed_count = consensus_metrics.node_bodies_processed_count as f64;
        snapshot.node_chain_blocks_processed_count = consensus_metrics.node_chain_blocks_processed_count as f64;
        snapshot.node_database_blocks_count = consensus_metrics.node_database_blocks_count as f64;
        snapshot.node_database_headers_count = consensus_metrics.node_database_headers_count as f64;
        snapshot.node_dependencies_processed_count = consensus_metrics.node_dependencies_processed_count as f64;
        snapshot.node_headers_processed_count = consensus_metrics.node_headers_processed_count as f64;
        snapshot.node_mass_processed_count = consensus_metrics.node_mass_processed_count as f64;
        snapshot.node_transactions_processed_count = consensus_metrics.node_transactions_processed_count as f64;
        
        snapshot
    }
}

#[async_trait]
impl Service for MetricsService {
    fn name(&self) -> &'static str {
        "metrics-service"
    }

    async fn attach_rpc(self: Arc<Self>, rpc_api: &Arc<dyn RpcApi>) -> Result<()> {
        self.rpc_api.lock().unwrap().replace(rpc_api.clone());

        let this = self.clone();
        self.metrics
            .register_sink(Arc::new(Box::new(move |snapshot: MetricsSnapshot| {
                if let Err(err) = this.ingest_metrics_snapshot(Box::new(snapshot)) {
                    println!("Error ingesting metrics snapshot: {}", err);
                }
                None
            })));

        self.reset_metrics_data()?;
        
        // Try to start tondi_metrics_core::Metrics task
        if let Err(e) = self.metrics.start_task().await {
            println!("[METRICS] Warning: tondi_metrics_core::Metrics start_task failed: {}", e);
        }
        
        // Bind RPC API
        self.metrics.bind_rpc(Some(rpc_api.clone()));
        
        // Start our manual metrics update loop as backup solution
        if let Err(e) = self.clone().start_manual_metrics_update_loop().await {
            println!("[METRICS] Warning: Failed to start manual metrics update loop: {}", e);
        }
        
        Ok(())
    }
    async fn detach_rpc(self: Arc<Self>) -> Result<()> {
        self.rpc_api.lock().unwrap().take();

        // Stop manual metrics update task
        if let Some(task_handle) = self.metrics_update_task.lock().unwrap().take() {
            task_handle.abort();
            println!("[METRICS] Manual metrics update task aborted");
        }

        self.metrics.unregister_sink();
        
        // Try to stop tondi_metrics_core::Metrics task
        if let Err(e) = self.metrics.stop_task().await {
            println!("[METRICS] Warning: tondi_metrics_core::Metrics stop_task failed: {}", e);
        }
        
        self.metrics.bind_rpc(None);

        Ok(())
    }

    async fn connect_rpc(self: Arc<Self>) -> Result<()> {
        self.samples_since_connection.store(0, Ordering::SeqCst);

        if let Some(rpc_api) = self.rpc_api() {
            if let Ok(system_info) = rpc_api.get_system_info().await {
                let GetSystemInfoResponse {
                    version, system_id, ..
                } = system_info;

                let system_id = system_id
                    .map(|id| format!(" - {}", id[0..8].to_vec().to_hex()))
                    .unwrap_or_else(|| "".to_string());

                self.application_events
                    .sender
                    .try_send(crate::events::Events::NodeInfo {
                        node_info: Some(Box::new(format!("{}{}", version, system_id))),
                    })
                    .unwrap();
            }
        }

        Ok(())
    }

    async fn disconnect_rpc(self: Arc<Self>) -> Result<()> {
        self.application_events
            .sender
            .try_send(crate::events::Events::NodeInfo { node_info: None })
            .unwrap();
        Ok(())
    }

    async fn spawn(self: Arc<Self>) -> Result<()> {
        Ok(())
    }

    fn terminate(self: Arc<Self>) {}

    async fn join(self: Arc<Self>) -> Result<()> {
        Ok(())
    }
}
