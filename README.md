# AIO MCP

Tek Platform. Tum MCP.

[![CI](https://github.com/CRTYPUBG/aio-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/CRTYPUBG/aio-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

Bu README Turkce yazildi ve bilerek ASCII karakterlerle tutuldu.
Boylece Windows terminal, GitHub ve farkli editor kodlamalarinda karakter bozulmasi yasanmaz.

## Neden Karakterler Bozuldu?

Sorunun nedeni genelde UTF-8 disinda kodlama ile kaydetme/goruntuleme farkidir.
Ornek bozulma: emoji veya Turkce karakterler bazen anlamsiz sekilde gorunur.

Bu repoda uygulanan cozum:

1. README sade ASCII metne cevrildi.
2. Emoji ve ozel kutu cizimleri kaldirildi.
3. Terminaller arasi gorunum farkini azaltmak icin sade markdown kullanildi.

## Proje Ozeti

AIO MCP, Model Context Protocol ekosistemi icin birlestirilmis bir kontrol ve calistirma platformudur.

Temel kabiliyetler:

- Plugin yonetimi
- API gateway
- Yetki yonetimi
- Konfigurasyon yonetimi
- Railway uyumlu HTTP server

## Mimari (ASCII Diyagram)

```text
+---------------------------------------------------------+
|                 Experience Layer                        |
|  Desktop App (Tauri) | Web Dashboard | CLI             |
+---------------------------------------------------------+
|                     API Layer                           |
|                REST /v1/*  |  WebSocket                 |
+---------------------------------------------------------+
|               Core Services (Rust)                      |
| Engine | Plugin Manager | Config | Permission | Gateway |
+---------------------------------------------------------+
|                     Data Layer                          |
|                SQLite / PostgreSQL / Redis              |
+---------------------------------------------------------+
```

## Railway Deployment

AIO MCP saf HTTP API olarak calisir. Frontend zorunlu degildir.

### 1) Environment degiskenleri

Railway Variables alanina asagidaki degerleri ekleyin:

| Variable       | Ornek                | Aciklama                                |
| -------------- | -------------------- | --------------------------------------- |
| `AIO_API_KEYS` | `sk-your-secret-key` | Birden fazla key icin virgul ile ayirin |
| `PORT`         | auto                 | Railway otomatik verir                  |
| `RUST_LOG`     | `aio_server=info`    | Opsiyonel                               |

Guclu key uretmek icin:

```bash
openssl rand -hex 32
```

### 2) Deploy

```bash
railway login
railway link
railway up
```

## API Referansi

`/v1/*` endpointleri API key ister.

Authentication:

```text
X-Api-Key: sk-your-key
# veya
Authorization: Bearer sk-your-key
```

Public endpointler:

| Method | Path      | Aciklama        |
| ------ | --------- | --------------- |
| `GET`  | `/`       | Servis bilgisi  |
| `GET`  | `/health` | Saglik kontrolu |

Korunan endpointler:

| Method | Path                      | Aciklama       |
| ------ | ------------------------- | -------------- |
| `GET`  | `/v1/plugins`             | Plugin listesi |
| `POST` | `/v1/plugins`             | Plugin kaydi   |
| `GET`  | `/v1/config/:scope/:key`  | Config oku     |
| `PUT`  | `/v1/config/:scope/:key`  | Config yaz     |
| `POST` | `/v1/permissions/request` | Izin talebi    |
| `POST` | `/v1/permissions/grant`   | Izin onayi     |
| `GET`  | `/v1/permissions/check`   | Izin kontrolu  |
| `GET`  | `/v1/services`            | Core servisler |
| `GET`  | `/v1/routes`              | Route tablosu  |

Ornek:

```bash
curl -X POST https://your-app.up.railway.app/v1/plugins \
  -H "X-Api-Key: sk-your-key" \
  -H "Content-Type: application/json" \
  -d '{"id":"official.github","version":"1.0.0"}'
```

## Lokal Gelistirme

Gereksinimler:

- Rust stable
- Node.js 22+ (opsiyonel, TypeScript app shell icin)

Calistirma:

```bash
git clone https://github.com/CRTYPUBG/aio-mcp.git
cd aio-mcp

cp .env.example .env
# .env icinde AIO_API_KEYS ayarla

cargo run --package aio-server
```

Test:

```bash
cargo test --workspace
```

Build (dist cikisi):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1
```

## Proje Yapisi

```text
aio-mcp/
|- server/
|- core/
|  |- engine/
|  |- plugin-manager/
|  |- configuration-manager/
|  |- permission-manager/
|  \- api-gateway/
|- apps/
|  |- desktop/
|  |- web-dashboard/
|  \- cli/
|- docs/
|- schemas/
|- scripts/
|- assets/
|- Dockerfile
\- railway.json
```

## Lisans

MIT - detaylar icin [LICENSE](LICENSE).

<div align="center">

<!-- Theme-adaptive banner -->
<picture>
  <source media="(prefers-color-scheme: dark)"  srcset="assets/banner-dark.jpg">
  <source media="(prefers-color-scheme: light)" srcset="assets/banner-ligth.jpg">
  <img alt="AIO MCP Banner" src="assets/banner.jpg" width="100%">
</picture>

<br/>
<br/>

<img src="assets/logo.png" alt="AIO MCP Logo" width="96">

# AIO MCP

### One Platform. Every MCP.

**The world's most advanced open-source platform for managing every Model Context Protocol server, plugin, AI provider, workflow, marketplace, registry, automation, and enterprise deployment.**

<br/>

[![CI](https://github.com/CRTYPUBG/aio-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/CRTYPUBG/aio-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Railway](https://img.shields.io/badge/deploy-Railway-blueviolet?logo=railway)](https://railway.app)

<br/>

[ğŸš€ Deploy to Railway](#railway-deployment) Â· [ğŸ“– Architecture](docs/architecture/phase-1.md) Â· [ğŸ”‘ API Reference](#api-reference) Â· [ğŸ›  Local Dev](#local-development)

</div>

---

## What is AIO MCP?

AIO MCP is a unified control plane and runtime for the full [Model Context Protocol](https://modelcontextprotocol.io) ecosystem.

Think of it as **Docker Desktop + VS Code Marketplace + npm** â€” built exclusively for MCP.

# AIO MCP

Tek Platform. Tum MCP.

[![CI](https://github.com/CRTYPUBG/aio-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/CRTYPUBG/aio-mcp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

Bu README Turkce yazildi ve bilerek ASCII karakterlerle tutuldu.
Boylece Windows terminal, GitHub ve farkli editor kodlamalarinda karakter bozulmasi yasanmaz.

## Neden Karakterler Bozuldu?

Sorunun nedeni genelde UTF-8 disinda kodlama ile kaydetme/goruntuleme farkidir.
Ornek bozulma: `🚀` gibi emoji veya Turkce karakterler bazen `ğŸš€` gibi gorunur.

Bu repoda cozum olarak:

1. README sade ASCII metne cevrildi.
2. Emoji ve ozel kutu cizimleri kaldirildi.
3. Terminaller arasi gorunum farkini azaltmak icin sade markdown kullanildi.

## Proje Ozeti

AIO MCP, Model Context Protocol ekosistemi icin birlestirilmis bir kontrol ve calistirma platformudur.

Temel kabiliyetler:

- Plugin yonetimi
- API gateway
- Yetki yonetimi
- Konfigurasyon yonetimi
- Railway uyumlu HTTP server

## Mimari (ASCII Diyagram)

```text
+---------------------------------------------------------+
|                 Experience Layer                        |
|  Desktop App (Tauri) | Web Dashboard | CLI             |
+---------------------------------------------------------+
|                     API Layer                           |
|                REST /v1/*  |  WebSocket                 |
+---------------------------------------------------------+
|               Core Services (Rust)                      |
| Engine | Plugin Manager | Config | Permission | Gateway |
+---------------------------------------------------------+
|                     Data Layer                          |
|                SQLite / PostgreSQL / Redis              |
+---------------------------------------------------------+
```

## Railway Deployment

AIO MCP saf HTTP API olarak calisir. Frontend zorunlu degildir.

### 1) Environment degiskenleri

Railway Variables alanina asagidaki degerleri ekleyin:

| Variable       | Ornek                | Aciklama                                |
| -------------- | -------------------- | --------------------------------------- |
| `AIO_API_KEYS` | `sk-your-secret-key` | Birden fazla key icin virgul ile ayirin |
| `PORT`         | auto                 | Railway otomatik verir                  |
| `RUST_LOG`     | `aio_server=info`    | Opsiyonel                               |

Guclu key uretmek icin:

```bash
openssl rand -hex 32
```

### 2) Deploy

```bash
railway login
railway link
railway up
```

## API Referansi

`/v1/*` endpointleri API key ister.

Authentication:

```text
X-Api-Key: sk-your-key
# veya
Authorization: Bearer sk-your-key
```

Public endpointler:

| Method | Path      | Aciklama        |
| ------ | --------- | --------------- |
| `GET`  | `/`       | Servis bilgisi  |
| `GET`  | `/health` | Saglik kontrolu |

Korunan endpointler:

| Method | Path                      | Aciklama       |
| ------ | ------------------------- | -------------- |
| `GET`  | `/v1/plugins`             | Plugin listesi |
| `POST` | `/v1/plugins`             | Plugin kaydi   |
| `GET`  | `/v1/config/:scope/:key`  | Config oku     |
| `PUT`  | `/v1/config/:scope/:key`  | Config yaz     |
| `POST` | `/v1/permissions/request` | Izin talebi    |
| `POST` | `/v1/permissions/grant`   | Izin onayi     |
| `GET`  | `/v1/permissions/check`   | Izin kontrolu  |
| `GET`  | `/v1/services`            | Core servisler |
| `GET`  | `/v1/routes`              | Route tablosu  |

Ornek:

```bash
curl -X POST https://your-app.up.railway.app/v1/plugins \
  -H "X-Api-Key: sk-your-key" \
  -H "Content-Type: application/json" \
  -d '{"id":"official.github","version":"1.0.0"}'
```

## Lokal Gelistirme

Gereksinimler:

- Rust stable
- Node.js 22+ (opsiyonel, TypeScript app shell icin)

Calistirma:

```bash
git clone https://github.com/CRTYPUBG/aio-mcp.git
cd aio-mcp

cp .env.example .env
# .env icinde AIO_API_KEYS ayarla

cargo run --package aio-server
```

Test:

```bash
cargo test --workspace
```

Build (dist cikisi):

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build.ps1
```

## Proje Yapisi

```text
aio-mcp/
|- server/
|- core/
|  |- engine/
|  |- plugin-manager/
|  |- configuration-manager/
|  |- permission-manager/
|  \- api-gateway/
|- apps/
|  |- desktop/
|  |- web-dashboard/
|  \- cli/
|- docs/
|- schemas/
|- scripts/
|- assets/
|- Dockerfile
\- railway.json
```

## Lisans

MIT - detaylar icin [LICENSE](LICENSE).
