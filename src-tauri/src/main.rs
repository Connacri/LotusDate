// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod battery;
mod chat;
mod matching;
mod p2p;
mod profile;

use battery::BatteryMonitor;
use p2p::P2pNetwork;
use profile::UserProfile;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;
use serde::Serialize;

struct AppState {
    network: Arc<Mutex<P2pNetwork>>,
    profile: Arc<Mutex<UserProfile>>,
    battery: Arc<Mutex<BatteryMonitor>>,
}

#[derive(Clone, Serialize)]
struct NewMessagePayload {
    peer_id: String,
    content: String,
}

#[tokio::main]
async fn main() {
    let battery_monitor = BatteryMonitor::new();
    let profile = UserProfile::load_or_create();
    let network = P2pNetwork::new(profile.peer_id()).await.unwrap();

    let app_state = AppState {
        network: Arc::new(Mutex::new(network)),
        profile: Arc::new(Mutex::new(profile)),
        battery: Arc::new(Mutex::new(battery_monitor)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_profiles,
            commands::send_like,
            commands::open_chat,
            commands::send_message,
            commands::close_chat,
            commands::get_battery_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod commands {
    use super::*;
    use tauri::State;

    #[tauri::command]
    async fn get_profiles(state: State<'_, AppState>) -> Result<Vec<profile::PublicProfile>, String> {
        let network = state.network.lock().await;
        network.discover_profiles().await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    async fn send_like(state: State<'_, AppState>, _peer_id: String) -> Result<bool, String> {
        let mut network = state.network.lock().await;
        network.send_like(&_peer_id).await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    async fn open_chat(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
        let mut network = state.network.lock().await;
        network.open_chat(&peer_id).await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    async fn send_message(app: tauri::AppHandle, state: State<'_, AppState>, peer_id: String, message: String) -> Result<(), String> {
        let mut network = state.network.lock().await;
        network.send_chat_message(&peer_id, &message).await.map_err(|e| e.to_string())?;

        // Mocking an auto-reply for demo purposes
        let reply_msg = format!("Écho: {}", message);
        app.emit("new-message", NewMessagePayload {
            peer_id: peer_id.clone(),
            content: reply_msg,
        }).map_err(|e| e.to_string())?;

        Ok(())
    }

    #[tauri::command]
    async fn close_chat(state: State<'_, AppState>, peer_id: String) -> Result<(), String> {
        let mut network = state.network.lock().await;
        network.close_chat(&peer_id).await;
        Ok(())
    }

    #[tauri::command]
    async fn get_battery_status(state: State<'_, AppState>) -> Result<f32, String> {
        let monitor = state.battery.lock().await;
        Ok(monitor.level())
    }
}
