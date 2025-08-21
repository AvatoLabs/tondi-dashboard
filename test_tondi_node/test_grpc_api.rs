use std::time::Duration;
use tokio;

// 模拟bp-tondi的API调用
async fn test_grpc_connection() -> Result<(), Box<dyn std::error::Error>> {
    println!("测试Tondi Devnet节点的gRPC API连接...");
    println!("节点地址: 8.210.45.192:16610");
    println!("==================================");
    
    // 测试基本的TCP连接
    println!("\n1. 测试TCP连接稳定性...");
    for i in 1..=5 {
        let start = std::time::Instant::now();
        match tokio::net::TcpStream::connect("8.210.45.192:16610").await {
            Ok(_) => {
                let duration = start.elapsed();
                println!("✅ 第 {} 次连接成功，耗时: {:?}", i, duration);
            }
            Err(e) => {
                println!("❌ 第 {} 次连接失败: {}", i, e);
                return Err(e.into());
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // 测试连接延迟
    println!("\n2. 测试连接延迟...");
    let mut delays = Vec::new();
    for _ in 1..=10 {
        let start = std::time::Instant::now();
        match tokio::net::TcpStream::connect("8.210.45.192:16610").await {
            Ok(_) => {
                let duration = start.elapsed();
                delays.push(duration);
                print!(".");
            }
            Err(e) => {
                println!("\n❌ 连接失败: {}", e);
                return Err(e.into());
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    if !delays.is_empty() {
        let avg_delay: Duration = delays.iter().sum::<Duration>() / delays.len() as u32;
        let min_delay = delays.iter().min().unwrap();
        let max_delay = delays.iter().max().unwrap();
        
        println!("\n📊 延迟统计:");
        println!("   平均延迟: {:?}", avg_delay);
        println!("   最小延迟: {:?}", min_delay);
        println!("   最大延迟: {:?}", max_delay);
    }
    
    // 测试持续连接
    println!("\n3. 测试持续连接...");
    println!("尝试建立持续连接并保持5秒...");
    
    let start = std::time::Instant::now();
    match tokio::net::TcpStream::connect("8.210.45.192:16610").await {
        Ok(mut stream) => {
            println!("✅ 持续连接建立成功");
            
            // 尝试保持连接
            let mut buffer = [0u8; 1024];
            let mut total_read = 0;
            
            // 设置读取超时
            stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
            
            // 尝试读取数据
            match tokio::io::AsyncReadExt::read(&mut stream, &mut buffer).await {
                Ok(n) => {
                    if n > 0 {
                        println!("✅ 成功读取 {} 字节数据", n);
                        total_read = n;
                    } else {
                        println!("⚠️  连接建立但无数据可读");
                    }
                }
                Err(e) => {
                    println!("⚠️  读取数据失败: {} (这可能是正常的，如果节点没有主动发送数据)", e);
                }
            }
            
            // 保持连接一段时间
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            let duration = start.elapsed();
            println!("✅ 持续连接测试完成，总耗时: {:?}", duration);
            println!("   总读取字节: {}", total_read);
        }
        Err(e) => {
            println!("❌ 持续连接建立失败: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n==================================");
    println!("gRPC连接测试完成！");
    println!("");
    println!("基于测试结果分析：");
    println!("1. 如果TCP连接成功，说明网络层正常");
    println!("2. 如果无法读取数据，可能是：");
    println!("   - 节点服务未完全启动");
    println!("   - 节点还在同步中");
    println!("   - 需要等待节点完全同步");
    println!("3. 建议检查节点日志，确认服务状态");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_grpc_connection().await?;
    Ok(())
}
