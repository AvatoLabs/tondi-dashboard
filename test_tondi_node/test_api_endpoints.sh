#!/bin/bash

echo "测试Tondi Devnet节点的具体API端点..."
echo "节点地址: 8.210.45.192"
echo "=================================="

# 测试wRPC端点
echo -e "\n1. 测试wRPC API端点 (端口 17610)..."

# 常见的wRPC端点
wrpc_endpoints=(
    "/"  # 根路径
    "/api/v1/status"  # 状态端点
    "/api/v1/info"    # 信息端点
    "/api/v1/blocks"  # 区块端点
    "/api/v1/peers"   # 节点端点
)

for endpoint in "${wrpc_endpoints[@]}"; do
    echo "测试端点: $endpoint"
    if curl -s --connect-timeout 5 "http://8.210.45.192:17610$endpoint" > /dev/null 2>&1; then
        echo "✅ 端点 $endpoint 响应"
        # 获取响应内容
        response=$(curl -s --connect-timeout 5 "http://8.210.45.192:17610$endpoint" 2>/dev/null)
        if [ ! -z "$response" ]; then
            echo "   响应内容: ${response:0:100}..."
        fi
    else
        echo "❌ 端点 $endpoint 无响应"
    fi
done

# 测试gRPC端点 (通过HTTP/2)
echo -e "\n2. 测试gRPC端点 (端口 16610)..."
echo "注意: gRPC通常使用HTTP/2协议，需要特殊工具测试"

# 尝试使用HTTP/1.1连接到gRPC端口
echo "尝试HTTP连接到gRPC端口..."
if curl -s --connect-timeout 5 "http://8.210.45.192:16610" > /dev/null 2>&1; then
    echo "✅ gRPC端口接受HTTP连接"
    response=$(curl -s --connect-timeout 5 "http://8.210.45.192:16610" 2>/dev/null)
    if [ ! -z "$response" ]; then
        echo "   响应内容: ${response:0:100}..."
    fi
else
    echo "❌ gRPC端口不接受HTTP连接 (这是正常的)"
fi

# 测试WebSocket连接
echo -e "\n3. 测试WebSocket连接..."
echo "尝试建立WebSocket连接..."

# 使用curl测试WebSocket升级
if curl -s --include --no-buffer \
    --header "Connection: Upgrade" \
    --header "Upgrade: websocket" \
    --header "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
    --header "Sec-WebSocket-Version: 13" \
    "http://8.210.45.192:17610" > /dev/null 2>&1; then
    echo "✅ WebSocket升级请求成功"
else
    echo "❌ WebSocket升级请求失败"
fi

# 测试具体的RPC调用
echo -e "\n4. 测试具体的RPC调用..."
echo "尝试调用getServerInfo..."

# 构造一个简单的RPC请求
rpc_request='{"jsonrpc":"2.0","method":"getServerInfo","params":[],"id":1}'

echo "发送RPC请求到端口 17610..."
response=$(curl -s --connect-timeout 5 \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$rpc_request" \
    "http://8.210.45.192:17610" 2>/dev/null)

if [ ! -z "$response" ]; then
    echo "✅ RPC调用成功"
    echo "   响应: ${response:0:200}..."
else
    echo "❌ RPC调用失败或无响应"
fi

echo -e "\n=================================="
echo "API端点测试完成！"
echo ""
echo "分析结果："
echo "1. 如果wRPC端点有响应，说明节点服务正在运行"
echo "2. 如果gRPC端口不接受HTTP连接，这是正常的"
echo "3. 如果WebSocket升级失败，可能是配置问题"
echo "4. 如果RPC调用失败，可能是端点路径不正确"
