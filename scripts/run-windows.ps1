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

# Try browsers in order of preference
if (Get-Command "chrome.exe" -ErrorAction SilentlyContinue) {
    Write-Host "Using Google Chrome..." -ForegroundColor Green
    Start-Process "chrome.exe" "--new-window --user-data-dir=`"$env:TEMP\chrome_instance1`" index.html"
    Start-Sleep 1
    Start-Process "chrome.exe" "--new-window --user-data-dir=`"$env:TEMP\chrome_instance2`" index.html"
    $BROWSER_NAME = "Chrome"
}
elseif (Get-Command "firefox.exe" -ErrorAction SilentlyContinue) {
    Write-Host "Using Firefox..." -ForegroundColor Green
    Start-Process "firefox.exe" "index.html"
    Start-Sleep 1
    Start-Process "firefox.exe" "index.html"
    $BROWSER_NAME = "Firefox"
}
elseif (Get-Command "msedge.exe" -ErrorAction SilentlyContinue) {
    Write-Host "Using Microsoft Edge..." -ForegroundColor Green
    Start-Process "msedge.exe" "index.html"
    Start-Sleep 1
    Start-Process "msedge.exe" "index.html"
    $BROWSER_NAME = "Edge"
}
elseif (Get-Command "opera.exe" -ErrorAction SilentlyContinue) {
    Write-Host "Using Opera..." -ForegroundColor Green
    Start-Process "opera.exe" "index.html"
    Start-Sleep 1
    Start-Process "opera.exe" "index.html"
    $BROWSER_NAME = "Opera"
}
elseif (Get-Command "brave.exe" -ErrorAction SilentlyContinue) {
    Write-Host "Using Brave..." -ForegroundColor Green
    Start-Process "brave.exe" "index.html"
    Start-Sleep 1
    Start-Process "brave.exe" "index.html"
    $BROWSER_NAME = "Brave"
}
else {
    Write-Host "❌ No supported browser found!" -ForegroundColor Red
    Write-Host "Please install one of: Chrome, Firefox, Edge, Opera, or Brave" -ForegroundColor Yellow
    Write-Host "Then manually open: index.html in two browser tabs" -ForegroundColor Yellow
    $BROWSER_NAME = "manual"
}

if ($BROWSER_NAME -ne "manual") {
    Write-Host "✅ Setup complete! Two browser windows should open for multiplayer testing." -ForegroundColor Green
    Write-Host ""
    Write-Host "🎯 HOW TO TEST MULTIPLAYER:" -ForegroundColor Yellow
    Write-Host "   1. Two $BROWSER_NAME windows should have opened automatically" -ForegroundColor White
    Write-Host "   2. Each window represents a different player" -ForegroundColor White
    Write-Host "   3. One window will be WHITE, the other BLACK" -ForegroundColor White
    Write-Host "   4. Click on pieces in one window to move, watch the other window update!" -ForegroundColor White
    Write-Host "   5. Take turns making moves between the two windows" -ForegroundColor White
} else {
    Write-Host "🎯 MANUAL SETUP REQUIRED:" -ForegroundColor Yellow
    Write-Host "   1. Open index.html in two separate browser windows" -ForegroundColor White
    Write-Host "   2. Each window represents a different player" -ForegroundColor White
    Write-Host "   3. One window will be WHITE, the other BLACK" -ForegroundColor White
    Write-Host "   4. Click on pieces in one window to move, watch the other window update!" -ForegroundColor White
    Write-Host "   5. Take turns making moves between the two windows" -ForegroundColor White
}
Write-Host ""
Write-Host "💡 Server is running in background (PID: $($process.Id))" -ForegroundColor Cyan
Write-Host "💡 To stop the server later, run: Stop-Process -Id $($process.Id)" -ForegroundColor Cyan
Write-Host "In your browser two tabs will have opened, each one will be a different colour in the chess game."