export const MASKED_API_KEY_PLACEHOLDER = '••••••••••••••••••••••••••••••••••••'

export function maskApiKey(key: string): string {
  if (key.length <= 12) return key
  return `${key.slice(0, 8)}...${key.slice(-4)}`
}
