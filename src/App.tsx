import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
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

function App() {
  const [profiles, setProfiles] = useState<PublicProfile[]>([]);
  const [chatPeer, setChatPeer] = useState<string | null>(null);
  const [battery, setBattery] = useState<number>(1.0);
  // BUG FIX: loading doit démarrer à true et ne passe à false qu'après
  // fetchProfiles (pas fetchBattery qui est indépendant)
  const [loading, setLoading] = useState(true);
  // UX FIX: empêcher double-like
  const [likingPeer, setLikingPeer] = useState<string | null>(null);
  // UX FIX: feedback de match
  const [matchNotif, setMatchNotif] = useState<string | null>(null);

  const fetchProfiles = useCallback(async () => {
    try {
      const p = await invoke<PublicProfile[]>('get_profiles');
      setProfiles(prev => {
        if (JSON.stringify(prev) === JSON.stringify(p)) return prev;
        return p;
      });
    } catch (e) {
      console.error('Failed to fetch profiles', e);
    } finally {
      // Toujours lever le loading même en cas d'erreur
      setLoading(false);
    }
  }, []);

  const fetchBattery = useCallback(async () => {
    try {
      const level = await invoke<number>('get_battery_status');
      setBattery(prev => (prev === level ? prev : level));
    } catch (e) {
      console.error('Failed to fetch battery status', e);
    }
  }, []);

  useEffect(() => {
    fetchProfiles();
    fetchBattery();

    const profileInterval = setInterval(fetchProfiles, 5000);
    const batteryInterval = setInterval(fetchBattery, 15000);

    return () => {
      clearInterval(profileInterval);
      clearInterval(batteryInterval);
    };
  }, [fetchProfiles, fetchBattery]);

  const handleLike = async (peerId: string) => {
    // UX FIX: ignorer si déjà en cours de like
    if (likingPeer) return;
    setLikingPeer(peerId);
    try {
      const matched = await invoke<boolean>('send_like', { peerId });
      if (matched) {
        // UX FIX: notif match avant d'ouvrir le chat
        setMatchNotif(peerId);
        await invoke('open_chat', { peerId });
        setTimeout(() => {
          setMatchNotif(null);
          setChatPeer(peerId);
        }, 1200);
      }
    } catch (e) {
      console.error('Error in handleLike', e);
    } finally {
      setLikingPeer(null);
    }
  };

  const handleCloseChat = async () => {
    if (chatPeer) {
      try {
        await invoke('close_chat', { peerId: chatPeer });
      } catch (e) {
        // BUG FIX: ne pas bloquer la fermeture UI si l'invoke échoue
        console.error('Error closing chat', e);
      } finally {
        setChatPeer(null);
      }
    }
  };

  return (
    <div className="app">
      {/* UX FIX: notification de match */}
      {matchNotif && (
        <div className="match-toast" aria-live="polite">
          💞 C'est un match ! Connexion en cours…
        </div>
      )}

      {!chatPeer && <BatteryIndicator level={battery} />}

      {chatPeer ? (
        <ChatWindow peerId={chatPeer} onClose={handleCloseChat} />
      ) : (
        <div className="profile-stack">
          {loading ? (
            <div className="empty-state">
              <div className="spinner" aria-label="Chargement" />
              <p>Initialisation du réseau Lotus…</p>
            </div>
          ) : profiles.length > 0 ? (
            profiles.map((p) => (
              <ProfileCard
                key={p.peer_id}
                profile={p}
                onLike={() => handleLike(p.peer_id)}
                isLiking={likingPeer === p.peer_id}
              />
            ))
          ) : (
            <div className="empty-state">
              <div className="empty-icon" aria-hidden="true">🌸</div>
              <h2>Personne à l'horizon…</h2>
              <p>LotusDate explore le réseau P2P. Assure-toi d'être connecté·e !</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default App;
