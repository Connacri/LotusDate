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
  const [isConfirmingClose, setIsConfirmingClose] = useState(false);

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

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      send();
    }
  };

  const handleClose = async () => {
    if (isConfirmingClose) {
      await invoke('close_chat', { peerId });
      onClose();
    } else {
      setIsConfirmingClose(true);
      // Reset confirmation after 3 seconds
      setTimeout(() => setIsConfirmingClose(false), 3000);
    }
  };

  return (
    <div className="chat-window">
      <div className="messages" role="log" aria-live="polite">
        {messages.map((m, i) => <p key={i}>{m}</p>)}
      </div>
      <div className="input-bar">
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Écrivez un message..."
          aria-label="Message à envoyer"
        />
        <button onClick={send} aria-label="Envoyer le message">Envoyer</button>
      </div>
      <button
        onClick={handleClose}
        className={isConfirmingClose ? 'confirming' : ''}
        aria-label={isConfirmingClose ? "Confirmer la destruction du chat" : "Fermer et détruire le chat"}
      >
        {isConfirmingClose ? 'Êtes-vous sûr ? (Destruction définitive)' : 'Fermer (détruire le chat)'}
      </button>
    </div>
  );
}
