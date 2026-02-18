<div align="center">

  <h1>Rust Chess App</h1>
  <p><em>A real-time online chess game with a Rust backend and a JavaScript frontend, allowing players to be automatically matched and play against each other in the browser.</em></p>
  

</div>

---

**Rusty Chess** is a *blazingly fast* chess game server that ensures smooth gameplay, real-time matchmaking, and a seamless experience for players across devices.

---

### Screenshots

<p align="center">
  <img src="chess.png" alt="Chess Game Screenshot" width="45%">
  <img src="engine.png" alt="Chess Engine Screenshot" width="45%">
</p>

---

### Features

- **Real-time Multiplayer** - Play against opponents from around the world with instant move updates
- **Automatic Matchmaking** - Get paired with another player automatically when you join
- **Complete Chess Rules** - Full implementation including pawn promotion, check, checkmate, stalemate, and draw conditions
- **Game Codes** - Join specific games using unique game codes
- **Role Assignment** - Automatic white/black role assignment for players
- **Responsive Design** - Works seamlessly across desktop and mobile devices
- **Blazingly Fast** - Built with Rust for optimal performance and low latency
- **WebSocket Communication** - Real-time bidirectional communication between players
- **Rematch Support** - Start a new game instantly after finishing

---

### Quick Start

#### Install Scripts

**Linux:**
```bash
curl -L https://raw.githubusercontent.com/Oliver-Form/chess/refs/heads/master/scripts/run-linux.sh -o run-linux.sh && chmod +x run-linux.sh && ./run-linux.sh
```

**Windows (PowerShell):**
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Oliver-Form/chess/refs/heads/master/scripts/run-windows.ps1" -OutFile "run-windows.ps1"; .\run-windows.ps1
```

**macOS:**
```bash
curl -L https://raw.githubusercontent.com/Oliver-Form/chess/refs/heads/master/scripts/run-macos.sh -o run-macos.sh && chmod +x run-macos.sh && ./run-macos.sh
```

These scripts will automatically download the correct binary for your platform, start the server, clone the repo for static assets, and open two browser windows for local multiplayer testing!

