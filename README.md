# Pingora Gateway — Universal (Windows/macOS/Linux)

Cross‑platform Pingora reverse proxy with optional TLS.

## Why this version?
- **No TLS by default**: avoids building BoringSSL on Windows.
- Works on **Windows**, **macOS**, and **Linux**.
- Enable TLS only when you want it: `--features tls` (requires BoringSSL toolchain).

## Run (Windows PowerShell)
```powershell
$env:GATEWAY_CONFIG="config/gateway.yaml"
cargo run
```

## Enable TLS (Linux/macOS, or Windows with toolchain installed)
```bash
# Linux/macOS
export GATEWAY_CONFIG=config/gateway.yaml
cargo run --features tls
```

On Windows to build with `--features tls`, install:
- Visual Studio Build Tools + CMake
- **NASM**
- **Perl** (for OpenSSL if any dep builds it)
Then:
```powershell
$env:GATEWAY_CONFIG="config/gateway.yaml"
cargo run --features tls
```

### Alternative: Terminate TLS in front (recommended on Windows)
Put **Caddy/NGINX/Traefik/IIS** in front for TLS, and let Pingora listen HTTP internally.

## Config
Edit `config/gateway.yaml`. TLS fields (`cert`,`key`,`https_listen`) are used only with `--features tls`.

## Features included
- Round‑robin LB with active health checks
- Middleware chain: ACL, API key/JWT (placeholder), rate limit, CORS, security headers
- Prometheus `/metrics` on separate Axum server
- Hot‑reloadable YAML config
