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
firefox index.html &
sleep 1
firefox index.html &

echo "✅ Setup complete! Two browser windows should open for multiplayer testing."
echo "💡 Server is running in background (PID: $SERVER_PID)"
echo "💡 Server output is logged to: chess-server.log"
echo "💡 To stop the server later, run: kill $SERVER_PID"
