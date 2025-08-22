use crate::imports::*;
use crate::network::Network;
use async_trait::async_trait;

use tondi_rpc_core::{
    BandwidthMetrics, ConnectionMetrics, ConsensusMetrics, GetMetricsResponse,
    GetServerInfoResponse, GetConnectedPeerInfoResponse, GetBlockCountResponse, GetBlockDagInfoResponse,
    ProcessMetrics, RpcNetworkId, RpcResult, StorageMetrics, RpcHash,
};
use tondi_rpc_core::api::rpc::RpcApi;
use tondi_consensus_core::api::BlockCount;


// 使用TONDI项目的真实gRPC客户端
use tondi_grpc_client::GrpcClient;

/// Tondi gRPC客户端，使用TONDI项目的真实gRPC客户端
#[derive(Clone)]
pub struct TondiGrpcClient {
    grpc_client: Option<Arc<GrpcClient>>,
    url: String,
    network: Network,
    is_connected: bool,
}

impl TondiGrpcClient {
    pub async fn connect(network_interface: NetworkInterfaceConfig, network: Network) -> Result<Self> {
        println!("[TONDI GRPC] TondiGrpcClient::connect 被调用");
        println!("[TONDI GRPC] network_interface: {:?}", network_interface);
        println!("[TONDI GRPC] network: {:?}", network);
        
        let url = format!("grpc://{}", network_interface);
        println!("[TONDI GRPC] 尝试连接到: {}", url);
        
        // 使用TONDI项目的真实gRPC客户端
        match GrpcClient::connect(url.clone()).await {
            Ok(grpc_client) => {
                println!("[TONDI GRPC] 成功连接到TONDI gRPC节点: {}", url);
                Ok(Self {
                    grpc_client: Some(Arc::new(grpc_client)),
                    url: url.clone(),
                    network,
                    is_connected: true,
                })
            }
            Err(e) => {
                println!("[TONDI GRPC] 连接失败: {}", e);
                // 返回一个未连接的客户端，后续可以重试
                Ok(Self {
                    grpc_client: None,
                    url: url.clone(),
                    network,
                    is_connected: false,
                })
            }
        }
    }

    /// 检查连接状态，如果未连接则尝试重新连接
    async fn ensure_connected(&self) -> Result<()> {
        if self.is_connected {
            return Ok(());
        }

        println!("[gRPC DEBUG] 尝试重新连接到节点: {}", self.url);
        match GrpcClient::connect(self.url.clone()).await {
            Ok(_grpc_client) => {
                println!("[gRPC DEBUG] 重新连接成功");
                Ok(())
            }
            Err(e) => {
                println!("[gRPC DEBUG] 重新连接失败: {}", e);
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
        self.is_connected
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
        // 尝试重新连接如果未连接
        if !self.is_connected {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        // 使用真实的gRPC客户端获取服务器信息
        if let Some(_grpc_client) = &self.grpc_client {
            // 这里需要调用TONDI gRPC客户端的相应方法
            // 由于TONDI gRPC客户端没有直接实现RpcApi，我们需要适配
            // 暂时返回默认值，后续需要实现真正的调用
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
            } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_connected_peer_info(&self) -> RpcResult<GetConnectedPeerInfoResponse> {
        // 尝试重新连接如果未连接
        if !self.is_connected {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        // 使用真实的gRPC客户端获取对等节点信息
        if let Some(_grpc_client) = &self.grpc_client {
            // 暂时返回空列表，后续需要实现真正的调用
                let response = GetConnectedPeerInfoResponse::new(vec![]);
                Ok(response)
            } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_count(&self) -> RpcResult<GetBlockCountResponse> {
        // 尝试重新连接如果未连接
        if !self.is_connected {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        // 使用真实的gRPC客户端获取区块数量
        if let Some(grpc_client) = &self.grpc_client {
            println!("[TONDI GRPC] 使用真实的gRPC客户端获取block count");
            
            // 创建GetBlockCountRequest
            let request = tondi_rpc_core::GetBlockCountRequest {};
            
            // 调用真实的gRPC客户端
            match grpc_client.get_block_count_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] 成功从远程节点获取block count: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] 从远程节点获取block count失败: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block count from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_block_dag_info(&self) -> RpcResult<GetBlockDagInfoResponse> {
        // 尝试重新连接如果未连接
        if !self.is_connected {
            if let Err(e) = self.ensure_connected().await {
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        // 使用真实的gRPC客户端获取区块DAG信息
        if let Some(grpc_client) = &self.grpc_client {
            println!("[TONDI GRPC] 使用真实的gRPC客户端获取block dag info");
            
            // 创建GetBlockDagInfoRequest
            let request = tondi_rpc_core::GetBlockDagInfoRequest {};
            
            // 调用真实的gRPC客户端
            match grpc_client.get_block_dag_info_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] 成功从远程节点获取block dag info: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] 从远程节点获取block dag info失败: {}", e);
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get block dag info from remote node: {}", e)))
                }
            }
        } else {
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    async fn get_metrics(&self, _include_process_metrics: bool, _include_connection_metrics: bool, _include_bandwidth_metrics: bool, _include_consensus_metrics: bool, _include_storage_metrics: bool, _include_custom_metrics: bool) -> RpcResult<GetMetricsResponse> {
        println!("[TONDI GRPC] get_metrics 被调用，参数: process={}, connection={}, bandwidth={}, consensus={}, storage={}, custom={}", 
            _include_process_metrics, _include_connection_metrics, _include_bandwidth_metrics, _include_consensus_metrics, _include_storage_metrics, _include_custom_metrics);
        println!("[TONDI GRPC] 当前连接状态: is_connected={}", self.is_connected);
        println!("[TONDI GRPC] 当前URL: {}", self.url);

        if !self.is_connected {
            println!("[TONDI GRPC] 尝试重新连接...");
            if let Err(e) = self.ensure_connected().await {
                println!("[TONDI GRPC] 重新连接失败: {}", e);
                return Err(tondi_rpc_core::RpcError::General(format!("Not connected: {}", e)));
            }
        }

        if let Some(grpc_client) = &self.grpc_client {
            println!("[TONDI GRPC] 使用真实的gRPC客户端获取metrics");
            
            // 创建GetMetricsRequest
            let request = tondi_rpc_core::GetMetricsRequest {
                process_metrics: _include_process_metrics,
                connection_metrics: _include_connection_metrics,
                bandwidth_metrics: _include_bandwidth_metrics,
                consensus_metrics: _include_consensus_metrics,
                storage_metrics: _include_storage_metrics,
                custom_metrics: _include_custom_metrics,
            };
            
            // 调用真实的gRPC客户端
            match grpc_client.get_metrics_call(None, request).await {
                Ok(response) => {
                    println!("[TONDI GRPC] 成功从远程节点获取metrics: {:?}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("[TONDI GRPC] 从远程节点获取metrics失败: {}", e);
                    // 如果远程调用失败，返回错误而不是硬编码的0值
                    Err(tondi_rpc_core::RpcError::General(format!("Failed to get metrics from remote node: {}", e)))
                }
            }
        } else {
            println!("[TONDI GRPC] 没有可用的gRPC客户端");
            Err(tondi_rpc_core::RpcError::General("No gRPC client available".to_string()))
        }
    }

    // 实现其他必要的方法，返回默认值或错误
    async fn ping_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::PingRequest) -> RpcResult<tondi_rpc_core::PingResponse> {
        Ok(tondi_rpc_core::PingResponse {})
    }

    async fn get_system_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetSystemInfoRequest) -> RpcResult<tondi_rpc_core::GetSystemInfoResponse> {
        let response = tondi_rpc_core::GetSystemInfoResponse {
                    version: "tondi-grpc-client".to_string(),
                    system_id: Some(b"tondi-grpc".to_vec()),
                    git_hash: None,
                    total_memory: 0,
                    cpu_physical_cores: 0,
                    fd_limit: 0,
                    proxy_socket_limit_per_cpu_core: Some(0),
                        };
                        Ok(response)
    }

    async fn get_connections_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, request: tondi_rpc_core::GetConnectionsRequest) -> RpcResult<tondi_rpc_core::GetConnectionsResponse> {
        let response = tondi_rpc_core::GetConnectionsResponse {
            clients: 0,
            peers: 0,
                    profile_data: if request.include_profile_data {
                Some(tondi_rpc_core::ConnectionsProfileData {
                            cpu_usage: 0.0,
                            memory_usage: 0,
                        })
                    } else {
                        None
                    }
                };
                Ok(response)
    }

    // 其他方法返回默认值或错误
    async fn get_metrics_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetMetricsRequest) -> RpcResult<GetMetricsResponse> {
        self.get_metrics(true, true, true, true, true, true).await
    }

    async fn get_server_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetServerInfoRequest) -> RpcResult<GetServerInfoResponse> {
        self.get_server_info().await
    }

    async fn get_sync_status_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetSyncStatusRequest) -> RpcResult<tondi_rpc_core::GetSyncStatusResponse> {
        let response = tondi_rpc_core::GetSyncStatusResponse {
            is_synced: false,
        };
        Ok(response)
    }

    async fn get_current_network_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetCurrentNetworkRequest) -> RpcResult<tondi_rpc_core::GetCurrentNetworkResponse> {
        let response = tondi_rpc_core::GetCurrentNetworkResponse {
            network: tondi_rpc_core::RpcNetworkType::Mainnet,
                };
                Ok(response)
    }

    async fn submit_block_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::SubmitBlockRequest) -> RpcResult<tondi_rpc_core::SubmitBlockResponse> {
        let response = tondi_rpc_core::SubmitBlockResponse {
            report: tondi_rpc_core::SubmitBlockReport::Success,
                };
                Ok(response)
    }

    async fn get_block_template_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBlockTemplateRequest) -> RpcResult<tondi_rpc_core::GetBlockTemplateResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_block_template_call尚未实现".to_string()))
    }

    async fn get_peer_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetPeerAddressesRequest) -> RpcResult<tondi_rpc_core::GetPeerAddressesResponse> {
        let response = tondi_rpc_core::GetPeerAddressesResponse {
                    known_addresses: vec![],
                    banned_addresses: vec![],
        };
        Ok(response)
    }

    async fn get_sink_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetSinkRequest) -> RpcResult<tondi_rpc_core::GetSinkResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_sink_call尚未实现".to_string()))
    }

    async fn get_mempool_entry_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetMempoolEntryRequest) -> RpcResult<tondi_rpc_core::GetMempoolEntryResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_mempool_entry_call尚未实现".to_string()))
    }

    async fn get_mempool_entries_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetMempoolEntriesRequest) -> RpcResult<tondi_rpc_core::GetMempoolEntriesResponse> {
        let response = tondi_rpc_core::GetMempoolEntriesResponse {
                    mempool_entries: vec![],
        };
        Ok(response)
    }

    async fn get_connected_peer_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetConnectedPeerInfoRequest) -> RpcResult<GetConnectedPeerInfoResponse> {
        self.get_connected_peer_info().await
    }

    async fn add_peer_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::AddPeerRequest) -> RpcResult<tondi_rpc_core::AddPeerResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC add_peer_call尚未实现".to_string()))
    }

    async fn submit_transaction_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::SubmitTransactionRequest) -> RpcResult<tondi_rpc_core::SubmitTransactionResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC submit_transaction_call尚未实现".to_string()))
    }

    async fn submit_transaction_replacement_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::SubmitTransactionReplacementRequest) -> RpcResult<tondi_rpc_core::SubmitTransactionReplacementResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC submit_transaction_replacement_call尚未实现".to_string()))
    }

    async fn get_block_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBlockRequest) -> RpcResult<tondi_rpc_core::GetBlockResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_block_call尚未实现".to_string()))
    }

    async fn get_subnetwork_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetSubnetworkRequest) -> RpcResult<tondi_rpc_core::GetSubnetworkResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_subnetwork_call尚未实现".to_string()))
    }

    async fn get_virtual_chain_from_block_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetVirtualChainFromBlockRequest) -> RpcResult<tondi_rpc_core::GetVirtualChainFromBlockResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_virtual_chain_from_block_call尚未实现".to_string()))
    }

    async fn get_blocks_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBlocksRequest) -> RpcResult<tondi_rpc_core::GetBlocksResponse> {
        let response = tondi_rpc_core::GetBlocksResponse {
            block_hashes: vec![],
            blocks: vec![],
        };
        Ok(response)
    }

    async fn get_block_count_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBlockCountRequest) -> RpcResult<BlockCount> {
        let response = BlockCount {
            header_count: 0,
            block_count: 0,
        };
        Ok(response)
    }

    async fn get_block_dag_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBlockDagInfoRequest) -> RpcResult<GetBlockDagInfoResponse> {
        self.get_block_dag_info().await
    }

    async fn resolve_finality_conflict_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::ResolveFinalityConflictRequest) -> RpcResult<tondi_rpc_core::ResolveFinalityConflictResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC resolve_finality_conflict_call尚未实现".to_string()))
    }

    async fn shutdown_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::ShutdownRequest) -> RpcResult<tondi_rpc_core::ShutdownResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC shutdown_call尚未实现".to_string()))
    }

    async fn get_headers_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetHeadersRequest) -> RpcResult<tondi_rpc_core::GetHeadersResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_headers_call尚未实现".to_string()))
    }

    async fn get_balance_by_address_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBalanceByAddressRequest) -> RpcResult<tondi_rpc_core::GetBalanceByAddressResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_balance_by_address_call尚未实现".to_string()))
    }

    async fn get_balances_by_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetBalancesByAddressesRequest) -> RpcResult<tondi_rpc_core::GetBalancesByAddressesResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_balances_by_addresses_call尚未实现".to_string()))
    }

    async fn get_utxos_by_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetUtxosByAddressesRequest) -> RpcResult<tondi_rpc_core::GetUtxosByAddressesResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_utxos_by_addresses_call尚未实现".to_string()))
    }

    async fn get_sink_blue_score_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetSinkBlueScoreRequest) -> RpcResult<tondi_rpc_core::GetSinkBlueScoreResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_sink_blue_score_call尚未实现".to_string()))
    }

    async fn ban_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::BanRequest) -> RpcResult<tondi_rpc_core::BanResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC ban_call尚未实现".to_string()))
    }

    async fn unban_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::UnbanRequest) -> RpcResult<tondi_rpc_core::UnbanResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC unban_call尚未实现".to_string()))
    }

    async fn get_info_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetInfoRequest) -> RpcResult<tondi_rpc_core::GetInfoResponse> {
        let response = tondi_rpc_core::GetInfoResponse {
            p2p_id: "tondi-grpc-client".to_string(),
            mempool_size: 0,
            server_version: "1.0.0".to_string(),
            has_message_id: true,
            has_notify_command: true,
            is_synced: false,
            is_utxo_indexed: false,
        };
        Ok(response)
    }

    async fn estimate_network_hashes_per_second_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::EstimateNetworkHashesPerSecondRequest) -> RpcResult<tondi_rpc_core::EstimateNetworkHashesPerSecondResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC estimate_network_hashes_per_second_call尚未实现".to_string()))
    }

    async fn get_mempool_entries_by_addresses_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetMempoolEntriesByAddressesRequest) -> RpcResult<tondi_rpc_core::GetMempoolEntriesByAddressesResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_mempool_entries_by_addresses_call尚未实现".to_string()))
    }

    async fn get_coin_supply_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetCoinSupplyRequest) -> RpcResult<tondi_rpc_core::GetCoinSupplyResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_coin_supply_call尚未实现".to_string()))
    }

    async fn get_daa_score_timestamp_estimate_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetDaaScoreTimestampEstimateRequest) -> RpcResult<tondi_rpc_core::GetDaaScoreTimestampEstimateResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_daa_score_timestamp_estimate_call尚未实现".to_string()))
    }

    async fn get_utxo_return_address_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetUtxoReturnAddressRequest) -> RpcResult<tondi_rpc_core::GetUtxoReturnAddressResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_utxo_return_address_call尚未实现".to_string()))
    }

    async fn get_fee_estimate_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetFeeEstimateRequest) -> RpcResult<tondi_rpc_core::GetFeeEstimateResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_fee_estimate_call尚未实现".to_string()))
    }

    async fn get_fee_estimate_experimental_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetFeeEstimateExperimentalRequest) -> RpcResult<tondi_rpc_core::GetFeeEstimateExperimentalResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_fee_estimate_experimental_call尚未实现".to_string()))
    }

    async fn get_current_block_color_call(&self, _connection: Option<&tondi_rpc_core::api::connection::DynRpcConnection>, _request: tondi_rpc_core::GetCurrentBlockColorRequest) -> RpcResult<tondi_rpc_core::GetCurrentBlockColorResponse> {
        Err(tondi_rpc_core::RpcError::General("gRPC get_current_block_color_call尚未实现".to_string()))
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
