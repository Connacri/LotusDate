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

export default function ProfileCard({ profile, onLike }: ProfileCardProps) {
  return (
    <div className="profile-card">
      <h3>{profile.pseudonym}, {profile.age}</h3>
      <p>{profile.interests.join(", ")}</p>
      <button onClick={onLike}>❤️ Like</button>
    </div>
  );
}