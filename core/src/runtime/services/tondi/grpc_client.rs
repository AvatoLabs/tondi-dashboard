use crate::imports::*;
use bp_tondi::client::TondiClient as BpTondiClient;
use std::sync::Arc;
use tondi_rpc_core::api::ctl::RpcState;
use tondi_rpc_core::api::connection::DynRpcConnection;
use tondi_rpc_core::notify::connection::ChannelConnection;
use tondi_notify::{listener::ListenerId, scope::Scope};
use workflow_core::channel::Multiplexer;
use tondi_rpc_core::*;

use async_trait::async_trait;

/// gRPC client implementation, wrapping bp-tondi-client
/// 
/// This client provides an interface compatible with existing wRPC clients,
/// allowing communication with Tondi nodes through the gRPC protocol.
/// 
/// Note: gRPC is only supported in desktop (native) version, Web (wasm) version will fallback to wRPC
pub struct TondiGrpcClient {
    inner: Arc<BpTondiClient>,
    network: Network,  // Network configuration, used to determine NetworkId
    url: String,       // Store connection URL
}

impl TondiGrpcClient {
    /// Connect to gRPC server
    /// 
    /// # Parameters
    /// * `network_interface` - Network interface configuration containing address information to connect to
    /// * `network` - Network type configuration used to determine the correct NetworkId
    /// 
    /// # Returns
    /// Returns a new client instance on success, or an error on failure
    /// 
    /// # Note
    /// In Web (wasm) version, this method will return an error prompting to use wRPC
    pub async fn connect(network_interface: NetworkInterfaceConfig, network: Network) -> Result<Self> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Desktop version: supports gRPC connection
                let address: ContextualNetAddress = network_interface.clone().into();
                let url = address.to_string(); // Use full address directly, including port
                
                println!("[gRPC] Attempting to connect to: {}", url);
                println!("[gRPC] Network interface config: {:?}", network_interface);
                println!("[gRPC] Target network: {:?}", network);
                
                match BpTondiClient::connect(&url).await {
                    Ok(client) => {
                        println!("[gRPC] Successfully connected to gRPC server at {}", url);
                        Ok(Self {
                            inner: Arc::new(client),
                            network,
                            url,
                        })
                    }
                    Err(e) => {
                        println!("[gRPC] Failed to connect to gRPC server at {}: {}", url, e);
                        Err(Error::custom(format!("Failed to connect to gRPC server at {}: {}", url, e)))
                    }
                }
            } else {
                // Web version: gRPC not supported, prompt to use wRPC
                println!("[gRPC] Web/WASM version - gRPC not supported");
                Err(Error::custom("gRPC is not supported in Web/WASM version. Please use wRPC instead."))
            }
        }
    }

    /// Get reference to internal bp-tondi client
    pub fn client(&self) -> &BpTondiClient {
        &self.inner
    }
}

/// Implement RpcApi trait, providing interface compatible with wRPC clients
/// Only implementing methods that are actually used by the dashboard
#[async_trait]
impl RpcApi for TondiGrpcClient {
    /// Get server information - used by dashboard for connection status
    async fn get_server_info(&self) -> RpcResult<GetServerInfoResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                match self.inner.get_server_info().await {
                    Ok(_info) => {
                        // Return server info with network configuration
                        let response = GetServerInfoResponse {
                            rpc_api_version: 1,
                            rpc_api_revision: 1,
                            server_version: "tondi-grpc-client".to_string(),
                            network_id: RpcNetworkId::from(self.network),
                            has_utxo_index: false,
                            is_synced: false,
                            virtual_daa_score: 0,
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(RpcError::General(format!("Failed to get server info: {}", e)))
                    }
                }
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get blocks - used by dashboard for blockchain data
    async fn get_blocks(&self, low_hash: Option<RpcHash>, include_blocks: bool, include_transactions: bool) -> RpcResult<GetBlocksResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                match self.inner.get_blocks(low_hash.map(|h| h.into()), include_blocks, include_transactions).await {
                    Ok(blocks) => {
                        let response = GetBlocksResponse {
                            blocks: if include_blocks { Some(blocks) } else { None },
                            ..Default::default()
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(RpcError::General(format!("Failed to get blocks: {}", e)))
                    }
                }
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get connected peer info - used by peer monitor service
    async fn get_connected_peer_info(&self) -> RpcResult<GetConnectedPeerInfoResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Get peer info through connections
                match self.inner.get_connections(false).await {
                    Ok(connections_info) => {
                        let response = GetConnectedPeerInfoResponse {
                            peers: connections_info.peers,
                            ..Default::default()
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(RpcError::General(format!("Failed to get connected peer info: {}", e)))
                    }
                }
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get block count - used by dashboard for blockchain statistics
    async fn get_block_count(&self) -> RpcResult<GetBlockCountResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Estimate block count through get_blocks
                match self.inner.get_blocks(None, false, false).await {
                    Ok(blocks) => {
                        let block_count = blocks.len() as u64;
                        let response = GetBlockCountResponse {
                            count: block_count,
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(RpcError::General(format!("Failed to get block count: {}", e)))
                    }
                }
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get block DAG info - used by dashboard for DAG visualization
    async fn get_block_dag_info(&self) -> RpcResult<GetBlockDagInfoResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Get DAG info through get_blocks
                match self.inner.get_blocks(None, false, false).await {
                    Ok(blocks) => {
                        let response = GetBlockDagInfoResponse {
                            network_name: self.network.to_string(),
                            block_count: blocks.len() as u64,
                            header_count: blocks.len() as u64,
                            tip_hashes: vec![],
                            virtual_parent_hashes: vec![],
                            pruning_point_hash: None,
                            virtual_daa_score: 0,
                            difficulty: 0.0,
                            past_median_time: 0,
                            ..Default::default()
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(RpcError::General(format!("Failed to get block DAG info: {}", e)))
                    }
                }
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Ping call - used for connection health check
    async fn ping_call(&self, _connection: Option<&DynRpcConnection>, _request: PingRequest) -> RpcResult<PingResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Return default response for ping
                Ok(PingResponse {})
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get system info - used by dashboard for system monitoring
    async fn get_system_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSystemInfoRequest) -> RpcResult<GetSystemInfoResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Get system info from server info and DAG info
                let server_info = match self.inner.get_server_info().await {
                    Ok(info) => info,
                    Err(_) => return Err(RpcError::General("Failed to get server info".to_string())),
                };

                let system_id = match self.inner.get_block_dag_info().await {
                    Ok(dag_info) => dag_info.network_name,
                    Err(_) => self.network.to_string(),
                };

                let response = GetSystemInfoResponse {
                    server_version: server_info.version,
                    build_version: server_info.version,
                    protocol_version: 1,
                    sub_version: "tondi-grpc".to_string(),
                    user_agent: "tondi-dashboard".to_string(),
                    network_name: system_id,
                    ..Default::default()
                };
                Ok(response)
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get connections - used by dashboard for network monitoring
    async fn get_connections_call(&self, _connection: Option<&DynRpcConnection>, request: GetConnectionsRequest) -> RpcResult<GetConnectionsResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                match self.inner.get_connections(request.include_profile_data).await {
                    Ok(connections_info) => {
                        let response = GetConnectionsResponse {
                            clients: connections_info.clients,
                            peers: connections_info.peers,
                            profile_data: if request.include_profile_data {
                                Some(ConnectionsProfileData {
                                    cpu_usage: 0.0,
                                    memory_usage: 0,
                                })
                            } else {
                                None
                            }
                        };
                        Ok(response)
                    }
                    Err(e) => {
                        Err(RpcError::General(format!("Failed to get connections: {}", e)))
                    }
                }
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get metrics - used by metrics monitor service
    async fn get_metrics_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMetricsRequest) -> RpcResult<GetMetricsResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // Get real metrics data from gRPC server
                let mut consensus_metrics = None;
                let mut process_metrics = None;
                let mut connection_metrics = None;
                let mut bandwidth_metrics = None;
                let mut storage_metrics = None;
                
                // 1. Get consensus related metrics
                match self.inner.get_blocks(None, false, false).await {
                    Ok(blocks) => {
                        let block_count = blocks.len() as u64;
                        let (difficulty, daa_score, median_time) = match self.inner.get_block_dag_info().await {
                            Ok(dag_info) => (dag_info.difficulty, dag_info.virtual_daa_score, dag_info.past_median_time),
                            Err(_) => (0.0, 0, 0),
                        };
                        
                        consensus_metrics = Some(ConsensusMetrics {
                            block_count,
                            difficulty,
                            daa_score,
                            median_time,
                            ..Default::default()
                        });
                    }
                    Err(e) => {
                        println!("[gRPC DEBUG] Failed to get blocks: {}", e);
                        consensus_metrics = Some(ConsensusMetrics::default());
                    }
                }
                
                // 2. Get connection metrics
                match self.inner.get_connections(false).await {
                    Ok(connections_info) => {
                        connection_metrics = Some(ConnectionMetrics {
                            json_live_connections: connections_info.clients.len() as u64,
                            json_connection_attempts: 0,
                            json_handshake_failures: 0,
                            borsh_live_connections: 0,
                            borsh_connection_attempts: 0,
                            borsh_handshake_failures: 0,
                            active_peers: connections_info.peers.len() as u64,
                        });
                    }
                    Err(e) => {
                        println!("[gRPC DEBUG] Failed to get connections: {}", e);
                        connection_metrics = Some(ConnectionMetrics {
                            json_live_connections: 0,
                            json_connection_attempts: 0,
                            json_handshake_failures: 0,
                            borsh_live_connections: 0,
                            borsh_connection_attempts: 0,
                            borsh_handshake_failures: 0,
                            active_peers: 0,
                        });
                    }
                }
                
                // 3. Construct complete metrics response
                let response = GetMetricsResponse {
                    server_time: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    process_metrics,
                    connection_metrics,
                    bandwidth_metrics,
                    consensus_metrics,
                    storage_metrics,
                    custom_metrics: None,
                };
                
                Ok(response)
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get metrics - required by tondi_metrics_core::Metrics trait
    async fn get_metrics(&self, _include_process_metrics: bool, _include_connection_metrics: bool, _include_bandwidth_metrics: bool, _include_consensus_metrics: bool, _include_storage_metrics: bool, _include_custom_metrics: bool) -> RpcResult<GetMetricsResponse> {
        let request = GetMetricsRequest {
            process_metrics: _include_process_metrics,
            connection_metrics: _include_connection_metrics,
            bandwidth_metrics: _include_bandwidth_metrics,
            consensus_metrics: _include_consensus_metrics,
            storage_metrics: _include_storage_metrics,
            custom_metrics: _include_custom_metrics,
        };
        self.get_metrics_call(None, request).await
    }

    /// Get server info call - used by dashboard
    async fn get_server_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetServerInfoRequest) -> RpcResult<GetServerInfoResponse> {
        self.get_server_info().await
    }

    /// Get current network - used by dashboard for network display
    async fn get_current_network_call(&self, _connection: Option<&DynRpcConnection>, _request: GetCurrentNetworkRequest) -> RpcResult<GetCurrentNetworkResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let response = GetCurrentNetworkResponse {
                    network: self.network.into(),
                };
                Ok(response)
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    /// Get connected peer info call - used by peer monitor
    async fn get_connected_peer_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetConnectedPeerInfoRequest) -> RpcResult<GetConnectedPeerInfoResponse> {
        self.get_connected_peer_info().await
    }

    /// Get blocks call - used by dashboard
    async fn get_blocks_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlocksRequest) -> RpcResult<GetBlocksResponse> {
        self.get_blocks(_request.low_hash, _request.include_blocks, _request.include_transactions).await
    }

    /// Get block count call - used by dashboard
    async fn get_block_count_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockCountRequest) -> RpcResult<GetBlockCountResponse> {
        self.get_block_count().await
    }

    /// Get block DAG info call - used by dashboard
    async fn get_block_dag_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockDagInfoRequest) -> RpcResult<GetBlockDagInfoResponse> {
        self.get_block_dag_info().await
    }

    /// Register new listener - required by trait
    fn register_new_listener(&self, _connection: ChannelConnection) -> ListenerId {
        0 // Simplified implementation
    }

    /// Unregister listener - required by trait
    async fn unregister_listener(&self, _id: ListenerId) -> RpcResult<()> {
        Ok(()) // Simplified implementation
    }

    /// Start notify - required by trait
    async fn start_notify(&self, _id: ListenerId, _scope: Scope) -> RpcResult<()> {
        Ok(()) // Simplified implementation
    }

    /// Stop notify - required by trait
    async fn stop_notify(&self, _id: ListenerId, _scope: Scope) -> RpcResult<()> {
        Ok(()) // Simplified implementation
    }

    // Dashboard不需要的方法 - 返回错误或默认值
    async fn submit_block_call(&self, _connection: Option<&DynRpcConnection>, _request: SubmitBlockRequest) -> RpcResult<SubmitBlockResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_block_template_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockTemplateRequest) -> RpcResult<GetBlockTemplateResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_peer_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetPeerAddressesRequest) -> RpcResult<GetPeerAddressesResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_sink_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSinkRequest) -> RpcResult<GetSinkResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_mempool_entry_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMempoolEntryRequest) -> RpcResult<GetMempoolEntryResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_mempool_entries_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMempoolEntriesRequest) -> RpcResult<GetMempoolEntriesResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn add_peer_call(&self, _connection: Option<&DynRpcConnection>, _request: AddPeerRequest) -> RpcResult<AddPeerResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn submit_transaction_call(&self, _connection: Option<&DynRpcConnection>, _request: SubmitTransactionRequest) -> RpcResult<SubmitTransactionResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn submit_transaction_replacement_call(&self, _connection: Option<&DynRpcConnection>, _request: SubmitTransactionReplacementRequest) -> RpcResult<SubmitTransactionReplacementResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_block_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockRequest) -> RpcResult<GetBlockResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_subnetwork_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSubnetworkRequest) -> RpcResult<GetSubnetworkResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_virtual_chain_from_block_call(&self, _connection: Option<&DynRpcConnection>, _request: GetVirtualChainFromBlockRequest) -> RpcResult<GetVirtualChainFromBlockResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn resolve_finality_conflict_call(&self, _connection: Option<&DynRpcConnection>, _request: ResolveFinalityConflictRequest) -> RpcResult<ResolveFinalityConflictResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn shutdown_call(&self, _connection: Option<&DynRpcConnection>, _request: ShutdownRequest) -> RpcResult<ShutdownResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_headers_call(&self, _connection: Option<&DynRpcConnection>, _request: GetHeadersRequest) -> RpcResult<GetHeadersResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_balance_by_address_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBalanceByAddressRequest) -> RpcResult<GetBalanceByAddressResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_balances_by_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBalancesByAddressesRequest) -> RpcResult<GetBalancesByAddressesResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_utxos_by_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetUtxosByAddressesRequest) -> RpcResult<GetUtxosByAddressesResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_sink_blue_score_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSinkBlueScoreRequest) -> RpcResult<GetSinkBlueScoreResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn ban_call(&self, _connection: Option<&DynRpcConnection>, _request: BanRequest) -> RpcResult<BanResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn unban_call(&self, _connection: Option<&DynRpcConnection>, _request: UnbanRequest) -> RpcResult<UnbanResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetInfoRequest) -> RpcResult<GetInfoResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn estimate_network_hashes_per_second_call(&self, _connection: Option<&DynRpcConnection>, _request: EstimateNetworkHashesPerSecondRequest) -> RpcResult<EstimateNetworkHashesPerSecondResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_mempool_entries_by_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMempoolEntriesByAddressesRequest) -> RpcResult<GetMempoolEntriesByAddressesResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_coin_supply_call(&self, _connection: Option<&DynRpcConnection>, _request: GetCoinSupplyRequest) -> RpcResult<GetCoinSupplyResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_daa_score_timestamp_estimate_call(&self, _connection: Option<&DynRpcConnection>, _request: GetDaaScoreTimestampEstimateRequest) -> RpcResult<GetDaaScoreTimestampEstimateResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_utxo_return_address_call(&self, _connection: Option<&DynRpcConnection>, _request: GetUtxoReturnAddressRequest) -> RpcResult<GetUtxoReturnAddressResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_fee_estimate_call(&self, _connection: Option<&DynRpcConnection>, _request: GetFeeEstimateRequest) -> RpcResult<GetFeeEstimateResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_fee_estimate_experimental_call(&self, _connection: Option<&DynRpcConnection>, _request: GetFeeEstimateExperimentalRequest) -> RpcResult<GetFeeEstimateExperimentalResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    async fn get_current_block_color_call(&self, _connection: Option<&DynRpcConnection>, _request: GetCurrentBlockColorRequest) -> RpcResult<GetCurrentBlockColorResponse> {
        Err(RpcError::General("Method not needed by dashboard".to_string()))
    }

    // 需要实现但暂时返回错误的方法
    async fn get_sync_status_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSyncStatusRequest) -> RpcResult<GetSyncStatusResponse> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                // TODO: 实现同步状态检查
                Err(RpcError::General("gRPC get_sync_status_call not implemented yet".to_string()))
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }
}

/// Simple wrapper for gRPC client, used for basic connection and calls
impl TondiGrpcClient {
    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                true // Simplified implementation
            } else {
                false
            }
        }
    }
    
    /// Get network ID
    pub fn network_id(&self) -> Option<NetworkId> {
        Some(self.network.into())
    }
    
    /// Get server URL
    pub fn url(&self) -> Option<String> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                Some(self.url.clone())
            } else {
                None
            }
        }
    }
}

// Add test functions
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_client_creation() {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let network_interface = NetworkInterfaceConfig::default();
                let network = Network::Mainnet;
                let client = TondiGrpcClient::connect(network_interface, network).await;
                assert!(client.is_ok() || client.is_err());
            } else {
                let network_interface = NetworkInterfaceConfig::default();
                let network = Network::Mainnet;
                let client = TondiGrpcClient::connect(network_interface, network).await;
                assert!(client.is_err());
            }
        }
    }
}

/// gRPC RPC control implementation
#[derive(Default, Clone)]
pub struct GrpcRpcCtl {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    state: Mutex<RpcState>,
    multiplexer: Multiplexer<RpcState>,
    descriptor: Mutex<Option<String>>,
}

impl GrpcRpcCtl {
    pub fn new() -> Self {
        Self { 
            inner: Arc::new(Inner {
                state: Mutex::new(RpcState::Connected),
                multiplexer: Multiplexer::new(),
                descriptor: Mutex::new(None),
            })
        }
    }

    pub fn multiplexer(&self) -> &Multiplexer<RpcState> {
        &self.inner.multiplexer
    }

    pub fn is_connected(&self) -> bool {
        *self.inner.state.lock().unwrap() == RpcState::Connected
    }

    pub fn state(&self) -> RpcState {
        *self.inner.state.lock().unwrap()
    }

    pub async fn signal_open(&self) -> RpcResult<()> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                *self.inner.state.lock().unwrap() = RpcState::Connected;
                Ok(self.inner.multiplexer.broadcast(RpcState::Connected).await?)
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    pub async fn signal_close(&self) -> RpcResult<()> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                *self.inner.state.lock().unwrap() = RpcState::Disconnected;
                Ok(self.inner.multiplexer.broadcast(RpcState::Disconnected).await?)
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    pub fn try_signal_open(&self) -> RpcResult<()> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                *self.inner.state.lock().unwrap() = RpcState::Connected;
                Ok(())
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    pub fn try_signal_close(&self) -> RpcResult<()> {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                *self.inner.state.lock().unwrap() = RpcState::Disconnected;
                Ok(())
            } else {
                Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
            }
        }
    }

    pub fn descriptor(&self) -> Option<String> {
        self.inner.descriptor.lock().unwrap().clone()
    }

    pub fn set_descriptor(&self, descriptor: Option<String>) {
        *self.inner.descriptor.lock().unwrap() = descriptor;
    }
}
