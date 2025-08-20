use std::net::SocketAddr;
use std::str::FromStr;
use tokio;

// 注意：这个测试需要bp-tondi依赖，在实际环境中需要添加到Cargo.toml

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始测试tondi dev节点的gRPC连接...");
    
    // 您的节点地址
    let node_url = "8.210.45.192";
    let node_port = 17110; // 默认gRPC端口
    
    println!("尝试连接到节点: {}:{}", node_url, node_port);
    
    // 测试基本的网络连接
    let socket_addr = SocketAddr::from_str(&format!("{}:{}", node_url, node_port))?;
    
    // 尝试建立TCP连接
    match tokio::net::TcpStream::connect(socket_addr).await {
        Ok(_) => {
            println!("✅ TCP连接成功建立到 {}:{}", node_url, node_port);
        }
        Err(e) => {
            println!("❌ TCP连接失败: {}", e);
            return Err(e.into());
        }
    }
    
    // 测试gRPC连接
    let grpc_url = format!("http://{}:{}", node_url, node_port);
    println!("尝试gRPC连接: {}", grpc_url);
    
    // 这里可以添加实际的gRPC客户端测试代码
    // 需要bp-tondi依赖
    
    println!("\n测试完成！");
    println!("TCP连接成功，说明节点网络可达");
    println!("要测试完整的gRPC功能，需要添加bp-tondi依赖");
    
    Ok(())
}
