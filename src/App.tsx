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
  const [loading, setLoading] = useState(true);

  const fetchProfiles = useCallback(async () => {
    try {
      const p = await invoke<PublicProfile[]>('get_profiles');
      // Comparison to avoid redundant state updates
      setProfiles(prev => {
        if (JSON.stringify(prev) === JSON.stringify(p)) return prev;
        return p;
      });
    } catch (e) {
      console.error("Failed to fetch profiles", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const fetchBattery = useCallback(async () => {
    try {
      const level = await invoke<number>('get_battery_status');
      setBattery(prev => prev === level ? prev : level);
    } catch (e) {
      console.error("Failed to fetch battery status", e);
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
    try {
      const matched = await invoke<boolean>('send_like', { peerId });
      if (matched) {
        await invoke('open_chat', { peerId });
        setChatPeer(peerId);
      }
    } catch (e) {
      console.error("Error in handleLike", e);
    }
  };

  const handleCloseChat = async () => {
    if (chatPeer) {
      await invoke('close_chat', { peerId: chatPeer });
      setChatPeer(null);
    }
  };

  return (
    <div className="app">
      {!chatPeer && <BatteryIndicator level={battery} />}

      {chatPeer ? (
        <ChatWindow peerId={chatPeer} onClose={handleCloseChat} />
      ) : (
        <div className="profile-stack">
          {profiles.length > 0 ? (
            profiles.map((p) => (
              <ProfileCard key={p.peer_id} profile={p} onLike={() => handleLike(p.peer_id)} />
            ))
          ) : (
            <div className="empty-state">
              {!loading ? (
                <>
                  <h2>Personne à l'horizon...</h2>
                  <p>LotusDate recherche d'autres fleurs dans le réseau P2P. Assure-toi d'être connecté !</p>
                </>
              ) : (
                <p>Initialisation du réseau Lotus...</p>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default App;
