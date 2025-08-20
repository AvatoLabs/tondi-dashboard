use std::net::SocketAddr;
use std::str::FromStr;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("开始测试tondi dev节点连接...");
    
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
    
    // 测试HTTP连接（如果节点支持HTTP）
    let http_url = format!("http://{}:{}", node_url, node_port);
    println!("尝试HTTP连接: {}", http_url);
    
    match reqwest::get(&http_url).await {
        Ok(response) => {
            println!("✅ HTTP连接成功，状态码: {}", response.status());
        }
        Err(e) => {
            println!("⚠️  HTTP连接失败（这可能是正常的，如果节点只支持gRPC）: {}", e);
        }
    }
    
    // 测试HTTPS连接
    let https_url = format!("https://{}:{}", node_url, node_port);
    println!("尝试HTTPS连接: {}", https_url);
    
    match reqwest::get(&https_url).await {
        Ok(response) => {
            println!("✅ HTTPS连接成功，状态码: {}", response.status());
        }
        Err(e) => {
            println!("⚠️  HTTPS连接失败（这可能是正常的，如果节点只支持gRPC）: {}", e);
        }
    }
    
    println!("\n测试完成！");
    println!("如果TCP连接成功，说明节点网络可达");
    println!("如果HTTP/HTTPS失败，这通常是正常的，因为tondi节点主要使用gRPC协议");
    
    Ok(())
}
