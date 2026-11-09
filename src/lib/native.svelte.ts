// Bridges to OS features that make the app feel native: real context menus,
// the macOS menu bar, taskbar/Dock progress, notifications, clipboard and
// window lifecycle.
import { getCurrentWindow, ProgressBarStatus } from "@tauri-apps/api/window";
import { CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu } from "@tauri-apps/api/menu";
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
  | { text: string; action: () => void; enabled?: boolean; checked?: boolean; accelerator?: string }
  | { text: string; enabled: false }
  | "separator";

/** A standard item AppKit implements itself (Copy, Hide, Minimize…). */
type Predefined = "Undo" | "Redo" | "Cut" | "Copy" | "Paste" | "SelectAll" | "Minimize" | "Maximize"
  | "Hide" | "HideOthers" | "ShowAll" | "CloseWindow" | "Services" | "BringAllToFront";
export type AppMenuEntry = MenuEntry | { predefined: Predefined; text: string };

/** Fallback flyout (browser preview, or if the native menu fails). */
export const fallbackMenu = $state<{ current: { entries: MenuEntry[]; x: number; y: number } | null }>({
  current: null,
});

function menuItems(entries: AppMenuEntry[]) {
  return Promise.all(
    entries.map((e) =>
      e === "separator"
        ? PredefinedMenuItem.new({ item: "Separator" })
        : "predefined" in e
          ? PredefinedMenuItem.new({ item: e.predefined, text: e.text })
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
                accelerator: "accelerator" in e ? e.accelerator : undefined,
                action: "action" in e ? () => e.action() : undefined,
              }),
    ),
  );
}

/**
 * Shows a native context menu (Windows or macOS) at (x, y) in window
 * coordinates.
 */
export async function showMenu(entries: MenuEntry[], at: { x: number; y: number }) {
  if (!isMock()) {
    try {
      const items = await menuItems(entries);
      const menu = await Menu.new({ items });
      await menu.popup(new LogicalPosition(at.x, at.y));
      return;
    } catch (err) {
      console.warn("native menu failed, using fallback", err);
    }
  }
  fallbackMenu.current = { entries, x: at.x, y: at.y };
}

let appMenuToken = 0;

/**
 * Replaces the macOS menu bar. The first submenu is the application menu
 * (AppKit titles it with the app name). Rebuilt whenever the language or the
 * enabled state of an item changes.
 */
export async function setAppMenu(menus: { text: string; items: AppMenuEntry[]; windows?: boolean }[]) {
  if (isMock()) return;
  const token = ++appMenuToken;
  try {
    const submenus = await Promise.all(
      menus.map(async (m) => {
        const submenu = await Submenu.new({ text: m.text, items: await menuItems(m.items) });
        if (m.windows) await submenu.setAsWindowsMenuForNSApp();
        return submenu;
      }),
    );
    if (token !== appMenuToken) return; // a newer menu is on its way
    await (await Menu.new({ items: submenus })).setAsAppMenu();
  } catch (err) {
    console.warn("couldn't set the menu bar", err);
  }
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

/** Mirrors overall download progress on the taskbar button (Windows) or Dock icon (macOS). */
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
