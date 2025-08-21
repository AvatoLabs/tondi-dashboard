#!/bin/bash

echo "最终测试tondi devnet节点连接..."
echo "节点地址: 8.210.45.192"

# 测试所有端口
ports=(
    16610  # gRPC接口
    16611  # P2P接口  
    17610  # wRPC接口
)

echo -e "\n1. 测试TCP连接..."
for port in "${ports[@]}"; do
    if timeout 3 bash -c "</dev/tcp/8.210.45.192/$port" 2>/dev/null; then
        echo "✅ TCP连接到端口 $port 成功"
    else
        echo "❌ TCP连接到端口 $port 失败"
    fi
done

echo -e "\n2. 测试端口响应..."
for port in "${ports[@]}"; do
    echo "测试端口 $port..."
    # 尝试发送一些数据看是否有响应
    if timeout 3 bash -c "echo 'test' | timeout 2 bash -c 'cat > /dev/tcp/8.210.45.192/$port'" 2>/dev/null; then
        echo "✅ 端口 $port 可以接收数据"
    else
        echo "⚠️  端口 $port 无数据响应（这可能是正常的）"
    fi
done

echo -e "\n3. 测试网络延迟..."
for port in "${ports[@]}"; do
    echo "测试端口 $port 的网络延迟..."
    start_time=$(date +%s%N)
    if timeout 3 bash -c "</dev/tcp/8.210.45.192/$port" 2>/dev/null; then
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 ))
        echo "✅ 端口 $port 连接延迟: ${duration}ms"
    else
        echo "❌ 端口 $port 连接失败"
    fi
done

echo -e "\n测试完成！"
echo "如果TCP连接成功，说明节点网络可达"
echo "端口 16610: gRPC接口"
echo "端口 16611: P2P接口"
echo "端口 17610: wRPC接口"
