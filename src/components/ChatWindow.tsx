import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Message {
  content: string;
  sender: 'me' | 'them';
}

function ChatWindow({ peerId, onClose }: { peerId: string; onClose: () => void }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');

  const send = async () => {
    if (!input.trim()) return;
    await invoke('send_message', { peerId, content: input });
    setMessages([...messages, { content: input, sender: 'me' }]);
    setInput('');
  };

  return (
    <div className="chat">
      <button onClick={onClose}>Retour</button>
      <div className="messages">
        {messages.map((m, i) => (
          <div key={i} className={`msg ${m.sender}`}>{m.content}</div>
        ))}
      </div>
      <input value={input} onChange={(e) => setInput(e.target.value)} />
      <button onClick={send}>Envoyer</button>
    </div>
  );
}

export default ChatWindow;
