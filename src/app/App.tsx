import { useEffect, useMemo, useState } from "react";

import { LanguageSelector } from "../components/LanguageSelector";
import {
  getMessages,
  resolveInitialLocale,
  type Locale,
} from "../i18n";
import { initializeDatabase } from "./database";

type DatabaseStatus = "initializing" | "ready" | "error";

export function App() {
  const [locale, setLocale] = useState<Locale>(() =>
    resolveInitialLocale(navigator.language),
  );
  const [databaseStatus, setDatabaseStatus] =
    useState<DatabaseStatus>("initializing");
  const messages = useMemo(() => getMessages(locale), [locale]);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  useEffect(() => {
    let active = true;

    initializeDatabase()
      .then(() => {
        if (active) setDatabaseStatus("ready");
      })
      .catch(() => {
        if (active) setDatabaseStatus("error");
      });

    return () => {
      active = false;
    };
  }, []);

  return (
    <main className="app-shell">
      <header className="app-header">
        <span className="wordmark">{messages.appName}</span>
        <LanguageSelector
          label={messages.languageLabel}
          locale={locale}
          languageNames={messages.languages}
          onChange={setLocale}
        />
      </header>

      <section className="landing" aria-labelledby="landing-title">
        <p className="eyebrow">{messages.loop}</p>
        <h1 id="landing-title">{messages.placeholderTitle}</h1>
        <p className="placeholder-copy">{messages.placeholderBody}</p>
        <p className={`database-status database-status--${databaseStatus}`} role="status">
          <span aria-hidden="true" />
          {messages.database[databaseStatus]}
        </p>
      </section>
    </main>
  );
}

