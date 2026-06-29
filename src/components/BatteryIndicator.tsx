import React from 'react';

interface BatteryIndicatorProps {
  level: number; // 0.0 à 1.0
}

const BatteryIndicator: React.FC<BatteryIndicatorProps> = ({ level }) => {
  // Clamp niveau entre 0 et 1
  const clamped = Math.max(0, Math.min(1, level));
  const percentage = Math.round(clamped * 100);
  const isNode = clamped > 0.3;

  // BUG FIX: les seuils de couleur étaient indépendants du seuil de mode nœud (30%).
  // On aligne maintenant : vert ≥ 60%, ambre 30-59% (zone transition), rouge < 30%.
  let batteryColor: string;
  if (percentage >= 60) {
    batteryColor = '#22c55e'; // Vert
  } else if (percentage >= 30) {
    batteryColor = '#f59e0b'; // Ambre (zone critique pour le nœud)
  } else {
    batteryColor = '#ef4444'; // Rouge (nœud inactif)
  }

  return (
    <div
      className={`battery-status ${isNode ? 'node' : 'client'}`}
      role="status"
      aria-label={`Batterie ${percentage}%, ${isNode ? 'nœud P2P actif' : 'mode économie'}`}
    >
      {/* Icône batterie SVG inline pour éviter des dépendances */}
      <svg
        width="24"
        height="13"
        viewBox="0 0 24 13"
        fill="none"
        aria-hidden="true"
        style={{ flexShrink: 0 }}
      >
        {/* Corps */}
        <rect x="0.75" y="0.75" width="20.5" height="11.5" rx="2.25" stroke="currentColor" strokeWidth="1.5" />
        {/* Borne + */}
        <path d="M22 4.5V8.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
        {/* Remplissage */}
        <rect
          x="2"
          y="2"
          width={Math.max(0, (percentage / 100) * 17)}
          height="9"
          rx="1"
          fill={batteryColor}
        />
      </svg>

      <span>
        {percentage}%&nbsp;·&nbsp;
        {isNode ? (
          <strong>Nœud actif</strong>
        ) : (
          <span>Mode économie</span>
        )}
      </span>
    </div>
  );
};

export default React.memo(BatteryIndicator);
