export default function BatteryIndicator({ level }: { level: number }) {
  const isNode = level > 0.3;
  return (
    <div className={`battery ${isNode ? 'node' : 'client'}`}>
      Batterie: {Math.round(level * 100)}% — {isNode ? 'Nœud actif' : 'Client léger'}
    </div>
  );
}