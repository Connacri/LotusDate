# 🌸 LotusDate

**Application de rencontre P2P 100% décentralisée**  
*zéro serveur, zéro tiers, juste le réseau*

Ton téléphone devient un nœud dans un réseau mondial auto-organisé.

## 📥 Télécharger

[![Télécharger l'APK](https://img.shields.io/badge/Android-T%C3%A9l%C3%A9charger%20l'APK-3DDC84?logo=android&logoColor=white)](https://github.com/Connacri/LotusDate/releases/latest/download/LotusDate.apk)

➡️ [Toutes les versions](https://github.com/Connacri/LotusDate/releases)

## ✨ Pourquoi LotusDate ?

- 🔐 **Confidentialité absolue** — Clé Ed25519 locale, aucun compte ni donnée centrale
- 💬 **Chats éphémères** — Double Ratchet E2EE, stockés uniquement en RAM (effacés à la fermeture)
- ⚡ **Instantané** — Swipe-to-chat en <300 ms grâce à WebRTC pré-connecté
- 🔋 **Respect batterie** — Nœud actif seulement >30% (ou en charge)
- 🌍 **Online & Offline** — Internet + Wi-Fi Direct
- 🧠 **DHT collectif** — Profils et likes via Kademlia + geohash

## 🛠️ Stack Technique

- **Frontend** : TypeScript, Vite, React
- **Backend** : Rust, rust-libp2p, libsodium
- **Multiplateforme** : Tauri v2

## 🚀 Installation rapide

```bash
git clone https://github.com/Connacri/LotusDate.git
cd LotusDate
npm install
npm run tauri dev          # Desktop
# ou
npm run tauri android build # APK