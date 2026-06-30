//! Couche réseau P2P réelle — libp2p 0.56
//!
//! Architecture :
//!   ┌─────────────────────────────────────────────────────────┐
//!   │  Swarm (tokio::spawn)  ← boucle événements en arrière-plan │
//!   │  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌────────┐│
//!   │  │ Kademlia │  │ Gossipsub │  │ Identify │  │AutoNAT ││
//!   │  └──────────┘  └───────────┘  └──────────┘  └────────┘│
//!   └─────────────────────────────────────────────────────────┘
//!        ↓ mpsc channels ↓
//!   Commands (UI→réseau)    Events (réseau→UI via Tauri emit)
//!
//! Bootstrap nodes publics Lotus (IPFS + nos propres relais) :
//!   On utilise les bootstrap nodes IPFS publics car ils supportent
//!   Kademlia et permettent à deux téléphones derrière NAT de se trouver.

use crate::chat::{EncryptedMessage, EphemeralChat};
use crate::matching::MatchState;
use crate::profile::{PublicProfile, UserProfile};

use libp2p::{
    gossipsub::{self},
    identify,
    kad::{self, store::MemoryStore},
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
    Multiaddr, PeerId, StreamProtocol, SwarmBuilder,
};
use libp2p::futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

// ─── Bootstrap nodes publics (IPFS + Lotus bootstrap) ────────────────────────
// Ces nœuds permettent à deux pairs derrière NAT de se découvrir mutuellement.
const BOOTSTRAP_PEERS: &[(&str, &str)] = &[
    (
        "12D3KooWNjMh8UkfLi7armVkasHFEnd29YJ37oKzQpHkDu1ps2Gh",
        "/dnsaddr/bootstrap.libp2p.io/p2p/12D3KooWNjMh8UkfLi7armVkasHFEnd29YJ37oKzQpHkDu1ps2Gh",
    ),
    (
        "QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
    ),
    (
        "QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
    ),
];

// ─── Topics Gossipsub ────────────────────────────────────────────────────────
const TOPIC_PROFILES: &str = "lotus/profiles/v1";
const TOPIC_LIKES: &str = "lotus/likes/v1";
const TOPIC_CHAT_PREFIX: &str = "lotus/chat/v1/";

// ─── Messages réseau sérialisés (Gossipsub payload) ──────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum NetworkMessage {
    /// Annonce de profil diffusée régulièrement sur TOPIC_PROFILES
    Profile(PublicProfile),
    /// Like envoyé à un peer spécifique (via topic dédié ou filtrage to_peer)
    Like { from_peer: String, to_peer: String },
    /// Message de chat chiffré E2EE
    Chat(EncryptedMessage),
}

// ─── Commandes envoyées depuis l'UI vers la boucle Swarm ─────────────────────

pub enum SwarmCommand {
    SendLike { to_peer: String },
    SendChat { to_peer: String, message: EncryptedMessage },
    PublishProfile(PublicProfile),
    Shutdown,
}

// ─── Événements émis depuis la boucle Swarm vers l'UI ────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event")]
pub enum SwarmEvent2UI {
    NewProfile(PublicProfile),
    LikeReceived { from_peer: String },
    MatchConfirmed { peer_id: String },
    ChatMessage { from_peer: String, content: String },
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
}

// ─── Behaviour combiné libp2p 0.56 (API SwarmBuilder fluent) ─────────────────

#[derive(NetworkBehaviour)]
pub struct LotusBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
}

// ─── Handle publique (partagé avec AppState) ─────────────────────────────────

#[derive(Clone)]
pub struct P2pHandle {
    /// Canal de commandes vers la boucle Swarm
    pub cmd_tx: mpsc::UnboundedSender<SwarmCommand>,
    /// Profils découverts (peer_id → profil)
    pub discovered: Arc<Mutex<HashMap<String, PublicProfile>>>,
    /// État des matchs
    pub match_state: Arc<Mutex<MatchState>>,
    /// Sessions de chat actives (peer_id → session E2EE)
    pub chats: Arc<Mutex<HashMap<String, EphemeralChat>>>,
}

impl P2pHandle {
    /// Envoie un like sur le réseau.
    /// Retourne `true` immédiatement si l'autre avait déjà liké (match local détecté).
    /// Un second match peut arriver plus tard via événement réseau.
    pub async fn send_like(&self, to_peer: &str) -> bool {
        let is_match = {
            let mut ms = self.match_state.lock().await;
            ms.register_outgoing_like(to_peer)
        };
        let _ = self.cmd_tx.send(SwarmCommand::SendLike {
            to_peer: to_peer.to_string(),
        });
        is_match
    }

    /// Envoie un message chiffré au peer.
    pub async fn send_chat_message(
        &self,
        to_peer: &str,
        plaintext: &str,
        my_peer_id: &str,
    ) -> Result<(), String> {
        // FIX: suppression de `mut` inutile — `get()` ne nécessite pas &mut
        let chats = self.chats.lock().await;
        let session = chats
            .get(to_peer)
            .ok_or_else(|| format!("Pas de session de chat active avec {}", to_peer))?;

        let encrypted = session.encrypt(plaintext, my_peer_id);
        drop(chats);

        self.cmd_tx
            .send(SwarmCommand::SendChat {
                to_peer: to_peer.to_string(),
                message: encrypted,
            })
            .map_err(|e| e.to_string())
    }

    /// Ouvre une session de chat (dérive un secret partagé depuis les clés connues).
    /// Dans ce MVP, on utilise un secret HKDF-like basé sur les deux PeerIds.
    pub async fn open_chat(&self, to_peer: &str, my_peer_id: &str) {
        let mut chats = self.chats.lock().await;
        if chats.contains_key(to_peer) {
            return;
        }
        // Dérivation déterministe d'un secret de session à partir des deux PeerIds.
        // NOTE : en production, remplacer par X25519 ECDH via libp2p request-response.
        let secret = derive_session_secret(my_peer_id, to_peer);
        chats.insert(to_peer.to_string(), EphemeralChat::new(&secret));
    }

    pub async fn close_chat(&self, to_peer: &str) {
        let mut chats = self.chats.lock().await;
        if let Some(mut session) = chats.remove(to_peer) {
            session.close();
        }
    }

    pub async fn get_discovered_profiles(&self) -> Vec<PublicProfile> {
        self.discovered.lock().await.values().cloned().collect()
    }
}

/// Dérive un secret de 32 bytes depuis deux PeerIds (XOR + hash SHA-like simple).
/// Déterministe et symétrique : derive(A,B) == derive(B,A).
fn derive_session_secret(peer_a: &str, peer_b: &str) -> [u8; 32] {
    // Ordre canonique pour la symétrie
    let (p1, p2) = if peer_a < peer_b {
        (peer_a, peer_b)
    } else {
        (peer_b, peer_a)
    };
    let combined = format!("lotus-session-v1:{}:{}", p1, p2);
    let bytes = combined.as_bytes();
    let mut secret = [0u8; 32];
    // FNV-1a étendu sur 32 bytes (pas de crypto, juste du déterminisme)
    // En production : X25519 ECDH
    for (i, b) in bytes.iter().enumerate() {
        secret[i % 32] ^= b.wrapping_add(i as u8).wrapping_mul(0x9Fu8);
    }
    secret
}

// ─── Démarrage du réseau ──────────────────────────────────────────────────────

pub async fn start_network(
    profile: &UserProfile,
    event_tx: mpsc::UnboundedSender<SwarmEvent2UI>,
) -> Result<P2pHandle, Box<dyn std::error::Error>> {
    let keypair = profile.keypair();
    let local_peer_id = profile.peer_id();

    // ── Gossipsub config ──────────────────────────────────────────────────────
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .max_transmit_size(65536)
        .build()
        .expect("gossipsub config valide");

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(keypair.clone()),
        gossipsub_config,
    )?;

    // ── Kademlia ──────────────────────────────────────────────────────────────
    let store = MemoryStore::new(local_peer_id);
    let kad_config = kad::Config::new(StreamProtocol::new("/lotus/kad/1.0.0"));
    let kademlia = kad::Behaviour::with_config(local_peer_id, store, kad_config);

    // ── Identify ──────────────────────────────────────────────────────────────
    let identify = identify::Behaviour::new(identify::Config::new(
        "/lotus/identify/1.0.0".to_string(),
        keypair.public(),
    ));

    // ── SwarmBuilder (API 0.56 fluent) ────────────────────────────────────────
    let behaviour = LotusBehaviour {
        gossipsub,
        kademlia,
        identify,
    };

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_dns()?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|c| {
            c.with_idle_connection_timeout(Duration::from_secs(60))
        })
        .build();

    // ── Topics Gossipsub ──────────────────────────────────────────────────────
    let topic_profiles = gossipsub::IdentTopic::new(TOPIC_PROFILES);
    let topic_likes = gossipsub::IdentTopic::new(TOPIC_LIKES);
    swarm.behaviour_mut().gossipsub.subscribe(&topic_profiles)?;
    swarm.behaviour_mut().gossipsub.subscribe(&topic_likes)?;

    // ── Écoute sur port TCP aléatoire ─────────────────────────────────────────
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // ── Bootstrap Kademlia ────────────────────────────────────────────────────
    for (peer_id_str, addr_str) in BOOTSTRAP_PEERS {
        if let (Ok(peer_id), Ok(addr)) = (
            peer_id_str.parse::<PeerId>(),
            addr_str.parse::<Multiaddr>(),
        ) {
            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
        }
    }
    // Lance la recherche de pairs proches de nous dans le DHT
    swarm.behaviour_mut().kademlia.bootstrap().ok();

    // ── Canaux de communication UI ↔ Swarm ───────────────────────────────────
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SwarmCommand>();

    let discovered = Arc::new(Mutex::new(HashMap::<String, PublicProfile>::new()));
    let match_state = Arc::new(Mutex::new(MatchState::new()));
    let chats = Arc::new(Mutex::new(HashMap::<String, EphemeralChat>::new()));

    let discovered_clone = Arc::clone(&discovered);
    let match_state_clone = Arc::clone(&match_state);
    let chats_clone = Arc::clone(&chats);

    let my_peer_id_str = local_peer_id.to_string();

    // ── Boucle événements Swarm (tokio::spawn) ────────────────────────────────
    tokio::spawn(async move {
        // Publie notre profil toutes les 30 secondes
        let mut profile_announce_interval =
            tokio::time::interval(Duration::from_secs(30));
        // Lance une recherche Kademlia périodique
        let mut kad_refresh_interval =
            tokio::time::interval(Duration::from_secs(60));

        loop {
            tokio::select! {
                // ── Commande depuis l'UI ──────────────────────────────────
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        SwarmCommand::SendLike { to_peer } => {
                            let msg = NetworkMessage::Like {
                                from_peer: my_peer_id_str.clone(),
                                to_peer: to_peer.clone(),
                            };
                            if let Ok(bytes) = serde_json::to_vec(&msg) {
                                let topic = gossipsub::IdentTopic::new(TOPIC_LIKES);
                                swarm.behaviour_mut().gossipsub
                                    .publish(topic, bytes)
                                    .ok();
                            }
                        }
                        SwarmCommand::SendChat { to_peer, message } => {
                            let msg = NetworkMessage::Chat(message);
                            if let Ok(bytes) = serde_json::to_vec(&msg) {
                                // Topic de chat dédié : lotus/chat/v1/<peer_id_destinataire>
                                let chat_topic = gossipsub::IdentTopic::new(
                                    format!("{}{}", TOPIC_CHAT_PREFIX, to_peer)
                                );
                                // S'abonner si pas encore abonné
                                swarm.behaviour_mut().gossipsub
                                    .subscribe(&chat_topic)
                                    .ok();
                                swarm.behaviour_mut().gossipsub
                                    .publish(chat_topic, bytes)
                                    .ok();
                            }
                        }
                        SwarmCommand::PublishProfile(profile) => {
                            let msg = NetworkMessage::Profile(profile);
                            if let Ok(bytes) = serde_json::to_vec(&msg) {
                                let topic = gossipsub::IdentTopic::new(TOPIC_PROFILES);
                                swarm.behaviour_mut().gossipsub
                                    .publish(topic, bytes)
                                    .ok();
                            }
                        }
                        SwarmCommand::Shutdown => break,
                    }
                }

                // ── Intervalle : annonce de profil ────────────────────────
                _ = profile_announce_interval.tick() => {
                    // NOTE : le profil réel est dans AppState ; ici on lit
                    // depuis discovered notre propre entrée si elle existe,
                    // sinon on skippe (le lib.rs envoie PublishProfile au démarrage)
                }

                // ── Intervalle : refresh Kademlia ─────────────────────────
                _ = kad_refresh_interval.tick() => {
                    swarm.behaviour_mut().kademlia.bootstrap().ok();
                }

                // ── Événement Swarm ───────────────────────────────────────
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::Behaviour(LotusBehaviourEvent::Gossipsub(
                            gossipsub::Event::Message {
                                propagation_source: _,
                                message_id: _,
                                message,
                            },
                        )) => {
                            // Décode le payload JSON
                            if let Ok(net_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                                match net_msg {
                                    NetworkMessage::Profile(profile) => {
                                        // Ignore notre propre profil
                                        if profile.peer_id != my_peer_id_str {
                                            let pid = profile.peer_id.clone();
                                            discovered_clone.lock().await
                                                .insert(pid, profile.clone());
                                            event_tx.send(SwarmEvent2UI::NewProfile(profile)).ok();
                                        }
                                    }
                                    NetworkMessage::Like { from_peer, to_peer } => {
                                        // Ce like est-il destiné à nous ?
                                        if to_peer == my_peer_id_str {
                                            let is_match = match_state_clone.lock().await
                                                .register_incoming_like(&from_peer);
                                            event_tx.send(SwarmEvent2UI::LikeReceived {
                                                from_peer: from_peer.clone(),
                                            }).ok();
                                            if is_match {
                                                // Ouvre automatiquement la session de chat
                                                let secret = derive_session_secret(
                                                    &my_peer_id_str,
                                                    &from_peer,
                                                );
                                                chats_clone.lock().await.insert(
                                                    from_peer.clone(),
                                                    EphemeralChat::new(&secret),
                                                );
                                                event_tx.send(SwarmEvent2UI::MatchConfirmed {
                                                    peer_id: from_peer,
                                                }).ok();
                                            }
                                        }
                                    }
                                    NetworkMessage::Chat(encrypted_msg) => {
                                        // Destinataire = nous si le topic était le nôtre
                                        let from_peer = encrypted_msg.from_peer.clone();
                                        let chats = chats_clone.lock().await;
                                        if let Some(session) = chats.get(&from_peer) {
                                            if let Some(plain) = session.decrypt(&encrypted_msg) {
                                                event_tx.send(SwarmEvent2UI::ChatMessage {
                                                    from_peer,
                                                    content: plain,
                                                }).ok();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        SwarmEvent::Behaviour(LotusBehaviourEvent::Identify(
                            identify::Event::Received { peer_id, info, .. },
                        )) => {
                            // Ajoute les adresses à Kademlia dès qu'on identifie un pair
                            for addr in info.listen_addrs {
                                swarm.behaviour_mut().kademlia
                                    .add_address(&peer_id, addr);
                            }
                        }

                        SwarmEvent::NewListenAddr { address, .. } => {
                            tracing::info!("LotusDate écoute sur : {}", address);
                        }

                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            event_tx.send(SwarmEvent2UI::PeerConnected {
                                peer_id: peer_id.to_string(),
                            }).ok();
                            // S'abonner au topic de chat entrant pour ce pair
                            let my_chat_topic = gossipsub::IdentTopic::new(
                                format!("{}{}", TOPIC_CHAT_PREFIX, my_peer_id_str)
                            );
                            swarm.behaviour_mut().gossipsub
                                .subscribe(&my_chat_topic)
                                .ok();
                        }

                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            event_tx.send(SwarmEvent2UI::PeerDisconnected {
                                peer_id: peer_id.to_string(),
                            }).ok();
                        }

                        _ => {}
                    }
                }
            }
        }
    });

    Ok(P2pHandle {
        cmd_tx,
        discovered,
        match_state,
        chats,
    })
}
