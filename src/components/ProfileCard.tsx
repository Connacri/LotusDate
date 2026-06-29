import React from 'react';

interface ProfileCardProps {
  profile: {
    peer_id: string;
    pseudonym: string;
    age: number;
    interests: string[];
    geohash: string;
  };
  onLike: () => void;
  isLiking?: boolean;
}

// BUG FIX: ne pas exposer le geohash brut (révèle la position précise).
// On dérive une indication de distance approximative depuis la précision du geohash.
function geohashToProximityLabel(geohash: string): string {
  const len = geohash.length;
  if (len >= 7) return 'Très proche';   // ~76 m
  if (len >= 5) return 'Proche';        // ~4.9 km
  if (len >= 3) return 'Dans la région'; // ~156 km
  return 'Quelque part';
}

const ProfileCard: React.FC<ProfileCardProps> = ({ profile, onLike, isLiking = false }) => {
  return (
    <div className="profile-card">
      <div className="profile-header">
        <h3>{profile.pseudonym}, {profile.age}</h3>
        <span className="proximity-badge" aria-label="Distance approximative">
          📍 {geohashToProximityLabel(profile.geohash)}
        </span>
      </div>

      {profile.interests.length > 0 && (
        <div className="interests-container" aria-label="Centres d'intérêt">
          {profile.interests.map((interest, index) => (
            <span key={index} className="interest-chip">
              {interest}
            </span>
          ))}
        </div>
      )}

      {/* UX FIX: disabled + aria pendant le like, feedback visuel */}
      <button
        className={`like-button${isLiking ? ' liking' : ''}`}
        onClick={onLike}
        disabled={isLiking}
        aria-busy={isLiking}
        aria-label={isLiking ? 'Envoi du like…' : `Liker ${profile.pseudonym}`}
      >
        {isLiking ? (
          <>⏳ Envoi…</>
        ) : (
          <>❤️ Like</>
        )}
      </button>
    </div>
  );
};

export default React.memo(ProfileCard);
