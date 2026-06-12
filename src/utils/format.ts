export function formatDate(date: Date): string {
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}

/**
 * Compact token count display used on the session card footer and the
 * session detail page. Keeps the footer short regardless of magnitude:
 *
 *   < 1K     -> pure integer, no suffix (e.g. 856)
 *   1K..<1M  -> "K" suffix, 1 decimal under 10K, integer from 10K
 *                (e.g. 1.2K, 9.9K, 10K, 123K, 999K)
 *   >= 1M    -> "M" suffix, same decimal rule (e.g. 1.2M, 12M, 100M)
 *
 * Negative or non-finite values fall back to the raw string.
 */
export function formatTokens(n: number): string {
  if (!Number.isFinite(n)) return String(n)
  if (n < 1000) return String(Math.round(n))
  if (n < 1_000_000) {
    const v = n / 1000
    // Round to one decimal first so 9.95K+ jumps cleanly to 10K instead
    // of displaying as "10.0K".
    const rounded = Math.round(v * 10) / 10
    if (rounded < 10) return v.toFixed(1) + 'K'
    return String(Math.round(v)) + 'K'
  }
  const v = n / 1_000_000
  const rounded = Math.round(v * 10) / 10
  if (rounded < 10) return v.toFixed(1) + 'M'
  return String(Math.round(v)) + 'M'
}
