// Which desktop the app runs on. The UI follows each platform's conventions:
// WinUI 3 / Fluent on Windows, AppKit on macOS. Platform styles hang off a
// class on <html> (`windows` or `macos`), set here before the first paint.

function detect(): "macos" | "windows" {
  if (typeof navigator === "undefined") return "windows";
  // In a plain browser during development, `?platform=windows` or
  // `?platform=macos` previews the other look.
  if (import.meta.env.DEV) {
    const forced = new URLSearchParams(location.search).get("platform");
    if (forced === "macos" || forced === "windows") return forced;
  }
  return /mac/i.test(navigator.platform || navigator.userAgent) ? "macos" : "windows";
}

export const platform = detect();
export const isMac = platform === "macos";

if (typeof document !== "undefined") {
  document.documentElement.classList.add(platform);
}

/** "⌘V" on macOS, "Ctrl+V" elsewhere. `key` is "V", "L", "Enter"… */
export function shortcut(key: string): string {
  if (!isMac) return `Ctrl+${key}`;
  return `⌘${key === "Enter" ? "↩" : key}`;
}
