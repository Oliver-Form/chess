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
echo "🎮 Opening chess game in Safari..."
open -n -a Safari index.html
sleep 1
open -n -a Safari index.html

echo "✅ Setup complete! Two Safari windows should open for multiplayer testing."
echo "💡 Server is running in background (PID: $SERVER_PID)"
echo "💡 Server output is logged to: chess-server.log"
echo "💡 To stop the server later, run: kill $SERVER_PID"
