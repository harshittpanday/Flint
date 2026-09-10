export type CosmeticBytes = number[] | ArrayBuffer | Uint8Array;

export function cosmeticBytesToBlob(bytes: CosmeticBytes): Blob {
  const source = bytes instanceof Uint8Array
    ? bytes
    : bytes instanceof ArrayBuffer
      ? new Uint8Array(bytes)
      : Uint8Array.from(bytes);
  const copy = new Uint8Array(source.byteLength);
  copy.set(source);
  return new Blob([copy.buffer], { type: "image/png" });
}

export function createCosmeticPreviewUrl(
  bytes: CosmeticBytes,
  createObjectUrl: (blob: Blob) => string = (blob) => URL.createObjectURL(blob),
): string {
  const blob = cosmeticBytesToBlob(bytes);
  if (blob.size === 0) throw new Error("Cosmetic preview data is empty.");
  return createObjectUrl(blob);
}

export async function loadCosmeticPreview(
  read: () => Promise<CosmeticBytes>,
  createObjectUrl?: (blob: Blob) => string,
): Promise<string> {
  try {
    return createCosmeticPreviewUrl(await read(), createObjectUrl);
  } catch {
    return "";
  }
}
