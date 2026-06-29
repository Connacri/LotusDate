import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

interface Props {
  peerId: string;
  onClose: () => void;
}

export default function ChatWindow({ peerId, onClose }: Props) {
  const [messages, setMessages] = useState<string[]>([]);
  const [input, setInput] = useState('');

  useEffect(() => {
    const unlisten = listen<string>('new-message', (event) => {
      if (event.payload.startsWith(peerId)) {
        setMessages(prev => [...prev, event.payload]);
      }
    });
    return () => { unlisten.then(fn => fn()) };
  }, [peerId]);

  const send = async () => {
    if (!input.trim()) return;
    await invoke('send_message', { peerId, message: input });
    setMessages(prev => [...prev, `Moi: ${input}`]);
    setInput('');
  };

  const handleClose = async () => {
    await invoke('close_chat', { peerId });
    onClose();
  };

  return (
    <div className="chat-window">
      <div className="messages">
        {messages.map((m, i) => <p key={i}>{m}</p>)}
      </div>
      <div className="input-bar">
        <input value={input} onChange={e => setInput(e.target.value)} />
        <button onClick={send}>Envoyer</button>
      </div>
      <button onClick={handleClose}>Fermer (détruire le chat)</button>
    </div>
  );
}