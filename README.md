# 🌸 LotusDate

**100% Decentralized P2P Dating App – No Servers, No Third Parties**

LotusDate transforme ton smartphone en nœud d’un réseau peer-to-peer mondial auto-organisé.

## ✨ Fonctionnalités

- 🔐 **Vie privée totale** : Identité Ed25519 locale, zéro compte, zéro email/téléphone.
- 💬 **Chat éphémère** : Double Ratchet E2EE, messages en RAM uniquement (effacés à la fermeture).
- ⚡ **Ultra-rapide** : Préconnexions WebRTC (<300ms), profils en cache DHT.
- 🔋 **Économie batterie** : Nœud complet seulement si batterie >30% (sinon client léger).
- 🌐 **Online/Offline** : Internet + Wi-Fi Direct.
- 🧠 **DHT Kademlia** : Profils et likes distribués via geohash.

## 🚀 Installation

1. Clone : `git clone https://github.com/Connacri/LotusDate.git`
2. `cd LotusDate && npm install`
3. `npm run tauri dev` (desktop) ou `npm run tauri android build` (APK).

## 🛠️ Tech Stack

- **Frontend** : TypeScript + Vite + React
- **Backend** : Rust + rust-libp2p + libsodium
- **Desktop/Mobile** : Tauri v2

## 📜 License

CC0-1.0 – Domaine public.