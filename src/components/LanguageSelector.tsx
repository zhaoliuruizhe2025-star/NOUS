import type { Locale } from "../i18n";

interface LanguageSelectorProps {
  label: string;
  locale: Locale;
  languageNames: Record<Locale, string>;
  onChange: (locale: Locale) => void;
}

export function LanguageSelector({
  label,
  locale,
  languageNames,
  onChange,
}: LanguageSelectorProps) {
  return (
    <label className="language-selector">
      <span>{label}</span>
      <select
        aria-label={label}
        value={locale}
        onChange={(event) => onChange(event.target.value as Locale)}
      >
        <option value="en">{languageNames.en}</option>
        <option value="zh-CN">{languageNames["zh-CN"]}</option>
      </select>
    </label>
  );
}

