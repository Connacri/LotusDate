import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke, listen } from '../lib/bridge';

interface Message {
  content: string;
  sender: 'me' | 'them';
  timestamp: number;
  // UX FIX: état d'envoi pour indicateur visuel
  status?: 'sending' | 'sent' | 'error';
}

interface NewMessagePayload {
  peer_id: string;
  content: string;
}

// UX FIX: formater timestamp en HH:MM
function formatTime(ts: number): string {
  return new Date(ts).toLocaleTimeString('fr-FR', {
    hour: '2-digit',
    minute: '2-digit',
  });
}

function ChatWindow({ peerId, onClose }: { peerId: string; onClose: () => void }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // BUG FIX: unlisten géré correctement (cancel flag)
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    let cancelled = false;

    listen<NewMessagePayload>('new-message', event => {
      if (event.payload.peer_id === peerId) {
        setMessages(prev => [
          ...prev,
          {
            content: event.payload.content,
            sender: 'them',
            timestamp: Date.now(),
            status: 'sent',
          },
        ]);
      }
    }).then(fn => {
      if (cancelled) {
        fn();
      } else {
        unlistenFn = fn;
      }
    });

    return () => {
      cancelled = true;
      unlistenFn?.();
    };
  }, [peerId]);

  // BUG FIX: scroll après rendu via requestAnimationFrame
  useEffect(() => {
    const el = scrollRef.current;
    if (el) {
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight;
      });
    }
  }, [messages]);

  // Focus auto à l'ouverture
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const send = useCallback(async () => {
    const trimmed = input.trim();
    if (!trimmed || sending) return;

    const ts = Date.now();
    const tempMsg: Message = {
      content: trimmed,
      sender: 'me',
      timestamp: ts,
      status: 'sending',
    };

    // UX FIX: message optimiste immédiat
    setMessages(prev => [...prev, tempMsg]);
    setInput('');
    setSending(true);

    try {
      await invoke('send_message', { peerId, content: trimmed });
      // Marquer le message comme envoyé
      setMessages(prev =>
        prev.map(m =>
          m.timestamp === ts && m.sender === 'me'
            ? { ...m, status: 'sent' }
            : m
        )
      );
    } catch (e) {
      console.error('Failed to send message', e);
      // UX FIX: marquer comme erreur (pas supprimer)
      setMessages(prev =>
        prev.map(m =>
          m.timestamp === ts && m.sender === 'me'
            ? { ...m, status: 'error', content: m.content + ' ⚠️' }
            : m
        )
      );
    } finally {
      setSending(false);
      inputRef.current?.focus();
    }
  }, [input, peerId, sending]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      send();
    }
  };

  const canSend = input.trim().length > 0 && !sending;

  return (
    <div className="chat" role="main" aria-label="Conversation éphémère">
      <header className="chat-header">
        <button
          className="back-btn"
          onClick={onClose}
          aria-label="Retour aux profils"
          title="Fermer et effacer le chat"
        >
          ←
        </button>
        <div className="chat-user-info">
          <strong>Chat éphémère</strong>
          <div className="peer-id-label" title={peerId}>
            🔒 {peerId.slice(0, 8)}…
          </div>
        </div>
        <div className="chat-e2ee-badge" title="Chiffrement bout-en-bout">
          🔐 E2EE
        </div>
      </header>

      <div
        className="chat-messages"
        ref={scrollRef}
        aria-live="polite"
        aria-label="Messages"
      >
        {messages.length === 0 ? (
          <div className="chat-empty">
            <span aria-hidden="true">💬</span>
            <p>Aucun message pour l'instant.</p>
            <p className="chat-ephemeral-note">
              Cette conversation sera effacée à la fermeture.
            </p>
          </div>
        ) : (
          messages.map((m, i) => (
            <div
              key={i}
              className={`msg ${m.sender}${m.status === 'error' ? ' msg-error' : ''}`}
              aria-label={m.sender === 'me' ? 'Moi' : 'Eux'}
            >
              <span className="msg-content">{m.content}</span>
              {/* UX FIX: timestamp + indicateur d'envoi */}
              <span className="msg-meta">
                {formatTime(m.timestamp)}
                {m.sender === 'me' && (
                  <span className="msg-status" aria-hidden="true">
                    {m.status === 'sending' && ' ⏳'}
                    {m.status === 'sent' && ' ✓'}
                    {m.status === 'error' && ' ✗'}
                  </span>
                )}
              </span>
            </div>
          ))
        )}
      </div>

      <div className="chat-input-container">
        <input
          ref={inputRef}
          className="chat-input"
          value={input}
          onChange={e => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Message…"
          aria-label="Écrire un message"
          disabled={sending}
          maxLength={2000}
        />
        <button
          className="send-btn"
          onClick={send}
          disabled={!canSend}
          aria-label="Envoyer"
          aria-busy={sending}
        >
          {sending ? '…' : '↑'}
        </button>
      </div>
    </div>
  );
}

export default ChatWindow;
