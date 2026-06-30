//! Point d'entrée Tauri (lib.rs utilisé pour la compilation Tauri)
//!
//! Architecture des modules :
//!   lib.rs      — bootstrap Tauri, AppState, commandes Tauri
//!   p2p.rs      — réseau libp2p réel (swarm, gossipsub, kademlia)
//!   chat.rs     — chiffrement E2EE (XChaCha20-Poly1305, pure Rust)
//!   matching.rs — logique de match (likes persistés)
//!   profile.rs  — profil utilisateur (keypair Ed25519 persistée)
//!   battery.rs  — niveau de batterie
//!
//! CORRECTIF (crash/ANR Android) :
//! Le démarrage du réseau P2P (DNS + connexions TCP vers les bootstrap nodes)
//! ne doit JAMAIS être attendu de façon synchrone (`block_on`) dans `setup()`,
//! car ce callback s'exécute sur le thread principal (UI) de l'Activity Android.
//! Le bloquer en attendant le réseau déclenche un ANR (Application Not
//! Responding) et Android tue le process — ce qui se manifeste par l'app qui
//! se fige sur l'écran de chargement puis disparaît/se ferme toute seule.
//!
//! Le réseau est donc maintenant démarré en arrière-plan via
//! `tauri::async_runtime::spawn` (non bloquant). `setup()` retourne
//! immédiatement, l'UI s'affiche tout de suite, et le P2pHandle est rangé
//! dans un `Arc<Mutex<Option<P2pHandle>>>` rempli dès que le réseau est prêt.
//! Le frontend est notifié via l'événement `network-ready` (ou `network-error`
//! en cas d'échec — sans paniquer/crasher l'app).

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

// ─── Message d'attente partagé avec le frontend ──────────────────────────────
// Le frontend (App.tsx) affiche déjà ce message dans son `catch` de
// `fetchProfiles()` — on réutilise exactement le même texte pour rester
// cohérent pendant que le réseau s'initialise en arrière-plan.
const NETWORK_NOT_READY: &str = "Réseau P2P en cours d'initialisation…";

// ─── AppState ─────────────────────────────────────────────────────────────────

struct AppState {
    /// `None` tant que `start_network()` n'a pas terminé en arrière-plan.
    handle: Arc<Mutex<Option<P2pHandle>>>,
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

#[derive(Clone, Serialize)]
struct NetworkErrorPayload {
    message: String,
}

// ─── Pont événements réseau → Tauri emit ─────────────────────────────────────

fn spawn_event_bridge(
    app_handle: tauri::AppHandle,
    mut event_rx: mpsc::UnboundedReceiver<SwarmEvent2UI>,
) {
    tauri::async_runtime::spawn(async move {
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

// ─── Démarrage réseau en arrière-plan (NE bloque PAS le thread principal) ────

fn spawn_network_startup(
    app_handle: tauri::AppHandle,
    handle_slot: Arc<Mutex<Option<P2pHandle>>>,
    profile: UserProfile,
) {
    tauri::async_runtime::spawn(async move {
        let (event_tx, event_rx) = mpsc::unbounded_channel::<SwarmEvent2UI>();

        match p2p::start_network(&profile, event_tx).await {
            Ok(handle) => {
                // Publie notre profil dès que le réseau est opérationnel
                let pub_profile = profile.public_version();
                handle
                    .cmd_tx
                    .send(p2p::SwarmCommand::PublishProfile(pub_profile))
                    .ok();

                spawn_event_bridge(app_handle.clone(), event_rx);

                *handle_slot.lock().await = Some(handle);

                // Notifie le frontend que le réseau est prêt (il peut relancer
                // get_profiles / get_my_profile sans rester bloqué sur le spinner)
                let _ = app_handle.emit("network-ready", ());
                tracing::info!("Réseau P2P démarré avec succès");
            }
            Err(e) => {
                // CORRECTIF : on n'utilise plus `.expect()` ici — une erreur réseau
                // (DNS indisponible sur Android, pas de connectivité, etc.) ne doit
                // jamais faire planter tout le process. On informe juste l'UI.
                tracing::error!("Échec démarrage réseau P2P : {}", e);
                let _ = app_handle.emit(
                    "network-error",
                    NetworkErrorPayload { message: e.to_string() },
                );
            }
        }
    });
}

// ─── Point d'entrée Tauri ─────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    tracing_subscriber::EnvFilter::new("warn")
                }),
        )
        .with_target(false)   // retire le chemin de module du log
        .with_thread_ids(false)
        .compact()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let profile = UserProfile::load_or_create();
            let battery = BatteryMonitor::new();

            let handle_slot: Arc<Mutex<Option<P2pHandle>>> = Arc::new(Mutex::new(None));

            // ── État géré IMMÉDIATEMENT, sans attendre le réseau ──────────────
            // L'UI (WebView) peut donc s'afficher tout de suite ; les commandes
            // qui ont besoin du réseau renverront une erreur explicite tant que
            // `handle_slot` est `None`, que le frontend gère déjà gracieusement.
            app.manage(AppState {
                handle: handle_slot.clone(),
                profile: Arc::new(Mutex::new(profile.clone())),
                battery: Arc::new(Mutex::new(battery)),
            });

            // ── Démarrage du réseau P2P EN ARRIÈRE-PLAN ───────────────────────
            // Plus aucun `block_on` ici : c'était la cause du blocage du thread
            // UI Android (ANR) → crash / fermeture automatique de l'app.
            let app_handle = app.handle().clone();
            spawn_network_startup(app_handle, handle_slot, profile);

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

    /// Récupère le handle réseau ou renvoie une erreur explicite (au lieu de
    /// paniquer / bloquer) tant que `start_network()` n'a pas terminé.
    async fn require_handle(state: &State<'_, AppState>) -> Result<P2pHandle, String> {
        state
            .handle
            .lock()
            .await
            .clone()
            .ok_or_else(|| NETWORK_NOT_READY.to_string())
    }

    #[tauri::command]
    pub async fn get_profiles(
        state: State<'_, AppState>,
    ) -> Result<Vec<profile::PublicProfile>, String> {
        let handle = require_handle(&state).await?;
        Ok(handle.get_discovered_profiles().await)
    }

    #[tauri::command]
    pub async fn send_like(
        state: State<'_, AppState>,
        peer_id: String,
    ) -> Result<bool, String> {
        let handle = require_handle(&state).await?;
        Ok(handle.send_like(&peer_id).await)
    }

    #[tauri::command]
    pub async fn open_chat(
        state: State<'_, AppState>,
        peer_id: String,
    ) -> Result<(), String> {
        let handle = require_handle(&state).await?;
        let my_id = {
            let p = state.profile.lock().await;
            p.peer_id().to_string()
        };
        handle.open_chat(&peer_id, &my_id).await;
        Ok(())
    }

    #[tauri::command]
    pub async fn send_message(
        state: State<'_, AppState>,
        peer_id: String,
        content: String,
    ) -> Result<(), String> {
        let handle = require_handle(&state).await?;
        let my_id = {
            let p = state.profile.lock().await;
            p.peer_id().to_string()
        };
        handle.send_chat_message(&peer_id, &content, &my_id).await
    }

    #[tauri::command]
    pub async fn close_chat(
        state: State<'_, AppState>,
        peer_id: String,
    ) -> Result<(), String> {
        let handle = require_handle(&state).await?;
        handle.close_chat(&peer_id).await;
        Ok(())
    }

    // ── Ces deux commandes ne dépendent PAS du réseau : elles restent
    //    disponibles immédiatement, dès l'affichage de l'UI. ──────────────────

    #[tauri::command]
    pub async fn get_battery_status(
        state: State<'_, AppState>,
    ) -> Result<f32, String> {
        Ok(state.battery.lock().await.level())
    }

    #[tauri::command]
    pub async fn get_my_profile(
        state: State<'_, AppState>,
    ) -> Result<profile::PublicProfile, String> {
        Ok(state.profile.lock().await.public_version())
    }
}