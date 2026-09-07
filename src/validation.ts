export function isValidMinecraftUsername(username: string): boolean {
  return /^[A-Za-z0-9_]{3,16}$/.test(username);
}

export function isValidProfileName(name: string): boolean {
  const length = [...name.trim()].length;
  return length >= 1 && length <= 40;
}
