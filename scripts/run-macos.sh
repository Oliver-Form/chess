#!/bin/bash

echo "🚀 Setting up Chess Game for macOS..."

# Detect architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "arm64" ]]; then
    BINARY_URL="https://github.com/Oliver-Form/chess/releases/latest/download/chess-macos-aarch64"
    echo "📱 Detected Apple Silicon Mac"
else
    BINARY_URL="https://github.com/Oliver-Form/chess/releases/latest/download/chess-macos-x86_64"
    echo "💻 Detected Intel Mac"
fi

# Download the macOS binary
echo "📥 Downloading chess server..."
curl -L $BINARY_URL -o chess-server
chmod +x chess-server

# Start the server in background
echo "🔧 Starting chess server..."
nohup ./chess-server > chess-server.log 2>&1 &
SERVER_PID=$!

# Clone the repository for frontend assets
echo "📂 Downloading frontend assets..."
git clone https://github.com/Oliver-Form/chess.git

# Navigate to frontend directory
cd chess/frontend

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 3

# Open two browser windows for multiplayer testing
echo "🎮 Opening chess game in browser..."

# Try browsers in order of preference
if command -v safari &> /dev/null || [ -d "/Applications/Safari.app" ]; then
    echo "Using Safari..."
    open -n -a Safari index.html
    sleep 1
    open -n -a Safari index.html
    BROWSER_NAME="Safari"
elif command -v google-chrome &> /dev/null || [ -d "/Applications/Google Chrome.app" ]; then
    echo "Using Google Chrome..."
    open -n -a "Google Chrome" index.html
    sleep 1
    open -n -a "Google Chrome" index.html
    BROWSER_NAME="Chrome"
elif command -v firefox &> /dev/null || [ -d "/Applications/Firefox.app" ]; then
    echo "Using Firefox..."
    open -n -a Firefox index.html
    sleep 1
    open -n -a Firefox index.html
    BROWSER_NAME="Firefox"
elif command -v opera &> /dev/null || [ -d "/Applications/Opera.app" ]; then
    echo "Using Opera..."
    open -n -a Opera index.html
    sleep 1
    open -n -a Opera index.html
    BROWSER_NAME="Opera"
elif command -v brave &> /dev/null || [ -d "/Applications/Brave Browser.app" ]; then
    echo "Using Brave..."
    open -n -a "Brave Browser" index.html
    sleep 1
    open -n -a "Brave Browser" index.html
    BROWSER_NAME="Brave"
else
    echo "❌ No supported browser found!"
    echo "Please install one of: Safari, Chrome, Firefox, Opera, or Brave"
    echo "Then manually open: index.html in two browser windows"
    BROWSER_NAME="manual"
fi

if [ "$BROWSER_NAME" != "manual" ]; then
    echo "✅ Setup complete! Two browser windows should open for multiplayer testing."
    echo ""
    echo "🎯 HOW TO TEST MULTIPLAYER:"
    echo "   1. Two $BROWSER_NAME windows should have opened automatically"
    echo "   2. Each window represents a different player"
    echo "   3. One window will be WHITE, the other BLACK"
    echo "   4. Click on pieces in one window to move, watch the other window update!"
    echo "   5. Take turns making moves between the two windows"
else
    echo "🎯 MANUAL SETUP REQUIRED:"
    echo "   1. Open index.html in two separate browser windows"
    echo "   2. Each window represents a different player"
    echo "   3. One window will be WHITE, the other BLACK"
    echo "   4. Click on pieces in one window to move, watch the other window update!"
    echo "   5. Take turns making moves between the two windows"
fi
echo ""
echo "💡 Server is running in background (PID: $SERVER_PID)"
echo "💡 Server output is logged to: chess-server.log"
echo "💡 To stop the server later, run: kill $SERVER_PID"
echo "In your browser two tabs will have opened, each one will be a different colour in the chess game."