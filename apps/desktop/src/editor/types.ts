/** Runtime language metadata. Languages without a grammar intentionally fall back to plain text. */
export interface LanguageDescriptor {
  id: string;
  extensions: readonly string[];
  server?: readonly string[];
  rootMarkers?: readonly string[];
  grammar?: "javascript" | "typescript" | "plain";
}

export interface EditorFile {
  uri: string;
  path: string;
  languageId: string;
  content: string;
}

export const plainTextDescriptor: LanguageDescriptor = {
  id: "plain",
  extensions: [],
  grammar: "plain",
};

export function descriptorForPath(
  path: string,
  catalog: readonly LanguageDescriptor[] = [],
): LanguageDescriptor {
  const extension = path.slice(path.lastIndexOf(".")).toLowerCase();
  return (
    catalog.find((descriptor) => descriptor.extensions.includes(extension)) ??
    plainTextDescriptor
  );
}
