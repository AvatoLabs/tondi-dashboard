use crate::imports::*;
use crate::network::Network;
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};

use tondi_rpc_core::{
    GetMetricsResponse,
    GetServerInfoResponse, GetConnectedPeerInfoResponse, GetBlockCountResponse, GetBlockDagInfoResponse,
    RpcResult,
};
use tondi_rpc_core::api::rpc::RpcApi;
use tondi_consensus_core::api::BlockCount;


// 使用TONDI项目的真实gRPC客户端
use tondi_grpc_client::GrpcClient;

/// Tondi gRPC客户端，使用TONDI项目的真实gRPC客户端
#[derive(Clone)]
pub struct TondiGrpcClient {
    grpc_client: Arc<Mutex<Option<GrpcClient>>>,
    url: String,
    network: Network,
    is_connected: Arc<AtomicBool>,
}

impl TondiGrpcClient {
    pub async fn connect(network_interface: NetworkInterfaceConfig, network: Network) -> Result<Self> {

        let url = format!("grpc://{}", network_interface);
        
        // 使用TONDI项目的真实gRPC客户端
        match GrpcClient::connect(url.clone()).await {
            Ok(grpc_client) => {
                Ok(Self {
                    grpc_client: Arc::new(Mutex::new(Some(grpc_client))),
                    url: url.clone(),
                    network,
                    is_connected: Arc::new(AtomicBool::new(true)),
                })
            }
            Err(_e) => {
                // 返回一个未连接的客户端，后续可以重试
                Ok(Self {
                    grpc_client: Arc::new(Mutex::new(None)),
                    url: url.clone(),
                    network,
                    is_connected: Arc::new(AtomicBool::new(false)),
                })
            }
        }
    }

    /// 检查连接状态，如果未连接则尝试重新连接
    async fn ensure_connected(&self) -> Result<()> {
        if self.is_connected.load(Ordering::Acquire) {
            return Ok(());
        }

        match GrpcClient::connect(self.url.clone()).await {
            Ok(grpc_client) => {
                // 更新连接状态和客户端
                {
                    let mut client_guard = self.grpc_client.lock().unwrap();
                    *client_guard = Some(grpc_client);
                }
                self.is_connected.store(true, Ordering::Release);
                println!("[TONDI GRPC] 重新连接成功");
                Ok(())
            }
            Err(e) => {
                println!("[TONDI GRPC] 重新连接失败: {}", e);
                Err(Error::custom(format!("Failed to reconnect: {}", e)))
            }
        }
    }

    /// 获取内部客户端引用
    pub fn client(&self) -> &Self {
        self
    }

    /// 检查客户端是否连接
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::Acquire)
    }
    
    /// 获取网络ID（如果可用）
    pub fn network_id(&self) -> Option<tondi_consensus_core::network::NetworkId> {
        Some(tondi_consensus_core::network::NetworkId::from(self.network))
    }
    
    /// 获取服务器URL
    pub fn url(&self) -> Option<String> {
        Some(self.url.clone())
    }
}

/// 实现RpcApi trait，提供与wRPC客户端兼容的接口
#[async_trait]
impl RpcApi for TondiGrpcClient {
    async fn get_server_info(&self) -> RpcResult<GetServerInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get server info");
            
            // Call the real gRPC client
            match grpc_client.get_server_info_call(None, tondi_rpc_core::GetServerInfoRequest {}).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got server info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get server info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get server info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_connected_peer_info(&self) -> RpcResult<GetConnectedPeerInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get connected peer info");
            
            // Call the real gRPC client
            match grpc_client.get_connected_peer_info_call(None, tondi_rpc_core::GetConnectedPeerInfoRequest {}).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got connected peer info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get connected peer info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get connected peer info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_count(&self) -> RpcResult<GetBlockCountResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get block count");
            
            // Create GetBlockCountRequest
            let request = tondi_rpc_core::GetBlockCountRequest {};
            
            // Call the real gRPC client
            match grpc_client.get_block_count_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got block count from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get block count from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block count from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_dag_info(&self) -> RpcResult<GetBlockDagInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get block DAG info");
            
            // Create GetBlockDagInfoRequest
            let request = tondi_rpc_core::GetBlockDagInfoRequest {};
            
            // Call the real gRPC client
            match grpc_client.get_block_dag_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got block DAG info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get block DAG info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block DAG info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_metrics(&self, _include_process_metrics: bool, _include_connection_metrics: bool, _include_bandwidth_metrics: bool, _include_consensus_metrics: bool, _include_storage_metrics: bool, _include_custom_metrics: bool) -> RpcResult<GetMetricsResponse> {
        println!("[TONDI GRPC] get_metrics called with parameters: process={}, connection={}, bandwidth={}, consensus={}, storage={}, custom={}", 
            _include_process_metrics, _include_connection_metrics, _include_bandwidth_metrics, _include_consensus_metrics, _include_storage_metrics, _include_custom_metrics);
        println!("[TONDI GRPC] Current connection status: is_connected={}", self.is_connected());
        println!("[TONDI GRPC] Current URL: {}", self.url);

        if !self.is_connected() {
            println!("[TONDI GRPC] Attempting to reconnect...");
            if let Err(e) = self.ensure_connected().await {
                println!("[TONDI GRPC] Reconnection failed: {}", e);
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get metrics");
            
            // Create GetMetricsRequest
            let request = tondi_rpc_core::GetMetricsRequest {
                process_metrics: _include_process_metrics,
                connection_metrics: _include_connection_metrics,
                bandwidth_metrics: _include_bandwidth_metrics,
                consensus_metrics: _include_consensus_metrics,
                storage_metrics: _include_storage_metrics,
                custom_metrics: _include_custom_metrics,
            };
            
            // Call the real gRPC client
            match grpc_client.get_metrics_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got metrics from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get metrics from remote node: {}", e);
                    // Return error instead of hardcoded 0 values if remote call fails
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get metrics from remote node: {}", e)))
                }
            }
        } else {
            println!("[TONDI GRPC] No gRPC client available");
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    // 实现其他必要的方法，返回默认值或错误
    async fn ping_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::PingRequest) -> RpcResult<tondi_rpc_core::PingResponse> {
        Ok(tondi_rpc_core::PingResponse {})
    }

    async fn get_system_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetSystemInfoRequest) -> RpcResult<tondi_rpc_core::GetSystemInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get system info");
            
            // Call the real gRPC client
            match grpc_client.get_system_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got system info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get system info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get system info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_connections_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetConnectionsRequest) -> RpcResult<tondi_rpc_core::GetConnectionsResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get connections");
            
            // Call the real gRPC client
            match grpc_client.get_connections_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got connections from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get connections from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get connections from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    // 其他方法返回默认值或错误
    async fn get_metrics_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetMetricsRequest) -> RpcResult<GetMetricsResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get metrics");
            
            // Call the real gRPC client
            match grpc_client.get_metrics_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got metrics from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get metrics from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get metrics from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_server_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetServerInfoRequest) -> RpcResult<GetServerInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get server info");
            
            // Call the real gRPC client
            match grpc_client.get_server_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got server info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get server info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get server info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_sync_status_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetSyncStatusRequest) -> RpcResult<tondi_rpc_core::GetSyncStatusResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get sync status");
            
            // Call the real gRPC client
            match grpc_client.get_sync_status_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got sync status from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get sync status from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get sync status from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_current_network_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetCurrentNetworkRequest) -> RpcResult<tondi_rpc_core::GetCurrentNetworkResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get current network");
            
            // Call the real gRPC client
            match grpc_client.get_current_network_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got current network from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get current network from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get current network from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn submit_block_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::SubmitBlockRequest) -> RpcResult<tondi_rpc_core::SubmitBlockResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to submit block");
            
            // Call the real gRPC client
            match grpc_client.submit_block_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully submitted block to remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to submit block to remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to submit block to remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_template_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBlockTemplateRequest) -> RpcResult<tondi_rpc_core::GetBlockTemplateResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get block template");
            
            // Call the real gRPC client
            match grpc_client.get_block_template_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got block template from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get block template from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block template from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_peer_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetPeerAddressesRequest) -> RpcResult<tondi_rpc_core::GetPeerAddressesResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get peer addresses");
            
            // Call the real gRPC client
            match grpc_client.get_peer_addresses_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got peer addresses from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get peer addresses from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get peer addresses from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_sink_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetSinkRequest) -> RpcResult<tondi_rpc_core::GetSinkResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get sink");
            
            // Call the real gRPC client
            match grpc_client.get_sink_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got sink from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get sink from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get sink from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_mempool_entry_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetMempoolEntryRequest) -> RpcResult<tondi_rpc_core::GetMempoolEntryResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get mempool entry");
            
            // Call the real gRPC client
            match grpc_client.get_mempool_entry_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got mempool entry from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get mempool entry from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get mempool entry from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_mempool_entries_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetMempoolEntriesRequest) -> RpcResult<tondi_rpc_core::GetMempoolEntriesResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get mempool entries");
            
            // Call the real gRPC client
            match grpc_client.get_mempool_entries_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got mempool entries from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get mempool entries from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get mempool entries from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_connected_peer_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetConnectedPeerInfoRequest) -> RpcResult<GetConnectedPeerInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get connected peer info");
            
            // Call the real gRPC client
            match grpc_client.get_connected_peer_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got connected peer info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get connected peer info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get connected peer info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn add_peer_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::AddPeerRequest) -> RpcResult<tondi_rpc_core::AddPeerResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to add peer");
            
            // Call the real gRPC client
            match grpc_client.add_peer_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully added peer to remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to add peer to remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to add peer to remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn submit_transaction_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::SubmitTransactionRequest) -> RpcResult<tondi_rpc_core::SubmitTransactionResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to submit transaction");
            
            // Call the real gRPC client
            match grpc_client.submit_transaction_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully submitted transaction to remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to submit transaction to remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to submit transaction to remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn submit_transaction_replacement_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::SubmitTransactionReplacementRequest) -> RpcResult<tondi_rpc_core::SubmitTransactionReplacementResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to submit transaction replacement");
            
            // Call the real gRPC client
            match grpc_client.submit_transaction_replacement_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully submitted transaction replacement to remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to submit transaction replacement to remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to submit transaction replacement to remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBlockRequest) -> RpcResult<tondi_rpc_core::GetBlockResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get block");
            
            // Call the real gRPC client
            match grpc_client.get_block_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got block from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get block from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_subnetwork_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetSubnetworkRequest) -> RpcResult<tondi_rpc_core::GetSubnetworkResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get subnetwork");
            
            // Call the real gRPC client
            match grpc_client.get_subnetwork_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got subnetwork from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get subnetwork from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get subnetwork from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_virtual_chain_from_block_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetVirtualChainFromBlockRequest) -> RpcResult<tondi_rpc_core::GetVirtualChainFromBlockResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get virtual chain from block");
            
            // Call the real gRPC client
            match grpc_client.get_virtual_chain_from_block_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got virtual chain from block from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get virtual chain from block from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get virtual chain from block from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_blocks_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBlocksRequest) -> RpcResult<tondi_rpc_core::GetBlocksResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get blocks");
            
            // Call the real gRPC client
            match grpc_client.get_blocks_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got blocks from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get blocks from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get blocks from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_count_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBlockCountRequest) -> RpcResult<BlockCount> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get block count");
            
            // Call the real gRPC client
            match grpc_client.get_block_count_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got block count from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get block count from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block count from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_dag_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBlockDagInfoRequest) -> RpcResult<GetBlockDagInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get block DAG info");
            
            // Call the real gRPC client
            match grpc_client.get_block_dag_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got block DAG info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get block DAG info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block DAG info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn resolve_finality_conflict_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::ResolveFinalityConflictRequest) -> RpcResult<tondi_rpc_core::ResolveFinalityConflictResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to resolve finality conflict");
            
            // Call the real gRPC client
            match grpc_client.resolve_finality_conflict_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully resolved finality conflict on remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to resolve finality conflict on remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to resolve finality conflict on remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn shutdown_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::ShutdownRequest) -> RpcResult<tondi_rpc_core::ShutdownResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to shutdown");
            
            // Call the real gRPC client
            match grpc_client.shutdown_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully initiated shutdown on remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to initiate shutdown on remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to initiate shutdown on remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_headers_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetHeadersRequest) -> RpcResult<tondi_rpc_core::GetHeadersResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get headers");
            
            // Call the real gRPC client
            match grpc_client.get_headers_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got headers from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get headers from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get headers from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_balance_by_address_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBalanceByAddressRequest) -> RpcResult<tondi_rpc_core::GetBalanceByAddressResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get balance by address");
            
            // Call the real gRPC client
            match grpc_client.get_balance_by_address_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got balance from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get balance from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get balance from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_balances_by_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetBalancesByAddressesRequest) -> RpcResult<tondi_rpc_core::GetBalancesByAddressesResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get balances by addresses");
            
            // Call the real gRPC client
            match grpc_client.get_balances_by_addresses_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got balances from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get balances from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get balances from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_utxos_by_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetUtxosByAddressesRequest) -> RpcResult<tondi_rpc_core::GetUtxosByAddressesResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get UTXOs by addresses");
            
            // Call the real gRPC client
            match grpc_client.get_utxos_by_addresses_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got UTXOs from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get UTXOs from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get UTXOs from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_sink_blue_score_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetSinkBlueScoreRequest) -> RpcResult<tondi_rpc_core::GetSinkBlueScoreResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get sink blue score");
            
            // Call the real gRPC client
            match grpc_client.get_sink_blue_score_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got sink blue score from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get sink blue score from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get sink blue score from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn ban_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::BanRequest) -> RpcResult<tondi_rpc_core::BanResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to ban peer");
            
            // Call the real gRPC client
            match grpc_client.ban_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully banned peer on remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to ban peer on remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to ban peer on remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn unban_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::UnbanRequest) -> RpcResult<tondi_rpc_core::UnbanResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to unban peer");
            
            // Call the real gRPC client
            match grpc_client.unban_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully unbanned peer on remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to unban peer on remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to unban peer on remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetInfoRequest) -> RpcResult<tondi_rpc_core::GetInfoResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get info");
            
            // Call the real gRPC client
            match grpc_client.get_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got info from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get info from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn estimate_network_hashes_per_second_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::EstimateNetworkHashesPerSecondRequest) -> RpcResult<tondi_rpc_core::EstimateNetworkHashesPerSecondResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to estimate network hashes per second");
            
            // Call the real gRPC client
            match grpc_client.estimate_network_hashes_per_second_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got network hashes per second estimate from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get network hashes per second estimate from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get network hashes per second estimate from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_mempool_entries_by_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetMempoolEntriesByAddressesRequest) -> RpcResult<tondi_rpc_core::GetMempoolEntriesByAddressesResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get mempool entries by addresses");
            
            // Call the real gRPC client
            match grpc_client.get_mempool_entries_by_addresses_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got mempool entries from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get mempool entries from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get mempool entries from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_coin_supply_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetCoinSupplyRequest) -> RpcResult<tondi_rpc_core::GetCoinSupplyResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get coin supply");
            
            // Call the real gRPC client
            match grpc_client.get_coin_supply_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got coin supply from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get coin supply from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get coin supply from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_daa_score_timestamp_estimate_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetDaaScoreTimestampEstimateRequest) -> RpcResult<tondi_rpc_core::GetDaaScoreTimestampEstimateResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get DAA score timestamp estimate");
            
            // Call the real gRPC client
            match grpc_client.get_daa_score_timestamp_estimate_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got DAA score timestamp estimate from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get DAA score timestamp estimate from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get DAA score timestamp estimate from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_utxo_return_address_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetUtxoReturnAddressRequest) -> RpcResult<tondi_rpc_core::GetUtxoReturnAddressResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get UTXO return address");
            
            // Call the real gRPC client
            match grpc_client.get_utxo_return_address_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got UTXO return address from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get UTXO return address from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get UTXO return address from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_fee_estimate_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetFeeEstimateRequest) -> RpcResult<tondi_rpc_core::GetFeeEstimateResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get fee estimate");
            
            // Call the real gRPC client
            match grpc_client.get_fee_estimate_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got fee estimate from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get fee estimate from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get fee estimate from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_fee_estimate_experimental_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetFeeEstimateExperimentalRequest) -> RpcResult<tondi_rpc_core::GetFeeEstimateExperimentalResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get experimental fee estimate");
            
            // Call the real gRPC client
            match grpc_client.get_fee_estimate_experimental_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got experimental fee estimate from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get experimental fee estimate from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get experimental fee estimate from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_current_block_color_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetCurrentBlockColorRequest) -> RpcResult<tondi_rpc_core::GetCurrentBlockColorResponse> {
        // Ensure connection before making the call
        if !self.is_connected() {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        let grpc_client = {
            let client_guard = self.grpc_client.lock().unwrap();
            client_guard.clone()
        };
        
        if let Some(grpc_client) = grpc_client {
            println!("[TONDI GRPC] Using real gRPC client to get current block color");
            
            // Call the real gRPC client
            match grpc_client.get_current_block_color_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] Successfully got current block color from remote node: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] Failed to get current block color from remote node: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get current block color from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    fn register_new_listener(&self, _connection: tondi_rpc_core::notify::connection::ChannelConnection) -> u64 {
        0
    }

    async fn unregister_listener(&self, _id: u64) -> RpcResult<()> {
        Ok(())
    }

    async fn start_notify(&self, _id: u64, _scope: tondi_notify::scope::Scope) -> RpcResult<()> {
        Ok(())
    }

    async fn stop_notify(&self, _id: u64, _scope: tondi_notify::scope::Scope) -> RpcResult<()> {
        Ok(())
    }
}
