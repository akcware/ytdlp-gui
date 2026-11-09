// Tiny i18n: English + Turkish, no dependencies. `t()` reads reactive state,
// so anything rendered with it updates when the language changes.
import { en } from "./locales/en";
import { tr } from "./locales/tr";

export type Locale = "en" | "tr";
export type LanguagePreference = "system" | Locale;
type Key = keyof typeof en;
type PluralBase = { [K in Key]: K extends `${infer B}_one` ? B : never }[Key];

const dictionaries: Record<Locale, Record<Key, string>> = { en, tr };
const STORAGE_KEY = "language";

/** Windows display language, as reported by WebView2. */
function systemLocale(): Locale {
  const lang = typeof navigator === "undefined" ? "" : (navigator.languages?.[0] ?? navigator.language);
  return lang?.toLowerCase().startsWith("tr") ? "tr" : "en";
}

function loadPreference(): LanguagePreference {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === "en" || v === "tr" ? v : "system";
  } catch {
    return "system";
  }
}

class I18n {
  preference = $state<LanguagePreference>(loadPreference());
  locale = $derived<Locale>(this.preference === "system" ? systemLocale() : this.preference);
  /** BCP 47 tag for Intl formatters. */
  tag = $derived(this.locale === "tr" ? "tr-TR" : "en-US");

  setPreference(p: LanguagePreference) {
    this.preference = p;
    try {
      if (p === "system") localStorage.removeItem(STORAGE_KEY);
      else localStorage.setItem(STORAGE_KEY, p);
    } catch {}
  }
}

export const i18n = new I18n();

function fill(text: string, params?: Record<string, string | number>) {
  return params ? text.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? "")) : text;
}

/** Translates `key`, replacing `{name}` placeholders with `params`. */
export function t(key: Key, params?: Record<string, string | number>): string {
  return fill(dictionaries[i18n.locale][key] ?? en[key] ?? key, params);
}

/** Plural form: picks `<base>_one` for 1 and `<base>_other` otherwise; `{n}` is filled in. */
export function tp(base: PluralBase, n: number, params?: Record<string, string | number>): string {
  const key = (n === 1 ? `${base}_one` : `${base}_other`) as Key;
  return t(key, { n, ...params });
}
