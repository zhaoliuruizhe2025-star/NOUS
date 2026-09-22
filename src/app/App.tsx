import { useEffect, useMemo, useState } from "react";

import { LanguageSelector } from "../components/LanguageSelector";
import { StructuredCapture } from "../components/StructuredCapture";
import { StructuredHistory } from "../components/StructuredHistory";
import { DataPortability } from "../components/DataPortability";
import {
  getMessages,
  resolveInitialLocale,
  type Locale,
} from "../i18n";
import { initializeDatabase } from "./database";

type DatabaseStatus = "initializing" | "ready" | "error";
type AppView = "capture" | "history" | "portability";

export function App() {
  const [locale, setLocale] = useState<Locale>(() =>
    resolveInitialLocale(navigator.language),
  );
  const [databaseStatus, setDatabaseStatus] =
    useState<DatabaseStatus>("initializing");
  const [view, setView] = useState<AppView>("capture");
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

      {databaseStatus === "ready" ? (
        <>
          <nav className="view-switcher" aria-label={messages.navigation.label}>
            <button
              className={view === "capture" ? "view-switcher__active" : ""}
              type="button"
              aria-current={view === "capture" ? "page" : undefined}
              onClick={() => setView("capture")}
            >
              {messages.navigation.capture}
            </button>
            <button
              className={view === "history" ? "view-switcher__active" : ""}
              type="button"
              aria-current={view === "history" ? "page" : undefined}
              onClick={() => setView("history")}
            >
              {messages.navigation.history}
            </button>
            <button
              className={view === "portability" ? "view-switcher__active" : ""}
              type="button"
              aria-current={view === "portability" ? "page" : undefined}
              onClick={() => setView("portability")}
            >
              {messages.navigation.portability}
            </button>
          </nav>
          {view === "capture" ? (
            <StructuredCapture messages={messages} />
          ) : view === "history" ? (
            <StructuredHistory messages={messages} />
          ) : (
            <DataPortability messages={messages} />
          )}
        </>
      ) : (
        <section className="landing" aria-labelledby="landing-title">
          <p className="eyebrow">{messages.loop}</p>
          <h1 id="landing-title">{messages.placeholderTitle}</h1>
          <p className="placeholder-copy">{messages.placeholderBody}</p>
          <p
            className={`database-status database-status--${databaseStatus}`}
            role="status"
          >
            <span aria-hidden="true" />
            {messages.database[databaseStatus]}
          </p>
        </section>
      )}
    </main>
  );
}
