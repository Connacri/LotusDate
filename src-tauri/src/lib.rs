//! Point d'entrée Tauri (lib.rs utilisé pour la compilation Tauri)
//!
//! Architecture des modules :
//!   lib.rs      — bootstrap Tauri, AppState, commandes Tauri
//!   p2p.rs      — réseau libp2p réel (swarm, gossipsub, kademlia)
//!   chat.rs     — chiffrement E2EE (XChaCha20-Poly1305)
//!   matching.rs — logique de match (likes persistés)
//!   profile.rs  — profil utilisateur (keypair Ed25519 persistée)
//!   battery.rs  — niveau de batterie

mod battery;
mod chat;
mod matching;
mod p2p;
mod profile;

use battery::BatteryMonitor;
use p2p::{P2pHandle, SwarmEvent2UI};
use profile::UserProfile;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::{mpsc, Mutex};
use serde::Serialize;

// ─── AppState ─────────────────────────────────────────────────────────────────

struct AppState {
    handle: P2pHandle,
    profile: Arc<Mutex<UserProfile>>,
    battery: Arc<Mutex<BatteryMonitor>>,
}

// ─── Payloads émis vers le frontend ──────────────────────────────────────────

#[derive(Clone, Serialize)]
struct NewMessagePayload {
    peer_id: String,
    content: String,
}

#[derive(Clone, Serialize)]
struct MatchPayload {
    peer_id: String,
}

#[derive(Clone, Serialize)]
struct LikePayload {
    from_peer: String,
}

// ─── Pont événements réseau → Tauri emit ─────────────────────────────────────

fn spawn_event_bridge(
    app_handle: tauri::AppHandle,
    mut event_rx: mpsc::UnboundedReceiver<SwarmEvent2UI>,
) {
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                SwarmEvent2UI::ChatMessage { from_peer, content } => {
                    let _ = app_handle.emit(
                        "new-message",
                        NewMessagePayload { peer_id: from_peer, content },
                    );
                }
                SwarmEvent2UI::MatchConfirmed { peer_id } => {
                    let _ = app_handle.emit("match-confirmed", MatchPayload { peer_id });
                }
                SwarmEvent2UI::LikeReceived { from_peer } => {
                    let _ = app_handle.emit("like-received", LikePayload { from_peer });
                }
                SwarmEvent2UI::NewProfile(_) => {
                    // Le frontend actualise les profils toutes les 5s via polling ;
                    // on peut aussi émettre un signal "profiles-updated" ici.
                    let _ = app_handle.emit("profiles-updated", ());
                }
                SwarmEvent2UI::PeerConnected { peer_id } => {
                    tracing::debug!("Pair connecté : {}", peer_id);
                }
                SwarmEvent2UI::PeerDisconnected { peer_id } => {
                    tracing::debug!("Pair déconnecté : {}", peer_id);
                }
            }
        }
    });
}

// ─── Point d'entrée Tauri ─────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let profile = UserProfile::load_or_create();
            let battery = BatteryMonitor::new();

            // Canal de remontée d'événements réseau vers l'UI
            let (event_tx, event_rx) = mpsc::unbounded_channel::<SwarmEvent2UI>();

            // Démarre le réseau P2P réel dans un runtime tokio existant
            let handle = tauri::async_runtime::block_on(async {
                p2p::start_network(&profile, event_tx)
                    .await
                    .expect("Impossible de démarrer le réseau P2P")
            });

            // Publie notre profil immédiatement au démarrage
            let pub_profile = profile.public_version();
            handle.cmd_tx.send(
                p2p::SwarmCommand::PublishProfile(pub_profile)
            ).ok();

            let app_handle = app.handle().clone();
            spawn_event_bridge(app_handle, event_rx);

            app.manage(AppState {
                handle,
                profile: Arc::new(Mutex::new(profile)),
                battery: Arc::new(Mutex::new(battery)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_profiles,
            commands::send_like,
            commands::open_chat,
            commands::send_message,
            commands::close_chat,
            commands::get_battery_status,
            commands::get_my_profile,
        ])
        .run(tauri::generate_context!())
        .expect("Erreur démarrage Tauri");
}

// ─── Commandes Tauri invocables depuis le frontend ───────────────────────────

mod commands {
    use super::*;
    use tauri::State;

    /// Retourne la liste des profils découverts sur le réseau P2P.
    /// Liste vide au démarrage → se remplit au fur et à mesure des pairs actifs.
    #[tauri::command]
    pub async fn get_profiles(
        state: State<'_, AppState>,
    ) -> Result<Vec<profile::PublicProfile>, String> {
        Ok(state.handle.get_discovered_profiles().await)
    }

    /// Envoie un like sur le réseau. Retourne `true` si c'est un match immédiat.
    #[tauri::command]
    pub async fn send_like(
        state: State<'_, AppState>,
        peer_id: String,
    ) -> Result<bool, String> {
        Ok(state.handle.send_like(&peer_id).await)
    }

    /// Ouvre une session de chat E2EE avec le peer.
    #[tauri::command]
    pub async fn open_chat(
        state: State<'_, AppState>,
        peer_id: String,
    ) -> Result<(), String> {
        let my_id = {
            let p = state.profile.lock().await;
            p.peer_id().to_string()
        };
        state.handle.open_chat(&peer_id, &my_id).await;
        Ok(())
    }

    /// Envoie un message chiffré E2EE au peer via Gossipsub.
    #[tauri::command]
    pub async fn send_message(
        state: State<'_, AppState>,
        peer_id: String,
        content: String,
    ) -> Result<(), String> {
        let my_id = {
            let p = state.profile.lock().await;
            p.peer_id().to_string()
        };
        state
            .handle
            .send_chat_message(&peer_id, &content, &my_id)
            .await
    }

    /// Ferme et efface la session de chat avec le peer.
    #[tauri::command]
    pub async fn close_chat(
        state: State<'_, AppState>,
        peer_id: String,
    ) -> Result<(), String> {
        state.handle.close_chat(&peer_id).await;
        Ok(())
    }

    /// Niveau de batterie (0.0 → 1.0).
    #[tauri::command]
    pub async fn get_battery_status(
        state: State<'_, AppState>,
    ) -> Result<f32, String> {
        Ok(state.battery.lock().await.level())
    }

    /// Retourne le profil public de l'utilisateur local.
    #[tauri::command]
    pub async fn get_my_profile(
        state: State<'_, AppState>,
    ) -> Result<profile::PublicProfile, String> {
        Ok(state.profile.lock().await.public_version())
    }
}