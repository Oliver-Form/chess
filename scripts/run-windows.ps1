Write-Host "🚀 Setting up Chess Game for Windows..." -ForegroundColor Green

# Download the Windows binary
Write-Host "📥 Downloading chess server..." -ForegroundColor Yellow
Invoke-WebRequest -Uri "https://github.com/Oliver-Form/chess/releases/latest/download/chess-windows-x86_64.exe" -OutFile "chess-server.exe"

# Start the server in background
Write-Host "🔧 Starting chess server..." -ForegroundColor Yellow
$process = Start-Process -FilePath ".\chess-server.exe" -PassThru

# Clone the repository for frontend assets
Write-Host "📂 Downloading frontend assets..." -ForegroundColor Yellow
git clone https://github.com/Oliver-Form/chess.git

# Navigate to frontend directory
Set-Location chess/frontend

# Wait for server to start
Write-Host "⏳ Waiting for server to start..." -ForegroundColor Yellow
Start-Sleep 3

# Open two browser windows for multiplayer testing
Write-Host "🎮 Opening chess game in browser..." -ForegroundColor Yellow
Start-Process "chrome.exe" "index.html"
Start-Sleep 1
Start-Process "chrome.exe" "index.html"

Write-Host "✅ Setup complete! Two browser windows should open for multiplayer testing." -ForegroundColor Green
Write-Host "💡 To stop the server later, run: Stop-Process -Id $($process.Id)" -ForegroundColor Cyan
