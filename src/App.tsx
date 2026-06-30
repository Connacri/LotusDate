import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import ProfileCard from './components/ProfileCard';
import ChatWindow from './components/ChatWindow';
import BatteryIndicator from './components/BatteryIndicator';

interface PublicProfile {
  peer_id: string;
  pseudonym: string;
  age: number;
  interests: string[];
  geohash: string;
}

interface MatchNotif {
  peerId: string;
  pseudonym: string;
}

interface MatchPayload {
  peer_id: string;
}

interface LikePayload {
  from_peer: string;
}

function App() {
  const [profiles, setProfiles] = useState<PublicProfile[]>([]);
  const [chatPeer, setChatPeer] = useState<string | null>(null);
  const [battery, setBattery] = useState<number>(1.0);
  const [loading, setLoading] = useState(true);
  const [likingPeer, setLikingPeer] = useState<string | null>(null);
  const [matchNotif, setMatchNotif] = useState<MatchNotif | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [myPeerId, setMyPeerId] = useState<string>('');
  const [peerCount, setPeerCount] = useState<number>(0);

  // Récupère notre propre PeerId au démarrage
  useEffect(() => {
    invoke<PublicProfile>('get_my_profile')
      .then(p => setMyPeerId(p.peer_id))
      .catch(console.error);
  }, []);

  const fetchProfiles = useCallback(async () => {
    try {
      const p = await invoke<PublicProfile[]>('get_profiles');
      setProfiles(prev =>
        JSON.stringify(prev) === JSON.stringify(p) ? prev : p
      );
      setPeerCount(p.length);
      setErrorMsg(null);
    } catch (e) {
      console.error('Failed to fetch profiles', e);
      setErrorMsg('Réseau P2P en cours d\'initialisation…');
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchBattery = useCallback(async () => {
    try {
      const nav = navigator as any;
      if (nav.getBattery) {
        const bat = await nav.getBattery();
        setBattery((prev) => prev === bat.level ? prev : bat.level);
        bat.onlevelchange = () => setBattery(bat.level);
      } else {
        const level = await invoke<number>('get_battery_status');
        setBattery(prev => prev === level ? prev : level);
      }
    } catch (e) {
      console.error('Failed to fetch battery status', e);
    }
  }, []);

  useEffect(() => {
    fetchProfiles();
    fetchBattery();

    // Polling toutes les 5 secondes (complété par l'événement "profiles-updated")
    const profileInterval = setInterval(fetchProfiles, 5000);
    const batteryInterval = setInterval(fetchBattery, 60000);

    return () => {
      clearInterval(profileInterval);
      clearInterval(batteryInterval);
    };
  }, [fetchProfiles, fetchBattery]);

  // ── Écoute des événements réseau réels ──────────────────────────────────────

  useEffect(() => {
    // Le réseau a annoncé un nouveau profil → rafraîchir la liste
    const unlistenProfiles = listen('profiles-updated', () => {
      fetchProfiles();
    });

    // Match confirmé depuis le réseau (l'autre a liké en retour)
    const unlistenMatch = listen<MatchPayload>('match-confirmed', async (event) => {
      const { peer_id } = event.payload;
      const prof = profiles.find(p => p.peer_id === peer_id);
      const pseudonym = prof?.pseudonym ?? peer_id.slice(0, 8) + '…';
      setMatchNotif({ peerId: peer_id, pseudonym });
      try {
        await invoke('open_chat', { peerId: peer_id });
      } catch (e) {
        console.error('open_chat error', e);
      }
      setTimeout(() => {
        setMatchNotif(null);
        setChatPeer(peer_id);
      }, 1400);
    });

    // Quelqu'un nous a liké (pas forcément un match)
    const unlistenLike = listen<LikePayload>('like-received', (event) => {
      const { from_peer } = event.payload;
      console.info('Like reçu de', from_peer);
      // Optionnel : afficher une notification discrète
    });

    return () => {
      unlistenProfiles.then(fn => fn());
      unlistenMatch.then(fn => fn());
      unlistenLike.then(fn => fn());
    };
  }, [profiles, fetchProfiles]);

  // ── Actions utilisateur ──────────────────────────────────────────────────────

  const handleLike = async (peerId: string, pseudonym: string) => {
    if (likingPeer) return;
    setLikingPeer(peerId);
    try {
      // send_like retourne true si MATCH LOCAL IMMÉDIAT (l'autre nous avait déjà likés)
      const matched = await invoke<boolean>('send_like', { peerId });
      if (matched) {
        setMatchNotif({ peerId, pseudonym });
        await invoke('open_chat', { peerId });
        setTimeout(() => {
          setMatchNotif(null);
          setChatPeer(peerId);
        }, 1400);
      }
      // Si pas de match immédiat, le match arrivera via 'match-confirmed' plus tard
    } catch (e) {
      console.error('Error in handleLike', e);
      setErrorMsg('Impossible d\'envoyer le like. Connexion P2P perdue ?');
      setTimeout(() => setErrorMsg(null), 3000);
    } finally {
      setLikingPeer(null);
    }
  };

  const handleCloseChat = async () => {
    if (chatPeer) {
      try {
        await invoke('close_chat', { peerId: chatPeer });
      } catch (e) {
        console.error('Error closing chat', e);
      } finally {
        setChatPeer(null);
      }
    }
  };

  // ── Rendu ────────────────────────────────────────────────────────────────────

  return (
    <div className="app">
      {matchNotif && (
        <div className="match-toast" role="alert" aria-live="assertive">
          💞 Match avec {matchNotif.pseudonym} ! Connexion…
        </div>
      )}

      {errorMsg && !matchNotif && (
        <div className="error-banner" role="alert">
          ⚠️ {errorMsg}
        </div>
      )}

      {!chatPeer && (
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', padding: '0 8px' }}>
          <BatteryIndicator level={battery} />
          {/* Compteur de pairs connectés — indicateur réseau réel */}
          <span
            style={{ fontSize: '0.75rem', opacity: 0.6 }}
            title={myPeerId}
          >
            {peerCount > 0
              ? `🟢 ${peerCount} pair${peerCount > 1 ? 's' : ''} trouvé${peerCount > 1 ? 's' : ''}`
              : '🔴 Recherche pairs…'}
          </span>
        </div>
      )}

      {chatPeer ? (
        <ChatWindow peerId={chatPeer} onClose={handleCloseChat} />
      ) : (
        <div className="profile-stack">
          {loading ? (
            <div className="empty-state">
              <div className="spinner" aria-label="Chargement" />
              <p>Bootstrap Kademlia en cours…</p>
            </div>
          ) : profiles.length > 0 ? (
            profiles
              .filter(p => p.peer_id !== myPeerId) // Ne pas s'afficher soi-même
              .map(p => (
                <ProfileCard
                  key={p.peer_id}
                  profile={p}
                  onLike={() => handleLike(p.peer_id, p.pseudonym)}
                  isLiking={likingPeer === p.peer_id}
                />
              ))
          ) : (
            <div className="empty-state">
              <div className="empty-icon" aria-hidden="true">🌸</div>
              <h2>Aucun pair trouvé</h2>
              <p>
                Le réseau Kademlia bootstrap en cours.<br />
                D'autres utilisateurs de LotusDate doivent être en ligne pour apparaître ici.
              </p>
              <p style={{ fontSize: '0.75rem', opacity: 0.5 }}>
                ID : {myPeerId ? myPeerId.slice(0, 12) + '…' : '…'}
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default App;