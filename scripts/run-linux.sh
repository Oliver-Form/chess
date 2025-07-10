#!/bin/bash

echo "🚀 Setting up Chess Game for Linux..."

# Download the Linux binary
echo "📥 Downloading chess server..."
curl -L https://github.com/Oliver-Form/chess/releases/latest/download/chess-linux-x86_64 -o chess-server
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
if command -v firefox &> /dev/null; then
    echo "Using Firefox..."
    firefox index.html &
    sleep 1
    firefox index.html &
    BROWSER_NAME="Firefox"
elif command -v google-chrome &> /dev/null; then
    echo "Using Google Chrome..."
    google-chrome index.html &
    sleep 1
    google-chrome index.html &
    BROWSER_NAME="Chrome"
elif command -v chromium-browser &> /dev/null; then
    echo "Using Chromium..."
    chromium-browser index.html &
    sleep 1
    chromium-browser index.html &
    BROWSER_NAME="Chromium"
elif command -v chromium &> /dev/null; then
    echo "Using Chromium..."
    chromium index.html &
    sleep 1
    chromium index.html &
    BROWSER_NAME="Chromium"
elif command -v opera &> /dev/null; then
    echo "Using Opera..."
    opera index.html &
    sleep 1
    opera index.html &
    BROWSER_NAME="Opera"
elif command -v brave-browser &> /dev/null; then
    echo "Using Brave..."
    brave-browser index.html &
    sleep 1
    brave-browser index.html &
    BROWSER_NAME="Brave"
else
    echo "❌ No supported browser found!"
    echo "Please install one of: Firefox, Chrome, Chromium, Opera, or Brave"
    echo "Then manually open: index.html in two browser tabs"
    BROWSER_NAME="manual"
fi

if [ "$BROWSER_NAME" != "manual" ]; then
    echo "✅ Setup complete! Two browser tabs should open for multiplayer testing."
    echo ""
    echo "🎯 HOW TO TEST MULTIPLAYER:"
    echo "   1. Two $BROWSER_NAME tabs should have opened automatically"
    echo "   2. Each tab represents a different player"
    echo "   3. One tab will be WHITE, the other BLACK"
    echo "   4. Click on pieces in one tab to move, watch the other tab update!"
    echo "   5. Take turns making moves between the two tabs"
else
    echo "🎯 MANUAL SETUP REQUIRED:"
    echo "   1. Open index.html in two separate browser tabs"
    echo "   2. Each tab represents a different player"
    echo "   3. One tab will be WHITE, the other BLACK"
    echo "   4. Click on pieces in one tab to move, watch the other tab update!"
    echo "   5. Take turns making moves between the two tabs"
fi
echo ""
echo "💡 Server is running in background (PID: $SERVER_PID)"
echo "💡 Server output is logged to: chess-server.log"
echo "💡 To stop the server later, run: kill $SERVER_PID"