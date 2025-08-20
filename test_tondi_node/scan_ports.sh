#!/bin/bash

echo "扫描tondi节点可能的端口..."
echo "节点地址: 8.210.45.192"

# 扫描常见的端口范围
echo -e "\n扫描端口 8000-9000..."
for port in {8000..8010}; do
    if curl -s --connect-timeout 2 "http://8.210.45.192:$port" > /dev/null 2>&1; then
        echo "✅ 端口 $port 响应"
    fi
done

echo -e "\n扫描端口 9000-9100..."
for port in {9000..9010}; do
    if curl -s --connect-timeout 2 "http://8.210.45.192:$port" > /dev/null 2>&1; then
        echo "✅ 端口 $port 响应"
    fi
done

echo -e "\n扫描完成！"
