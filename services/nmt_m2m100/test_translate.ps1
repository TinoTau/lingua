# PowerShell 测试脚本（正确处理中文编码）

# 使用 UTF-8 编码
$body = @{
    src_lang = "zh"
    tgt_lang = "en"
    text = "你好"
} | ConvertTo-Json -Compress

# 确保使用 UTF-8 编码
$utf8Body = [System.Text.Encoding]::UTF8.GetBytes($body)

$response = Invoke-RestMethod -Uri http://127.0.0.1:5008/v1/translate `
    -Method POST `
    -ContentType "application/json; charset=utf-8" `
    -Body $utf8Body

Write-Host "翻译结果: $($response.text)"
Write-Host "耗时: $($response.extra.elapsed_ms)ms"

