export function formatRelativeTime(isoDate: string): string {
  const now = Date.now();
  const then = new Date(isoDate).getTime();
  const diffMs = now - then;

  if (diffMs < 0) return "just now";

  const seconds = Math.floor(diffMs / 1000);
  if (seconds < 60) return "just now";

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;

  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;

  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;

  const months = Math.floor(days / 30);
  return `${months}mo ago`;
}

export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength).trimEnd() + "...";
}

export function isUrl(text: string | null | undefined): boolean {
  if (!text) return false;
  const trimmed = text.trim();
  // Single line URL
  return !trimmed.includes("\n") && /^https?:\/\/\S+$/.test(trimmed);
}

export function getContentIcon(contentType: string): string {
  switch (contentType) {
    case "image":
      return "image";
    case "file_path":
      return "file";
    default:
      return "text";
  }
}
