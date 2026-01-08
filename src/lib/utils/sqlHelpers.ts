export function parseSqlScript(script: string): string[] {
  if (!script) return [];

  const noBlockComments = script.replace(/\/\*[\s\S]*?\*\//g, "");

  const noLineComments = noBlockComments.replace(/\-\-.*$/gm, "");

  return noLineComments
    .split(";")
    .map((q) => q.trim())
    .filter((q) => q.length > 0);
}

export function isSelectQuery(query: string): boolean {
  const q = query.toLowerCase();
  return (
    q.startsWith("select") ||
    q.startsWith("values") ||
    q.startsWith("with") ||
    q.startsWith("explain") ||
    q.startsWith("show")
  );
}
