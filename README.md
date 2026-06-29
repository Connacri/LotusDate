# 🌸 LotusDate

**100% Decentralized Dating App – Zero Servers, Zero Third Parties, Just the Network**

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Build APK](https://github.com/<user>/LotusDate/actions/workflows/build-apk.yml/badge.svg)](https://github.com/<user>/LotusDate/actions/workflows/build-apk.yml)
[![Rust](https://img.shields.io/badge/Rust-1.80+-orange)](https://www.rust-lang.org)
[![Tauri v2](https://img.shields.io/badge/Tauri-2.0-9cf)](https://tauri.app)

LotusDate transforms your smartphone into a node of a global, self‑organizing peer‑to‑peer network.  
**No servers, no cloud, no accounts, no third‑party services.**  
Your data stays on your device, matches happen directly, and conversations disappear forever when you close them.

---

## ✨ Why LotusDate?

- 🔐 **True privacy** – no email, no phone number, no central database. Your identity is an Ed25519 keypair generated on your device.
- 💬 **Ephemeral chat** – every message is end‑to‑end encrypted (Double Ratchet) and stored **only in RAM**. Closing the chat permanently erases all traces.
- ⚡ **Blazing fast** – pre‑cached profiles and pre‑established WebRTC connections deliver swipe‑to‑chat in **under 300 ms**.
- 🔋 **Battery‑aware networking** – your phone acts as a full node only when battery > 30% (or while charging). Otherwise it runs as a lightweight client.
- 🌐 **Online & offline** – works over the Internet (4G/5G/Wi‑Fi) or locally via Wi‑Fi Direct when no connection is available.
- 🧠 **Collective database** – profiles and pending likes are stored in a Kademlia DHT maintained by the community. No central index, no single point of failure.

---

## 🧭 How It Works

```mermaid
graph TD
    A[You] <-->|Kademlia DHT| B[Profiles nearby]
    A -- Like --> C[Recipient's mailbox]
    C -- Match --> D[WebRTC channel]
    D -- Double Ratchet --> E[Ephemeral Chat]
    F[Community nodes] -->|Store & relay| C
    G[Battery > 30% ?] -->|Yes| F
    G -->|No| H[Light client]