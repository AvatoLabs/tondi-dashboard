use std::sync::Arc;
use tondi_rpc_core::*;
use tondi_dashboard_core::runtime::services::tondi::grpc_client::TondiGrpcClient;
use tondi_dashboard_core::settings::{Network, NetworkInterfaceConfig, NetworkInterfaceKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 测试gRPC客户端基本功能...");
    
    // 创建网络接口配置
    let mut network_interface = NetworkInterfaceConfig::default();
    network_interface.kind = NetworkInterfaceKind::Custom;
    network_interface.custom = "8.210.45.192:16610".parse().unwrap();
    
    // 连接到gRPC服务器
    println!("连接到: 8.210.45.192:16610");
    let client = TondiGrpcClient::connect(network_interface, Network::Devnet).await?;
    println!("✅ gRPC连接成功");
    
    // 测试get_metrics方法（这是RpcApi trait的方法）
    println!("\n测试 get_metrics (RpcApi trait方法)...");
    match client.get_metrics(true, true, true, true, true, false).await {
        Ok(metrics) => {
            println!("✅ get_metrics成功:");
            println!("   服务器时间: {}", metrics.server_time);
            if let Some(consensus) = &metrics.consensus_metrics {
                println!("   共识指标:");
                println!("     数据库区块数: {}", consensus.node_database_blocks_count);
                println!("     数据库头数: {}", consensus.node_database_headers_count);
                println!("     提交区块数: {}", consensus.node_blocks_submitted_count);
                println!("     处理头数: {}", consensus.node_headers_processed_count);
            } else {
                println!("   ❌ consensus_metrics为None");
            }
        }
        Err(e) => {
            println!("❌ get_metrics失败: {}", e);
            println!("   错误类型: {:?}", e);
        }
    }
    
    // 测试get_metrics_call方法（这是我们直接实现的方法）
    println!("\n测试 get_metrics_call (直接实现的方法)...");
    let metrics_request = GetMetricsRequest {
        process_metrics: true,
        connection_metrics: true,
        bandwidth_metrics: true,
        consensus_metrics: true,
        storage_metrics: true,
        custom_metrics: false,
    };
    match client.get_metrics_call(None, metrics_request).await {
        Ok(metrics) => {
            println!("✅ get_metrics_call成功:");
            println!("   服务器时间: {}", metrics.server_time);
            if let Some(consensus) = &metrics.consensus_metrics {
                println!("   共识指标:");
                println!("     数据库区块数: {}", consensus.node_database_blocks_count);
                println!("     数据库头数: {}", consensus.node_database_headers_count);
                println!("     提交区块数: {}", consensus.node_blocks_submitted_count);
                println!("     处理头数: {}", consensus.node_headers_processed_count);
            } else {
                println!("   ❌ consensus_metrics为None");
            }
        }
        Err(e) => {
            println!("❌ get_metrics_call失败: {}", e);
            println!("   错误类型: {:?}", e);
        }
    }
    
    println!("\n🎯 gRPC客户端测试完成！");
    Ok(())
}
