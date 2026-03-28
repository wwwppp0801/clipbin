import { useClipStore } from "../stores/clipStore";
import ClipCard from "./ClipCard";

export default function ClipList() {
  const clips = useClipStore((s) => s.clips);
  const isLoading = useClipStore((s) => s.isLoading);

  if (isLoading && clips.length === 0) {
    return (
      <div className="flex flex-1 items-center justify-center text-sm text-gray-500">
        Loading...
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div
        className="flex flex-1 items-center justify-center text-sm text-gray-500"
        data-testid="empty-state"
      >
        No clips yet. Copy something!
      </div>
    );
  }

  return (
    <div className="flex-1 space-y-1 overflow-y-auto py-1" data-testid="clip-list">
      {clips.map((clip) => (
        <ClipCard key={clip.id} clip={clip} />
      ))}
    </div>
  );
}
