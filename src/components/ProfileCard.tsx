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
}

const ProfileCard: React.FC<ProfileCardProps> = ({ profile, onLike }) => {
  return (
    <div className="profile-card">
      <h3>{profile.pseudonym}, {profile.age}</h3>
      <p>À proximité • {profile.geohash}</p>

      <div className="interests-container">
        {profile.interests.map((interest, index) => (
          <span key={index} className="interest-chip">
            {interest}
          </span>
        ))}
      </div>

      <button className="like-button" onClick={onLike}>
        <span>❤️</span> Like
      </button>
    </div>
  );
};

export default React.memo(ProfileCard);
