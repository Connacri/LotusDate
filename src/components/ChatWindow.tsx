import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface Message {
  content: string;
  sender: 'me' | 'them';
  timestamp: number;
}

interface NewMessagePayload {
  peer_id: string;
  content: string;
}

function ChatWindow({ peerId, onClose }: { peerId: string; onClose: () => void }) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState('');
  const [sending, setSending] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // BUG FIX: unlisten était appelé sans await sur la Promise.
  // La cleanup doit gérer le cas où la Promise n'est pas encore résolue.
  useEffect(() => {
    let unlistenFn: (() => void) | undefined;
    let cancelled = false;

    listen<NewMessagePayload>('new-message', (event) => {
      if (event.payload.peer_id === peerId) {
        setMessages((prev) => [
          ...prev,
          {
            content: event.payload.content,
            sender: 'them',
            timestamp: Date.now(),
          },
        ]);
      }
    }).then((fn) => {
      if (cancelled) {
        // Composant déjà démonté — unlisten immédiatement
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

  // BUG FIX: scroll après rendu, pas pendant (useLayoutEffect serait idéal
  // mais useEffect avec flush synchrone est suffisant ici)
  useEffect(() => {
    const el = scrollRef.current;
    if (el) {
      // requestAnimationFrame garantit que le DOM est peint avant scroll
      requestAnimationFrame(() => {
        el.scrollTop = el.scrollHeight;
      });
    }
  }, [messages]);

  // Focus auto sur l'input à l'ouverture
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const send = useCallback(async () => {
    const trimmed = input.trim();
    // UX FIX: rien à envoyer ou déjà en cours
    if (!trimmed || sending) return;

    setSending(true);
    try {
      await invoke('send_message', { peerId, content: trimmed });
      setMessages((prev) => [
        ...prev,
        { content: trimmed, sender: 'me', timestamp: Date.now() },
      ]);
      setInput('');
    } catch (e) {
      console.error('Failed to send message', e);
      // UX FIX: feedback d'erreur inline plutôt que silencieux
      setMessages((prev) => [
        ...prev,
        {
          content: '⚠️ Échec d\'envoi. Connexion P2P perdue ?',
          sender: 'me',
          timestamp: Date.now(),
        },
      ]);
    } finally {
      setSending(false);
      inputRef.current?.focus();
    }
  }, [input, peerId, sending]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      // BUG FIX: preventDefault évite tout comportement form natif résiduel
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
        <div className="chat-e2ee-badge" title="Chiffrement Double Ratchet bout-en-bout">
          🔐 E2EE
        </div>
      </header>

      <div className="chat-messages" ref={scrollRef} aria-live="polite" aria-label="Messages">
        {messages.length === 0 ? (
          <div className="chat-empty">
            <span aria-hidden="true">💬</span>
            <p>Aucun message pour l'instant.</p>
            <p className="chat-ephemeral-note">Cette conversation sera effacée à la fermeture.</p>
          </div>
        ) : (
          messages.map((m, i) => (
            <div
              key={i}
              className={`msg ${m.sender}`}
              aria-label={m.sender === 'me' ? 'Moi' : 'Eux'}
            >
              {m.content}
            </div>
          ))
        )}
      </div>

      <div className="chat-input-container">
        <input
          ref={inputRef}
          className="chat-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Message…"
          aria-label="Écrire un message"
          disabled={sending}
          maxLength={2000}
        />
        {/* UX FIX: disabled quand rien à envoyer */}
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
