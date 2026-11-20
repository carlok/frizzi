# Frizzi

Applicazione passwordless per organizzare eventi e raccogliere disponibilità. Il frontend usa SvelteKit, TypeScript, Tailwind e Lucide. L’API usa Rust e Axum, con PostgreSQL per i dati e Mailpit per le email di sviluppo.

L'organizzatore crea l'evento e riceve il link privato via email. Gli invitati vedono le risposte pubbliche, votano e ricevono il proprio link di modifica. L'organizzatore può modificare i dettagli, confermare una data, annullare o riaprire il sondaggio. Gli aggiornamenti inviano notifiche localizzate con le modifiche precedenti e successive. Email e token privati non sono inclusi nell'API pubblica.

## Struttura del progetto

| Percorso | Contenuto |
| --- | --- |
| `src/routes/` e `src/lib/` | Pagine, componenti e logica frontend |
| `src/*.rs` e `src/mail/` | API, email, tracciamento e manutenzione |
| `api/index.rs` | Ingresso dell’API per Vercel |
| `migrations/` | Migrazioni PostgreSQL |
| `locales/` | Dizionari di interfaccia ed email |
| `tests/` | Test di integrazione e browser |

## Avvio con Docker

Servono Node.js 22.12+ (o 24+), npm, Rust stabile e Docker Compose.

```sh
test -f .env || install -m 600 .env.example .env
npm ci
docker compose up -d --wait
cargo run --bin migrate
npm run dev:full
```

Aprire http://localhost:5173. La UI Mailpit è su http://localhost:8025. L'API Rust ascolta su `127.0.0.1:3001`; Vite inoltra `/api`. `cargo run --bin migrate` applica le migrazioni SQLx senza dover installare un altro strumento. In alternativa si può usare `sqlx migrate run` dopo aver installato `sqlx-cli`.

`npm run dev:full` avvia backend e frontend insieme. Per lo sviluppo separato: `cargo run --bin local` e `npm run dev`. Il frontend usa il ricaricamento automatico; riavviare il backend dopo modifiche Rust, oppure usare `cargo watch -x 'run --bin local'`.

## Avvio senza privilegi amministrativi

Se l'ambiente non consente Docker, si possono usare gli stessi servizi come processi utente:

```sh
test -f .env || install -m 600 .env.example .env
npm ci
npm run services:native
```

Lo script usa PostgreSQL 16, installato tramite `embedded-postgres`, e Mailpit, scaricato dalla release ufficiale. La versione del database resta PostgreSQL reale. I dati locali persistono in `~/.cache/frizzi/postgres-16`.

Configurare `.env` con queste porte e mantenere `APP_URL` e `MAIL_FROM` come nell’esempio.

```dotenv
DATABASE_URL=postgres://frizzi:frizzi@localhost:55432/frizzi
SMTP_HOST=localhost
SMTP_PORT=11025
```

In un altro terminale eseguire `cargo run --bin migrate` e `npm run dev:full`. Mailpit è su http://localhost:18025.

## Lingue

Sono presenti italiano, inglese, spagnolo, cinese semplificato, hindi, arabo, francese, bengalese, portoghese, russo e tedesco. La lingua predefinita è italiano; il selettore persiste la scelta nel browser e imposta RTL per l'arabo. Le operazioni di organizzatori e partecipanti salvano la lingua nel database.

I dizionari JSON in `locales/` sono condivisi da `svelte-i18n` e `rust-i18n`. Date e orari usano `Intl.DateTimeFormat` nel browser e `chrono` con locale nelle email. I contenuti inseriti dagli utenti restano nella lingua originale. Le traduzioni sono redatte per questa applicazione; non sono una revisione linguistica professionale.

## Autocompletamento del luogo

Impostare `GEOAPIFY_API_KEY` nel file `.env` locale o nelle variabili server di Vercel.
La chiave è usata lato server. Il backend limita le ricerche a 120 per ora per istanza
del database; tenere conto anche degli altri progetti che usano la stessa quota.
Il campo continua ad accettare indirizzi, nomi e link scritti a mano anche
in assenza di una chiave o se il servizio di ricerca non risponde.

## Consegna delle email

Le operazioni salvano messaggi e dati nella stessa transazione. Dopo il commit tentano la consegna. Un errore SMTP o Resend lascia il messaggio in coda, con ritentativi progressivi. In locale un worker controlla la coda ogni dieci secondi. Su Vercel il cron incluso ritenta ogni giorno alle 06:00 UTC; per ritentativi più frequenti, impostare uno scheduler compatibile con il proprio piano.

L'endpoint `GET /api/cron/email` richiede `Authorization: Bearer <CRON_SECRET>`. Sono elaborati al massimo 25 messaggi per chiamata. Resend riceve una chiave di idempotenza per ridurre i duplicati. SMTP offre consegna almeno una volta: in caso di arresto fra invio e registrazione dell'esito può esserci un duplicato. Gli indirizzi e i link nei messaggi consegnati vengono rimossi dalla coda.

Modificare una data o un orario cancella soltanto le disponibilità su quel giorno e riapre il sondaggio; cambiare fuso orario invalida tutte le disponibilità. I voti sulle date rimaste uguali vengono conservati. Se qualcuno ha già scelto più date, non è possibile passare alla scelta singola senza prima aggiornare le risposte.

## Configurazione locale

Il file [`.env.example`](.env.example) contiene i valori di sviluppo e i nomi delle variabili lato server. Conservare i valori reali in `.env` con permessi `0600` o nelle variabili protette dell’hosting. Il database e la casella di test di Docker sono esposti solo su loopback.

Per abilitare gli allegati PDF, impostare `FRIZZI_TYPST` e `FRIZZI_PDF_FONTS`. Lo [script di installazione](deploy/setup-invitation-pdf.py) prepara il binario e i font su Linux x86-64. In assenza di Typst, la conferma viene inviata in HTML e testo.

## Verifiche

```sh
npm run test:setup
npm run check
npm run build
cargo test --lib
npm test
npm audit
```

Su un sistema Linux senza le librerie richieste dal browser, installarle con `npx playwright install-deps chromium`. Il test usa anche le eventuali librerie locali già preparate in `~/.cache/frizzi/browser-libs`.

`npm test` crea un database PostgreSQL 16 temporaneo e avvia Mailpit, Rust e Vite su porte dedicate. I test controllano creazione, privacy, autorizzazione, isolamento fra eventi, validazione, unicità delle email, voto, modifica dei dettagli, conservazione dei voti, notifiche localizzate e ritentativi dopo un guasto SMTP. I controlli browser attraversano le undici lingue, RTL, persistenza, creazione, voto, modifica, conferma e layout mobile. Screenshot in `test-results/`. Le email di test restano locali: la chiave Resend non viene utilizzata.

## Conservazione

I sondaggi restano disponibili fino al termine del decimo giorno successivo alla data dell’evento nel suo fuso orario. Senza una data confermata si usa l’ultima proposta. La manutenzione elimina i dati scaduti e ruota i conteggi aggregati d’uso. L’API emette log JSON e, con `FRIZZI_LOG_DIR`, usa file giornalieri con rotazione locale.
