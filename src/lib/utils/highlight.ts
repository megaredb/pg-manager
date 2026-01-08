export function highlightMatch(text: string, query: string) {
  if (!query || query.trim() === "") return text;
  const safeQuery = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${safeQuery})`, "gi");
  return text.replace(
    regex,
    '<mark class="bg-warning/40 text-inherit rounded-sm px-0.5">$1</mark>'
  );
}
