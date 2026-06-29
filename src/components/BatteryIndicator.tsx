import React from 'react';

interface BatteryIndicatorProps {
  level: number;
}

const BatteryIndicator: React.FC<BatteryIndicatorProps> = ({ level }) => {
  const isNode = level > 0.3;
  const percentage = Math.round(level * 100);

  // Calculate battery color
  let batteryColor = '#ef4444'; // Red
  if (percentage > 60) batteryColor = '#22c55e'; // Green
  else if (percentage > 20) batteryColor = '#f59e0b'; // Amber

  return (
    <div className={`battery-status ${isNode ? 'node' : 'client'}`}>
      <div style={{
        width: '24px',
        height: '12px',
        border: '1.5px solid currentColor',
        borderRadius: '3px',
        position: 'relative',
        display: 'flex',
        padding: '1px'
      }}>
        <div style={{
          width: `${percentage}%`,
          height: '100%',
          backgroundColor: batteryColor,
          borderRadius: '1px'
        }} />
        <div style={{
          position: 'absolute',
          right: '-4px',
          top: '3px',
          width: '3px',
          height: '4px',
          backgroundColor: 'currentColor',
          borderRadius: '0 1px 1px 0'
        }} />
      </div>
      <span>{percentage}% • {isNode ? 'Nœud Actif (P2P Full)' : 'Mode Économie (Light)'}</span>
    </div>
  );
};

export default React.memo(BatteryIndicator);
