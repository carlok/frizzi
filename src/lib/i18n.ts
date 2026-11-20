import { addMessages, init, locale, format } from 'svelte-i18n';
import { derived } from 'svelte/store';
import it from '../../locales/it.json';
import en from '../../locales/en.json';
import es from '../../locales/es.json';
import zh from '../../locales/zh.json';
import hi from '../../locales/hi.json';
import ar from '../../locales/ar.json';
import fr from '../../locales/fr.json';
import bn from '../../locales/bn.json';
import pt from '../../locales/pt.json';
import ru from '../../locales/ru.json';
import de from '../../locales/de.json';
export const languages: Record<string,string> = { it:'Italiano', en:'English', es:'Español', zh:'中文', hi:'हिन्दी', ar:'العربية', fr:'Français', bn:'বাংলা', pt:'Português', ru:'Русский', de:'Deutsch' };
for (const [code, messages] of Object.entries({it,en,es,zh,hi,ar,fr,bn,pt,ru,de})) addMessages(code,messages);
init({ fallbackLocale:'it', initialLocale:'it' });
export const lang = locale;
export const t = derived(format, $format => Object.fromEntries(Object.keys(it.ui).map(key=>[key,$format(`ui.${key}`)])) as typeof it.ui);
export function errorText(code: string, messages: typeof it.ui): string {
    const keys: Record<string,keyof typeof it.ui> = { invalid_input:'invalidInput', email_exists:'emailExists', not_found:'notFound', multiple_votes_exist:'multipleVotesExist', rate_limited:'rateLimited', event_closed:'eventClosed', invalid_deadline:'invalidDeadline', invalid_dates:'invalidDates', deadline_passed:'deadlinePassed', tiebreak_unavailable:'tiebreakUnavailable', service_unavailable:'serviceUnavailable' };
    const [kind, reference] = code.split(':', 2);
    const message = messages[keys[kind] || 'error'];
    return reference && /^[a-f0-9-]{36}$/.test(reference) ? `${message} ${messages.supportCode} ${reference}` : message;
}
const dateLocales: Record<string, string> = {
    it: 'it-IT', en: 'en-GB', es: 'es-ES', zh: 'zh-CN', hi: 'hi-IN', ar: 'ar-SA',
    fr: 'fr-FR', bn: 'bn-BD', pt: 'pt-PT', ru: 'ru-RU', de: 'de-DE'
};
export function dateFormatter(language: string | null | undefined, withWeekday = false) {
    return new Intl.DateTimeFormat(dateLocales[language ?? ''] ?? dateLocales.it, {
        weekday: withWeekday ? 'long' : undefined,
        day: '2-digit', month: '2-digit', year: 'numeric', timeZone: 'UTC', calendar: 'gregory'
    });
}
export function formatDate(date: string, language: string | null | undefined) {
    const day = new Date(`${date}T12:00:00Z`);
    return `${dateFormatter(language, true).formatToParts(day).find(part => part.type === 'weekday')?.value} · ${dateFormatter(language).format(day)}`;
}
export function formatTime(time: string, locale: string | null | undefined) {
    return new Intl.DateTimeFormat(locale || 'it',{hour:'numeric',minute:'2-digit',timeZone:'UTC'}).format(new Date(`2000-01-01T${time}Z`));
}
