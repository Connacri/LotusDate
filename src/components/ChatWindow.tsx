import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface Message {
  content: string;
  sender: 'me' | 'them';
}

interface NewMessagePayload {
  peer_id: string;
  content: string;
}

function ChatWindow({ peerId, onClose }: { peerId: string; onClose: () => void }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const unlisten = listen<NewMessagePayload>('new-message', (event) => {
      if (event.payload.peer_id === peerId) {
        setMessages((prev) => [...prev, { content: event.payload.content, sender: 'them' }]);
      }
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, [peerId]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages]);

  const send = async () => {
    if (!input.trim()) return;
    try {
      await invoke('send_message', { peerId, content: input });
      setMessages((prev) => [...prev, { content: input, sender: 'me' }]);
      setInput('');
    } catch (e) {
      console.error("Failed to send message", e);
    }
  };

  return (
    <div className="chat">
      <header className="chat-header">
        <button className="back-btn" onClick={onClose}>←</button>
        <div className="chat-user-info">
          <strong>Chat éphémère</strong>
          <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>{peerId.slice(0, 8)}...</div>
        </div>
      </header>

      <div className="chat-messages" ref={scrollRef}>
        {messages.map((m, i) => (
          <div key={i} className={`msg ${m.sender}`}>
            {m.content}
          </div>
        ))}
        {messages.length === 0 && (
          <div style={{ textAlign: 'center', color: 'var(--text-muted)', marginTop: '20px' }}>
            Aucun message. La conversation sera effacée à la fermeture.
          </div>
        )}
      </div>

      <div className="chat-input-container">
        <input
          className="chat-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && send()}
          placeholder="Message..."
        />
        <button className="send-btn" onClick={send}>Envoyer</button>
      </div>
    </div>
  );
}

export default ChatWindow;
