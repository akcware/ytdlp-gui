// Bridges to OS features that make the app feel native: real context menus,
// taskbar progress, toast notifications, clipboard and window lifecycle.
import { getCurrentWindow, ProgressBarStatus } from "@tauri-apps/api/window";
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem } from "@tauri-apps/api/menu";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

/** True when running in a plain browser with the dev mock backend. */
export const isMock = (): boolean => (globalThis as any).__YTDLP_MOCK__ === true;

// ---- menus -------------------------------------------------------------------

export type MenuEntry =
  /** `checked` set (true/false) makes it a check item, e.g. for a language choice. */
  | { text: string; action: () => void; enabled?: boolean; checked?: boolean }
  | { text: string; enabled: false }
  | "separator";

/** Fallback flyout (browser preview, or if the native menu fails). */
export const fallbackMenu = $state<{ current: { entries: MenuEntry[]; x: number; y: number } | null }>({
  current: null,
});

/**
 * Shows a native Windows context menu at (x, y) in window coordinates.
 * Without coordinates it opens at the mouse cursor.
 */
export async function showMenu(entries: MenuEntry[], at: { x: number; y: number }) {
  if (!isMock()) {
    try {
      const items = await Promise.all(
        entries.map((e) =>
          e === "separator"
            ? PredefinedMenuItem.new({ item: "Separator" })
            : "checked" in e && e.checked !== undefined
              ? CheckMenuItem.new({
                  text: e.text,
                  checked: e.checked,
                  enabled: e.enabled ?? true,
                  action: () => e.action(),
                })
              : MenuItem.new({
                  text: e.text,
                  enabled: e.enabled ?? true,
                  action: "action" in e ? () => e.action() : undefined,
                }),
        ),
      );
      const menu = await Menu.new({ items });
      await menu.popup(new LogicalPosition(at.x, at.y));
      return;
    } catch (err) {
      console.warn("native menu failed, using fallback", err);
    }
  }
  fallbackMenu.current = { entries, x: at.x, y: at.y };
}

/** Position for a menu dropping down from a button, like a WinUI MenuFlyout. */
export function belowElement(el: Element): { x: number; y: number } {
  const r = el.getBoundingClientRect();
  return { x: r.left, y: r.bottom + 4 };
}

// ---- clipboard -----------------------------------------------------------------

export async function readClipboard(): Promise<string> {
  try {
    return (await readText()) ?? "";
  } catch {
    return "";
  }
}

export async function writeClipboard(text: string) {
  try {
    await writeText(text);
  } catch {
    await navigator.clipboard?.writeText(text).catch(() => {});
  }
}

// ---- notifications ------------------------------------------------------------

let notifyAllowed: boolean | null = null;

export async function notify(title: string, body: string) {
  try {
    if (notifyAllowed == null) {
      notifyAllowed = await isPermissionGranted();
      if (!notifyAllowed) notifyAllowed = (await requestPermission()) === "granted";
    }
    if (notifyAllowed) sendNotification({ title, body });
  } catch {
    notifyAllowed = false;
  }
}

// ---- window ------------------------------------------------------------------

let lastTaskbar = "";

/** Mirrors overall download progress on the taskbar button. */
export function setTaskbarProgress(state: "none" | "indeterminate" | "error" | number) {
  const key = typeof state === "number" ? `n${Math.round(state)}` : state;
  if (key === lastTaskbar) return;
  lastTaskbar = key;
  const win = getCurrentWindow();
  const payload =
    typeof state === "number"
      ? { status: ProgressBarStatus.Normal, progress: Math.round(state) }
      : {
          status:
            state === "none"
              ? ProgressBarStatus.None
              : state === "error"
                ? ProgressBarStatus.Error
                : ProgressBarStatus.Indeterminate,
        };
  win.setProgressBar(payload).catch(() => {});
}

export function showWindow() {
  return getCurrentWindow()
    .show()
    .catch(() => {});
}
