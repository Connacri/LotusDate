<div align="center">

# 🌸 LotusDate

### Application de rencontre P2P 100 % décentralisée
*Zéro serveur. Zéro tiers. Juste le réseau.*

[![Platform](https://img.shields.io/badge/Plateforme-Android-3DDC84?logo=android&logoColor=white)](#-télécharger)
[![Site web](https://img.shields.io/badge/Site%20web-connacri.github.io-FF4D6D?logo=googlechrome&logoColor=white)](https://connacri.github.io/LotusDate/)
[![Built with Tauri](https://img.shields.io/badge/Construit%20avec-Tauri%20v2-FFC131?logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Backend-Rust-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/Statut-Alpha%20%2F%20MVP-orange)](#%EF%B8%8F-statut-du-projet)

Ton téléphone devient un nœud dans un réseau mondial auto-organisé —
aucun compte, aucun serveur, aucune donnée centralisée.

<br>

[![Télécharger l'APK](https://img.shields.io/badge/⬇️_Télécharger_l'APK-3DDC84?style=for-the-badge&logo=android&logoColor=white)](https://github.com/Connacri/LotusDate/releases/latest/download/LotusDate.apk)
[![Voir le site](https://img.shields.io/badge/🌐_Voir_le_site-FF4D6D?style=for-the-badge&logoColor=white)](https://connacri.github.io/LotusDate/)

[Voir toutes les versions →](https://github.com/Connacri/LotusDate/releases)

</div>

<br>

## ✨ Pourquoi LotusDate ?

| | |
|---|---|
| 🔐 **Confidentialité absolue** | Clé Ed25519 générée localement — aucun compte, aucune donnée centrale |
| 💬 **Chats éphémères** | Messages chiffrés E2EE, stockés uniquement en RAM, effacés à la fermeture |
| ⚡ **Instantané** | Swipe-to-chat en moins de 300 ms grâce à WebRTC pré-connecté |
| 🔋 **Respect de la batterie** | Nœud actif uniquement au-dessus de 30 % (ou en charge) |
| 🌍 **Online & Offline** | Fonctionne via Internet *et* Wi-Fi Direct |
| 🧠 **DHT collectif** | Découverte des profils et likes via Kademlia + geohash |

<br>

## 🛠️ Stack technique

![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri%20v2-FFC131?logo=tauri&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)
![React](https://img.shields.io/badge/React-61DAFB?logo=react&logoColor=black)
![Vite](https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white)
![libp2p](https://img.shields.io/badge/libp2p-6E40C9)

| Couche | Technologies |
|---|---|
| **Frontend** | TypeScript · Vite · React |
| **Backend** | Rust · rust-libp2p (Kademlia · Gossipsub · Noise · Yamux) |
| **Multiplateforme** | Tauri v2 |

<br>

## 🚀 Démarrage rapide

```bash
git clone https://github.com/Connacri/LotusDate.git
cd LotusDate
npm install
```

| Cible | Commande |
|---|---|
| 🖥️ Desktop *(dev)* | `npm run tauri dev` |
| 📱 Android *(build)* | `npm run tauri android build` |

<br>

## ⚠️ Statut du projet

LotusDate est en **développement actif (alpha / MVP)**. L'architecture P2P fonctionne,
mais certaines briques de sécurité (échange de clés de session, par exemple) sont encore
des implémentations de démonstration appelées à être renforcées avant tout usage en
production. Les retours, tests et contributions sont les bienvenus via les
[issues GitHub](https://github.com/Connacri/LotusDate/issues).

<br>

<div align="center">

*Construit avec 🌸 par [Connacri](https://github.com/Connacri)*

</div>