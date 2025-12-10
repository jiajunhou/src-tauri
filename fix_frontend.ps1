# 修复前端代码脚本
$indexPath = "..\dist\index.html"

Write-Host "正在修复前端代码..." -ForegroundColor Green

# 读取文件内容
$content = Get-Content $indexPath -Raw -Encoding UTF8

# 1. 修复 updateFocusTimer 错误
Write-Host "1. 移除 updateFocusTimer 调用..." -ForegroundColor Yellow
$content = $content -replace 'updateFocusTimer\(\);', '// updateFocusTimer() - Focus功能暂未实现'

# 2. 优化 Todo 容器间距
Write-Host "2. 优化 Todo 页面间距..." -ForegroundColor Yellow

# 修复 todo-container
$content = $content -replace '\.todo-container \{[^}]+max-width: 800px;[^}]+margin: 0 auto;[^}]+padding: 0 20px;', @'
.todo-container {
            max-width: 800px;
            margin: 0 auto;
            padding: 40px 32px;
'@

# 修复 todo-input-wrapper (移除 sticky 定位)
$content = $content -replace '\.todo-input-wrapper \{[^}]+\}', @'
.todo-input-wrapper {
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 12px 20px;
            display: flex;
            align-items: center;
            gap: 16px;
            margin-bottom: 32px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.05);
            transition: box-shadow 0.2s, border-color 0.2s;
        }
'@

# 修复 todo-input
$content = $content -replace '(\.todo-input \{[^}]+padding:) 12px 0;', '$1 0;'
$content = $content -replace '(\.todo-input \{[^}]+font-size:) 16px;', '$1 15px;'

# 修复 todo-list gap
$content = $content -replace '(\.todo-list \{[^}]+gap:) 12px;', '$1 16px;'
$content = $content -replace '(\.todo-list \{[^}]+padding-bottom:) 40px;', '$1 60px;'

# 修复 todo-item
$content = $content -replace '(\.todo-item \{[^}]+padding:) 16px 20px;', '$1 18px 24px;'
$content = $content -replace '(\.todo-item \{[^}]+background-color:) var\(--bg-primary\);', '$1 var(--bg-secondary);'

# 添加 gap 到 todo-item
$oldPattern = '(\.todo-item \{[^}]+position: relative;)'
$newPattern = '$1' + "`n            gap: 4px;"
$content = $content -replace $oldPattern, $newPattern

# 修复 completed 样式
$content = $content -replace '(\.todo-item\.completed \{[^}]+background-color:) var\(--bg-secondary\);', '$1 var(--bg-tertiary);'
$content = $content -replace '(\.todo-item\.completed \{[^}]+border-color:) transparent;', '$1 var(--border);'
$content = $content -replace '(\.todo-item\.completed \{[^}]+opacity:) 0\.8;', '$1 0.7;'

# 修复 checkbox
$content = $content -replace '(\.todo-checkbox-custom \{[^}]+width:) 22px;', '$1 24px;'
$content = $content -replace '(\.todo-checkbox-custom \{[^}]+height:) 22px;', '$1 24px;'
$content = $content -replace '(\.todo-checkbox-custom \{[^}]+margin-right:) 16px;', '$1 0;'

# 修复 todo-title line-height
$oldPattern2 = '(\.todo-title \{[^}]+word-break: break-word;)'
$newPattern2 = '$1' + "`n            line-height: 1.5;"
$content = $content -replace $oldPattern2, $newPattern2

# 修复 todo-description
$content = $content -replace '(\.todo-description \{[^}]+margin-top:) 4px;', '$1 6px;'
$oldPattern3 = '(\.todo-description \{[^}]+margin-top: 6px;)'
$newPattern3 = '$1' + "`n            line-height: 1.4;"
$content = $content -replace $oldPattern3, $newPattern3

# 保存文件
$content | Set-Content $indexPath -Encoding UTF8 -NoNewline

Write-Host "✓ 前端代码修复完成！" -ForegroundColor Green
Write-Host ""
Write-Host "修复内容：" -ForegroundColor Cyan
Write-Host "  1. 移除了 updateFocusTimer 调用" -ForegroundColor White
Write-Host "  2. 优化了 Todo 页面的间距和布局" -ForegroundColor White
Write-Host "  3. 增大了容器内边距 (40px 32px)" -ForegroundColor White
Write-Host "  4. 调整了输入框间距 (12px 20px)" -ForegroundColor White
Write-Host "  5. 增加了列表项间距 (16px)" -ForegroundColor White
Write-Host "  6. 优化了复选框大小和间距" -ForegroundColor White
Write-Host ""
Write-Host "请重新加载应用查看效果" -ForegroundColor Yellow
