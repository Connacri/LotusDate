import { useState, useEffect } from 'react';
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
  const [battery, setBattery] = useState<number>(100);

  useEffect(() => {
    const load = async () => {
      const p = await invoke<PublicProfile[]>('get_profiles');
      setProfiles(p);
    };
    load();
    const interval = setInterval(load, 3000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    const getBattery = async () => {
      const level = await invoke<number>('get_battery_status');
      setBattery(level);
    };
    getBattery();
    const interval = setInterval(getBattery, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleLike = async (peerId: string) => {
    const matched = await invoke<boolean>('send_like', { peerId });
    if (matched) {
      await invoke('open_chat', { peerId });
      setChatPeer(peerId);
    }
  };

  return (
    <div className="app">
      <BatteryIndicator level={battery} />
      {chatPeer ? (
        <ChatWindow peerId={chatPeer} onClose={() => setChatPeer(null)} />
      ) : (
        <div className="profile-stack">
          {profiles.map((p) => (
            <ProfileCard key={p.peer_id} profile={p} onLike={() => handleLike(p.peer_id)} />
          ))}
        </div>
      )}
    </div>
  );
}

export default App;
