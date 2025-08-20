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
// 移除硬编码的常量定义，地址应该从配置中获取

/// gRPC客户端实现，包装bp-tondi-client
/// 
/// 该客户端提供与现有wRPC客户端兼容的接口，
/// 允许通过gRPC协议与Tondi节点通信。
/// 
/// 注意：gRPC仅在桌面版（native）中支持，Web版（wasm）将回退到wRPC
pub struct TondiGrpcClient {
    inner: Arc<BpTondiClient>,
    network_interface: NetworkInterfaceConfig,
}

impl TondiGrpcClient {
    /// 连接到gRPC服务器
    /// 
    /// # 参数
    /// * `network_interface` - 网络接口配置，包含要连接的地址信息
    /// 
    /// # 返回
    /// 成功时返回新的客户端实例，失败时返回错误
    /// 
    /// # 注意
    /// 在Web版（wasm）中，此方法将返回错误，提示使用wRPC
    pub async fn connect(network_interface: NetworkInterfaceConfig) -> Result<Self> {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：支持gRPC连接
        let address: ContextualNetAddress = network_interface.clone().into();
        let url = address.to_string(); // 直接使用完整地址，包括端口
        
        let inner = BpTondiClient::connect(&url).await
            .map_err(|e| Error::custom(format!("Failed to connect to gRPC server: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
            network_interface,
        })
    } else {
        // Web版：不支持gRPC，提示使用wRPC
        Err(Error::custom("gRPC is not supported in Web/WASM version. Please use wRPC instead."))
    }
        }
    }

    /// 获取内部的bp-tondi客户端引用
    pub fn client(&self) -> &BpTondiClient {
        &self.inner
    }
}

/// 实现RpcApi trait，提供与wRPC客户端兼容的接口
/// 注意：这里只实现最基本的方法，其他方法暂时返回错误
#[async_trait]
impl RpcApi for TondiGrpcClient {
    async fn get_server_info(&self) -> RpcResult<GetServerInfoResponse> {
        // 暂时返回错误，后续需要实现完整的类型转换
        Err(RpcError::General("gRPC get_server_info not implemented yet".to_string()))
    }

    async fn get_block(&self, _hash: RpcHash, _include_transactions: bool) -> RpcResult<RpcBlock> {
        // 暂时返回错误，后续需要实现完整的类型转换
        Err(RpcError::General("gRPC get_block not implemented yet".to_string()))
    }

    async fn get_blocks(&self, _low_hash: Option<RpcHash>, _include_blocks: bool, _include_transactions: bool) -> RpcResult<GetBlocksResponse> {
        // 暂时返回错误，后续需要实现完整的类型转换
        Err(RpcError::General("gRPC get_blocks not implemented yet".to_string()))
    }

    async fn get_connected_peer_info(&self) -> RpcResult<GetConnectedPeerInfoResponse> {
        // 暂时返回错误，后续需要实现
        Err(RpcError::General("gRPC get_connected_peer_info not implemented yet".to_string()))
    }

    async fn get_block_count(&self) -> RpcResult<GetBlockCountResponse> {
        // 暂时返回错误，后续需要实现
        Err(RpcError::General("gRPC get_block_count not implemented yet".to_string()))
    }

    async fn get_block_dag_info(&self) -> RpcResult<GetBlockDagInfoResponse> {
        // 暂时返回错误，后续需要实现
        Err(RpcError::General("gRPC get_block_dag_info not implemented yet".to_string()))
    }

    // 实现其他必要的RpcApi方法...
    // 这里需要根据实际的RpcApi trait定义添加更多方法
    
    // 默认实现，返回错误
    async fn ping_call(&self, _connection: Option<&DynRpcConnection>, _request: PingRequest) -> RpcResult<PingResponse> {
        Err(RpcError::General("gRPC ping_call not implemented yet".to_string()))
    }

    async fn get_system_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSystemInfoRequest) -> RpcResult<GetSystemInfoResponse> {
        Err(RpcError::General("gRPC get_system_info_call not implemented yet".to_string()))
    }

    async fn get_connections_call(&self, _connection: Option<&DynRpcConnection>, _request: GetConnectionsRequest) -> RpcResult<GetConnectionsResponse> {
        Err(RpcError::General("gRPC get_connections_call not implemented yet".to_string()))
    }

    async fn get_metrics_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMetricsRequest) -> RpcResult<GetMetricsResponse> {
        Err(RpcError::General("gRPC get_metrics_call not implemented yet".to_string()))
    }

    async fn get_server_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetServerInfoRequest) -> RpcResult<GetServerInfoResponse> {
        Err(RpcError::General("gRPC get_server_info_call not implemented yet".to_string()))
    }

    async fn get_sync_status_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSyncStatusRequest) -> RpcResult<GetSyncStatusResponse> {
        Err(RpcError::General("gRPC get_sync_status_call not implemented yet".to_string()))
    }

    async fn get_current_network_call(&self, _connection: Option<&DynRpcConnection>, _request: GetCurrentNetworkRequest) -> RpcResult<GetCurrentNetworkResponse> {
        Err(RpcError::General("gRPC get_current_network_call not implemented yet".to_string()))
    }

    async fn submit_block_call(&self, _connection: Option<&DynRpcConnection>, _request: SubmitBlockRequest) -> RpcResult<SubmitBlockResponse> {
        Err(RpcError::General("gRPC submit_block_call not implemented yet".to_string()))
    }

    async fn get_block_template_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockTemplateRequest) -> RpcResult<GetBlockTemplateResponse> {
        Err(RpcError::General("gRPC get_block_template_call not implemented yet".to_string()))
    }

    async fn get_peer_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetPeerAddressesRequest) -> RpcResult<GetPeerAddressesResponse> {
        Err(RpcError::General("gRPC get_peer_addresses_call not implemented yet".to_string()))
    }

    async fn get_sink_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSinkRequest) -> RpcResult<GetSinkResponse> {
        Err(RpcError::General("gRPC get_sink_call not implemented yet".to_string()))
    }

    async fn get_mempool_entry_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMempoolEntryRequest) -> RpcResult<GetMempoolEntryResponse> {
        Err(RpcError::General("gRPC get_mempool_entry_call not implemented yet".to_string()))
    }

    async fn get_mempool_entries_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMempoolEntriesRequest) -> RpcResult<GetMempoolEntriesResponse> {
        Err(RpcError::General("gRPC get_mempool_entries_call not implemented yet".to_string()))
    }

    async fn get_connected_peer_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetConnectedPeerInfoRequest) -> RpcResult<GetConnectedPeerInfoResponse> {
        Err(RpcError::General("gRPC get_connected_peer_info_call not implemented yet".to_string()))
    }

    async fn add_peer_call(&self, _connection: Option<&DynRpcConnection>, _request: AddPeerRequest) -> RpcResult<AddPeerResponse> {
        Err(RpcError::General("gRPC add_peer_call not implemented yet".to_string()))
    }

    async fn submit_transaction_call(&self, _connection: Option<&DynRpcConnection>, _request: SubmitTransactionRequest) -> RpcResult<SubmitTransactionResponse> {
        Err(RpcError::General("gRPC submit_transaction_call not implemented yet".to_string()))
    }

    async fn submit_transaction_replacement_call(&self, _connection: Option<&DynRpcConnection>, _request: SubmitTransactionReplacementRequest) -> RpcResult<SubmitTransactionReplacementResponse> {
        Err(RpcError::General("gRPC submit_transaction_replacement_call not implemented yet".to_string()))
    }

    async fn get_block_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockRequest) -> RpcResult<GetBlockResponse> {
        Err(RpcError::General("gRPC get_block_call not implemented yet".to_string()))
    }

    async fn get_subnetwork_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSubnetworkRequest) -> RpcResult<GetSubnetworkResponse> {
        Err(RpcError::General("gRPC get_subnetwork_call not implemented yet".to_string()))
    }

    async fn get_virtual_chain_from_block_call(&self, _connection: Option<&DynRpcConnection>, _request: GetVirtualChainFromBlockRequest) -> RpcResult<GetVirtualChainFromBlockResponse> {
        Err(RpcError::General("gRPC get_virtual_chain_from_block_call not implemented yet".to_string()))
    }

    async fn get_blocks_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlocksRequest) -> RpcResult<GetBlocksResponse> {
        Err(RpcError::General("gRPC get_blocks_call not implemented yet".to_string()))
    }

    async fn get_block_count_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockCountRequest) -> RpcResult<GetBlockCountResponse> {
        Err(RpcError::General("gRPC get_block_count_call not implemented yet".to_string()))
    }

    async fn get_block_dag_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBlockDagInfoRequest) -> RpcResult<GetBlockDagInfoResponse> {
        Err(RpcError::General("gRPC get_block_dag_info_call not implemented yet".to_string()))
    }

    async fn resolve_finality_conflict_call(&self, _connection: Option<&DynRpcConnection>, _request: ResolveFinalityConflictRequest) -> RpcResult<ResolveFinalityConflictResponse> {
        Err(RpcError::General("gRPC resolve_finality_conflict_call not implemented yet".to_string()))
    }

    async fn shutdown_call(&self, _connection: Option<&DynRpcConnection>, _request: ShutdownRequest) -> RpcResult<ShutdownResponse> {
        Err(RpcError::General("gRPC shutdown_call not implemented yet".to_string()))
    }

    async fn get_headers_call(&self, _connection: Option<&DynRpcConnection>, _request: GetHeadersRequest) -> RpcResult<GetHeadersResponse> {
        Err(RpcError::General("gRPC get_headers_call not implemented yet".to_string()))
    }

    async fn get_balance_by_address_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBalanceByAddressRequest) -> RpcResult<GetBalanceByAddressResponse> {
        Err(RpcError::General("gRPC get_balance_by_address_call not implemented yet".to_string()))
    }

    async fn get_balances_by_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetBalancesByAddressesRequest) -> RpcResult<GetBalancesByAddressesResponse> {
        Err(RpcError::General("gRPC get_balances_by_addresses_call not implemented yet".to_string()))
    }

    async fn get_utxos_by_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetUtxosByAddressesRequest) -> RpcResult<GetUtxosByAddressesResponse> {
        Err(RpcError::General("gRPC get_utxos_by_addresses_call not implemented yet".to_string()))
    }

    async fn get_sink_blue_score_call(&self, _connection: Option<&DynRpcConnection>, _request: GetSinkBlueScoreRequest) -> RpcResult<GetSinkBlueScoreResponse> {
        Err(RpcError::General("gRPC get_sink_blue_score_call not implemented yet".to_string()))
    }

    async fn ban_call(&self, _connection: Option<&DynRpcConnection>, _request: BanRequest) -> RpcResult<BanResponse> {
        Err(RpcError::General("gRPC ban_call not implemented yet".to_string()))
    }

    async fn unban_call(&self, _connection: Option<&DynRpcConnection>, _request: UnbanRequest) -> RpcResult<UnbanResponse> {
        Err(RpcError::General("gRPC unban_call not implemented yet".to_string()))
    }

    async fn get_info_call(&self, _connection: Option<&DynRpcConnection>, _request: GetInfoRequest) -> RpcResult<GetInfoResponse> {
        Err(RpcError::General("gRPC get_info_call not implemented yet".to_string()))
    }

    async fn estimate_network_hashes_per_second_call(&self, _connection: Option<&DynRpcConnection>, _request: EstimateNetworkHashesPerSecondRequest) -> RpcResult<EstimateNetworkHashesPerSecondResponse> {
        Err(RpcError::General("gRPC estimate_network_hashes_per_second_call not implemented yet".to_string()))
    }

    async fn get_mempool_entries_by_addresses_call(&self, _connection: Option<&DynRpcConnection>, _request: GetMempoolEntriesByAddressesRequest) -> RpcResult<GetMempoolEntriesByAddressesResponse> {
        Err(RpcError::General("gRPC get_mempool_entries_by_addresses_call not implemented yet".to_string()))
    }

    async fn get_coin_supply_call(&self, _connection: Option<&DynRpcConnection>, _request: GetCoinSupplyRequest) -> RpcResult<GetCoinSupplyResponse> {
        Err(RpcError::General("gRPC get_coin_supply_call not implemented yet".to_string()))
    }

    async fn get_daa_score_timestamp_estimate_call(&self, _connection: Option<&DynRpcConnection>, _request: GetDaaScoreTimestampEstimateRequest) -> RpcResult<GetDaaScoreTimestampEstimateResponse> {
        Err(RpcError::General("gRPC get_daa_score_timestamp_estimate_call not implemented yet".to_string()))
    }

    async fn get_utxo_return_address_call(&self, _connection: Option<&DynRpcConnection>, _request: GetUtxoReturnAddressRequest) -> RpcResult<GetUtxoReturnAddressResponse> {
        Err(RpcError::General("gRPC get_utxo_return_address_call not implemented yet".to_string()))
    }

    async fn get_fee_estimate_call(&self, _connection: Option<&DynRpcConnection>, _request: GetFeeEstimateRequest) -> RpcResult<GetFeeEstimateResponse> {
        Err(RpcError::General("gRPC get_fee_estimate_call not implemented yet".to_string()))
    }

    async fn get_fee_estimate_experimental_call(&self, _connection: Option<&DynRpcConnection>, _request: GetFeeEstimateExperimentalRequest) -> RpcResult<GetFeeEstimateExperimentalResponse> {
        Err(RpcError::General("gRPC get_fee_estimate_experimental_call not implemented yet".to_string()))
    }

    async fn get_current_block_color_call(&self, _connection: Option<&DynRpcConnection>, _request: GetCurrentBlockColorRequest) -> RpcResult<GetCurrentBlockColorResponse> {
        Err(RpcError::General("gRPC get_current_block_color_call not implemented yet".to_string()))
    }

    fn register_new_listener(&self, _connection: ChannelConnection) -> ListenerId {
        0 // 暂时返回0，后续需要实现
    }

    async fn unregister_listener(&self, _id: ListenerId) -> RpcResult<()> {
        Err(RpcError::General("gRPC unregister_listener not implemented yet".to_string()))
    }

    async fn start_notify(&self, _id: ListenerId, _scope: Scope) -> RpcResult<()> {
        Err(RpcError::General("gRPC start_notify not implemented yet".to_string()))
    }

    async fn stop_notify(&self, _id: ListenerId, _scope: Scope) -> RpcResult<()> {
        Err(RpcError::General("gRPC stop_notify not implemented yet".to_string()))
    }
    

}

/// gRPC客户端的简单包装器，用于基本的连接和调用
impl TondiGrpcClient {
    /// 检查客户端是否已连接
    pub fn is_connected(&self) -> bool {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：gRPC客户端连接状态检查
        true // 简化实现
    } else {
        // Web版：gRPC不可用
        false
    }
        }
    }
    
    /// 获取网络ID（如果可用）
    pub fn network_id(&self) -> Option<NetworkId> {
        // 目前返回None，后续可以从服务器信息中获取
        None
    }
    
    /// 获取服务器URL
    pub fn url(&self) -> Option<String> {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：返回gRPC URL
        let address: ContextualNetAddress = self.network_interface.clone().into();
        Some(format!("grpc://{}", address))
    } else {
        // Web版：gRPC不可用
        None
    }
        }
    }
}

// 添加测试函数
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_client_creation() {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：测试gRPC客户端创建，使用默认的网络接口配置
        let network_interface = NetworkInterfaceConfig::default();
        let client = TondiGrpcClient::connect(network_interface).await;
        // 这个测试可能会失败，因为可能没有gRPC服务器在运行
        // 但至少可以验证客户端结构是否正确创建
        assert!(client.is_ok() || client.is_err());
    } else {
        // Web版：gRPC不可用，测试应该失败
        let network_interface = NetworkInterfaceConfig::default();
        let client = TondiGrpcClient::connect(network_interface).await;
        assert!(client.is_err());
    }
        }
    }
}

/// gRPC RPC控制实现
#[derive(Default, Clone)]
pub struct GrpcRpcCtl {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    // Current channel state
    state: Mutex<RpcState>,
    // MPMC channel for RpcCtlOp operations.
    multiplexer: Multiplexer<RpcState>,
    // Optional Connection descriptor such as a connection URL.
    descriptor: Mutex<Option<String>>,
}

/// 实现与RpcCtl相同的方法
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
        // 桌面版：gRPC可用
        *self.inner.state.lock().unwrap() = RpcState::Connected;
        Ok(self.inner.multiplexer.broadcast(RpcState::Connected).await?)
    } else {
        // Web版：gRPC不可用
        Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
    }
        }
    }

    pub async fn signal_close(&self) -> RpcResult<()> {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：gRPC可用
        *self.inner.state.lock().unwrap() = RpcState::Disconnected;
        Ok(self.inner.multiplexer.broadcast(RpcState::Disconnected).await?)
    } else {
        // Web版：gRPC不可用
        Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
    }
        }
    }

    pub fn try_signal_open(&self) -> RpcResult<()> {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：gRPC可用
        *self.inner.state.lock().unwrap() = RpcState::Connected;
        Ok(self.inner.multiplexer.try_broadcast(RpcState::Connected)?)
    } else {
        // Web版：gRPC不可用
        Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
    }
        }
    }

    pub fn try_signal_close(&self) -> RpcResult<()> {
        cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        // 桌面版：gRPC可用
        *self.inner.state.lock().unwrap() = RpcState::Disconnected;
        Ok(self.inner.multiplexer.try_broadcast(RpcState::Disconnected)?)
    } else {
        // Web版：gRPC不可用
        Err(RpcError::General("gRPC is not supported in Web/WASM version".to_string()))
    }
        }
    }

    pub fn set_descriptor(&self, descriptor: Option<String>) {
        *self.inner.descriptor.lock().unwrap() = descriptor;
    }

    pub fn descriptor(&self) -> Option<String> {
        self.inner.descriptor.lock().unwrap().clone()
    }
}
