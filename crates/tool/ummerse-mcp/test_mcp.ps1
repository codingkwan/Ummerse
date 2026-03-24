# test_mcp.ps1 - 快速测试 Ummerse MCP Server 功能
# 使用方式：在 PowerShell 中运行 .\crates\ummerse-mcp\test_mcp.ps1
#
# 注意：此脚本会启动 MCP Server，发送一系列测试请求，然后检查响应

$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

Write-Host "=== Ummerse MCP Server 功能测试 ===" -ForegroundColor Cyan
Write-Host ""

# 准备测试请求序列
$requests = @(
    '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}',
    '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}',
    '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_scene","arguments":{}}}',
    '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"list_entities","arguments":{}}}',
    '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"move_block","arguments":{"name":"MainBlock","dx":50}}}',
    '{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"get_entity","arguments":{"name":"MainBlock"}}}',
    '{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"spawn_entity","arguments":{"name":"TestEnemy","kind":"circle","x":300,"y":200}}}',
    '{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"set_property","arguments":{"name":"MainBlock","property":"rotation","value":1.57}}}',
    '{"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"despawn_entity","arguments":{"name":"TestEnemy"}}}',
    '{"jsonrpc":"2.0","id":10,"method":"ping","params":{}}'
)

$input_data = ($requests -join "`n") + "`n"

Write-Host "启动 MCP Server 并发送 $($requests.Count) 个请求..." -ForegroundColor Yellow
Write-Host ""

# 启动 cargo run 并通过 stdin 传入请求
$process = New-Object System.Diagnostics.Process
$process.StartInfo.FileName = "cargo"
$process.StartInfo.Arguments = "run -p ummerse-mcp"
$process.StartInfo.UseShellExecute = $false
$process.StartInfo.RedirectStandardInput = $true
$process.StartInfo.RedirectStandardOutput = $true
$process.StartInfo.RedirectStandardError = $false  # stderr（日志）不重定向，直接显示
$process.StartInfo.WorkingDirectory = (Get-Location).Path

$process.Start() | Out-Null

# 写入所有请求后关闭 stdin
$process.StandardInput.Write($input_data)
$process.StandardInput.Close()

# 读取所有响应
$responses = @()
while (-not $process.StandardOutput.EndOfStream) {
    $line = $process.StandardOutput.ReadLine()
    if ($line) {
        $responses += $line
    }
}

$process.WaitForExit(10000) | Out-Null

Write-Host "=== 收到 $($responses.Count) 个响应 ===" -ForegroundColor Green
Write-Host ""

# 解析并展示每个响应
$testsPassed = 0
$testsFailed = 0

for ($i = 0; $i -lt $responses.Count; $i++) {
    $resp = $responses[$i] | ConvertFrom-Json -ErrorAction SilentlyContinue
    
    if ($null -eq $resp) {
        Write-Host "[$($i+1)] 解析失败: $($responses[$i])" -ForegroundColor Red
        $testsFailed++
        continue
    }
    
    $id = $resp.id
    $method = $requests[$i] | ConvertFrom-Json | Select-Object -ExpandProperty method -ErrorAction SilentlyContinue
    
    if ($resp.error) {
        Write-Host "[$id] ❌ $method → 错误: $($resp.error.message)" -ForegroundColor Red
        $testsFailed++
    } else {
        Write-Host "[$id] ✅ $method → 成功" -ForegroundColor Green
        $testsPassed++
        
        # 特殊展示部分响应内容
        if ($method -eq "tools/list") {
            $toolCount = $resp.result.tools.Count
            Write-Host "      工具数量: $toolCount" -ForegroundColor DarkGray
        }
        elseif ($method -eq "tools/call") {
            $content = $resp.result.content[0].text
            if ($content.Length -gt 80) { $content = $content.Substring(0, 80) + "..." }
            Write-Host "      结果: $content" -ForegroundColor DarkGray
        }
    }
}

Write-Host ""
Write-Host "=== 测试结果: $testsPassed 通过, $testsFailed 失败 ===" -ForegroundColor $(if ($testsFailed -eq 0) { "Green" } else { "Yellow" })
