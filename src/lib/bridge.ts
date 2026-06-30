/**
 * Pont plateforme — LotusDate
 * ──────────────────────────────────────────────────────────────────────────
 * L'app Android utilise le vrai backend Rust/libp2p via Tauri (invoke/listen).
 * Le site web statique (GitHub Pages) n'a PAS ce backend : Tauri n'existe pas
 * dans le navigateur, donc tous les invoke() échouaient silencieusement et le
 * site restait bloqué sur "Recherche pairs…".
 *
 * Ce module détecte l'environnement et, sur le web, bascule sur un réseau P2P
 * simulé en local (profils, likes, matches, messages) afin que le site se
 * comporte visuellement et fonctionnellement comme l'application réelle.
 */
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen } from '@tauri-apps/api/event';

export const isTauri = (): boolean =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// ── Mini event-bus pour simuler listen/emit côté web ───────────────────────
type Listener = (event: { payload: any }) => void;
const listeners: Record<string, Listener[]> = {};
function emit(event: string, payload: any) {
  (listeners[event] || []).forEach(fn => fn({ payload }));
}

// ── Identité locale persistée ───────────────────────────────────────────────
function genId() {
  return 'web-' + Math.random().toString(36).slice(2, 10) + Math.random().toString(36).slice(2, 10);
}
function getOrCreateMyId(): string {
  if (typeof localStorage === 'undefined') return genId();
  const stored = localStorage.getItem('lotus_my_peer_id');
  if (stored) return stored;
  const id = genId();
  localStorage.setItem('lotus_my_peer_id', id);
  return id;
}
const MY_PEER_ID = getOrCreateMyId();

// ── Génération de profils simulés ───────────────────────────────────────────
const PSEUDOS = ['Aria', 'Lina', 'Sami', 'Nour', 'Yanis', 'Maya', 'Iris', 'Zoé', 'Karim', 'Eléa'];
const INTERESTS_POOL = ['Musique', 'Voyage', 'Cinéma', 'Sport', 'Lecture', 'Cuisine', 'Photo', 'Tech', 'Art', 'Nature'];

function randomProfile() {
  const interests = [...INTERESTS_POOL]
    .sort(() => 0.5 - Math.random())
    .slice(0, 2 + Math.floor(Math.random() * 2));
  return {
    peer_id: genId(),
    pseudonym: PSEUDOS[Math.floor(Math.random() * PSEUDOS.length)] + Math.floor(Math.random() * 99),
    age: 19 + Math.floor(Math.random() * 20),
    interests,
    geohash: 'sw8' + Math.random().toString(36).slice(2, 2 + Math.floor(Math.random() * 4)),
  };
}

let mockProfiles = Array.from({ length: 4 + Math.floor(Math.random() * 3) }, randomProfile);
const matched = new Set<string>();
const chatHistory: Record<string, { content: string; sender: 'me' | 'them' }[]> = {};

// Simule l'arrivée occasionnelle de nouveaux pairs sur le réseau
if (typeof window !== 'undefined' && !isTauri()) {
  setInterval(() => {
    if (Math.random() < 0.35 && mockProfiles.length < 9) {
      mockProfiles = [...mockProfiles, randomProfile()];
      emit('profiles-updated', {});
    }
  }, 8000);
}

const REPLIES = [
  'Haha sympa ! 😄',
  'Ah intéressant, raconte-moi en plus.',
  'Et toi, tu fais quoi de beau ce soir ?',
  'Trop bien, j\'adore ce genre de truc !',
  'Hâte d\'en discuter davantage 🌸',
];

async function mockInvoke(cmd: string, args?: Record<string, unknown>): Promise<any> {
  switch (cmd) {
    case 'get_my_profile':
      return { peer_id: MY_PEER_ID, pseudonym: 'Moi', age: 0, interests: [], geohash: '' };

    case 'get_profiles':
      return mockProfiles;

    case 'get_battery_status':
      return 0.82;

    case 'send_like': {
      const peerId = args?.peerId as string;
      const immediateMatch = Math.random() < 0.6;
      if (immediateMatch) {
        matched.add(peerId);
        return true;
      }
      setTimeout(() => {
        matched.add(peerId);
        emit('match-confirmed', { peer_id: peerId });
      }, 1500 + Math.random() * 2000);
      return false;
    }

    case 'open_chat': {
      const peerId = args?.peerId as string;
      if (!chatHistory[peerId]) chatHistory[peerId] = [];
      return undefined;
    }

    case 'close_chat':
      return undefined;

    case 'send_message': {
      const peerId = args?.peerId as string;
      const content = args?.content as string;
      if (!chatHistory[peerId]) chatHistory[peerId] = [];
      chatHistory[peerId].push({ content, sender: 'me' });
      setTimeout(() => {
        const reply = REPLIES[Math.floor(Math.random() * REPLIES.length)];
        emit('new-message', { peer_id: peerId, content: reply });
      }, 1200 + Math.random() * 1800);
      return undefined;
    }

    default:
      return undefined;
  }
}

function mockListen(event: string, handler: Listener) {
  if (!listeners[event]) listeners[event] = [];
  listeners[event].push(handler);
  return Promise.resolve(() => {
    listeners[event] = (listeners[event] || []).filter(fn => fn !== handler);
  });
}

// ── Export unifié ────────────────────────────────────────────────────────
export const invoke = (isTauri() ? tauriInvoke : mockInvoke) as typeof tauriInvoke;
export const listen = (isTauri() ? tauriListen : mockListen) as typeof tauriListen;
