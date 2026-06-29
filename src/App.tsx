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

// UX FIX: on garde le pseudonyme du match pour l'afficher dans la notif
interface MatchNotif {
  peerId: string;
  pseudonym: string;
}

function App() {
  const [profiles, setProfiles] = useState<PublicProfile[]>([]);
  const [chatPeer, setChatPeer] = useState<string | null>(null);
  const [battery, setBattery] = useState<number>(1.0);
  const [loading, setLoading] = useState(true);
  const [likingPeer, setLikingPeer] = useState<string | null>(null);
  const [matchNotif, setMatchNotif] = useState<MatchNotif | null>(null);
  // UX FIX: message d'erreur global (réseau, invoke échoué)
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const fetchProfiles = useCallback(async () => {
    try {
      const p = await invoke<PublicProfile[]>('get_profiles');
      setProfiles(prev =>
        JSON.stringify(prev) === JSON.stringify(p) ? prev : p
      );
      // UX FIX: effacer l'erreur dès qu'on récupère des données
      setErrorMsg(null);
    } catch (e) {
      console.error('Failed to fetch profiles', e);
      setErrorMsg('Réseau P2P indisponible. Vérifiez votre connexion.');
    } finally {
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

  const handleLike = async (peerId: string, pseudonym: string) => {
    if (likingPeer) return;
    setLikingPeer(peerId);
    try {
      const matched = await invoke<boolean>('send_like', { peerId });
      if (matched) {
        // UX FIX: notif avec le vrai pseudonyme
        setMatchNotif({ peerId, pseudonym });
        await invoke('open_chat', { peerId });
        setTimeout(() => {
          setMatchNotif(null);
          setChatPeer(peerId);
        }, 1400);
      }
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

  return (
    <div className="app">
      {/* Notification de match */}
      {matchNotif && (
        <div className="match-toast" role="alert" aria-live="assertive">
          💞 Match avec {matchNotif.pseudonym} ! Connexion…
        </div>
      )}

      {/* Bannière d'erreur réseau */}
      {errorMsg && !matchNotif && (
        <div className="error-banner" role="alert">
          ⚠️ {errorMsg}
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
            profiles.map(p => (
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
