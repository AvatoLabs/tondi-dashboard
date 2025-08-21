use std::sync::Arc;
use tondi_rpc_core::*;
use tondi_dashboard_core::runtime::services::tondi::grpc_client::TondiGrpcClient;
use tondi_dashboard_core::settings::{Network, NetworkInterfaceConfig, NetworkInterfaceKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 测试gRPC客户端集成...");
    
    // 创建网络接口配置
    let mut network_interface = NetworkInterfaceConfig::default();
    network_interface.kind = NetworkInterfaceKind::Custom;
    network_interface.custom = "8.210.45.192:16610".parse().unwrap();
    
    // 连接到gRPC服务器
    println!("连接到: 8.210.45.192:16610");
    let client = TondiGrpcClient::connect(network_interface, Network::Devnet).await?;
    println!("✅ gRPC连接成功");
    
    // 测试get_server_info
    println!("\n测试 get_server_info...");
    match client.get_server_info().await {
        Ok(info) => {
            println!("✅ 服务器信息获取成功:");
            println!("   RPC API版本: {}", info.rpc_api_version);
            println!("   RPC API修订: {}", info.rpc_api_revision);
            println!("   服务器版本: {}", info.server_version);
            println!("   网络ID: {:?}", info.network_id);
            println!("   有UTXO索引: {}", info.has_utxo_index);
            println!("   已同步: {}", info.is_synced);
            println!("   虚拟DAA分数: {}", info.virtual_daa_score);
        }
        Err(e) => println!("❌ get_server_info失败: {}", e),
    }
    
    // 测试get_blocks
    println!("\n测试 get_blocks...");
    match client.get_blocks(None, false, false).await {
        Ok(blocks) => {
            println!("✅ 区块信息获取成功:");
            println!("   区块数量: {}", blocks.block_hashes.len());
            if !blocks.block_hashes.is_empty() {
                println!("   第一个区块哈希: {:?}", blocks.block_hashes[0]);
            }
        }
        Err(e) => println!("❌ get_blocks失败: {}", e),
    }
    
    // 测试get_metrics_call
    println!("\n测试 get_metrics_call...");
    let metrics_request = GetMetricsRequest {};
    match client.get_metrics_call(None, metrics_request).await {
        Ok(metrics) => {
            println!("✅ Metrics获取成功:");
            println!("   服务器时间: {}", metrics.server_time);
            if let Some(consensus) = &metrics.consensus_metrics {
                println!("   共识指标:");
                println!("     数据库区块数: {}", consensus.node_database_blocks_count);
                println!("     数据库头数: {}", consensus.node_database_headers_count);
                println!("     提交区块数: {}", consensus.node_blocks_submitted_count);
                println!("     处理头数: {}", consensus.node_headers_processed_count);
            }
        }
        Err(e) => println!("❌ get_metrics_call失败: {}", e),
    }
    
    println!("\n🎯 gRPC客户端集成测试完成！");
    Ok(())
}
