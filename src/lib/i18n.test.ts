import { afterEach, describe, expect, it } from 'vitest';
import { get } from 'svelte/store';
import { dateFormatter, errorText, formatDate, formatTime, intlLocale, lang, languages, t } from './i18n.ts';
import it_ from '../../locales/it.json';
import en from '../../locales/en.json';

const ui = it_.ui;
const reference = '0f8fad5b-d9cb-41a0-a1b2-0c1d2e3f4a5b';

describe('errorText', () => {
    it('maps every API error code to its message', () => {
        const codes: Record<string, keyof typeof ui> = {
            invalid_input: 'invalidInput', email_exists: 'emailExists', not_found: 'notFound',
            multiple_votes_exist: 'multipleVotesExist', rate_limited: 'rateLimited', event_closed: 'eventClosed',
            invalid_deadline: 'invalidDeadline', deadline_passed: 'deadlinePassed',
            tiebreak_unavailable: 'tiebreakUnavailable', service_unavailable: 'serviceUnavailable'
        };
        for (const [code, key] of Object.entries(codes)) expect(errorText(code, ui), code).toBe(ui[key]);
    });

    it('falls back to the generic message for unknown codes', () => {
        expect(errorText('error', ui)).toBe(ui.error);
        expect(errorText('something_new', ui)).toBe(ui.error);
        expect(errorText('', ui)).toBe(ui.error);
    });

    it('ignores codes inherited from Object.prototype', () => {
        for (const code of ['toString', 'constructor', '__proto__', 'hasOwnProperty', 'valueOf'])
            expect(errorText(code, ui), code).toBe(ui.error);
    });

    it('has a message for a failed copy to the clipboard', () => {
        expect(errorText('copy_failed', ui)).toBe(ui.copyFailed);
    });

    it('appends a support code when the reference is a request id', () => {
        expect(errorText(`service_unavailable:${reference}`, ui))
            .toBe(`${ui.serviceUnavailable} ${ui.supportCode} ${reference}`);
        expect(errorText(`not_found:${reference}`, ui)).toBe(`${ui.notFound} ${ui.supportCode} ${reference}`);
        expect(errorText(`unknown:${reference}`, ui)).toBe(`${ui.error} ${ui.supportCode} ${reference}`);
    });

    it('drops references that are not request ids', () => {
        expect(errorText('service_unavailable:abc', ui)).toBe(ui.serviceUnavailable);
        expect(errorText(`service_unavailable:${reference.toUpperCase()}`, ui)).toBe(ui.serviceUnavailable);
        expect(errorText(`service_unavailable:${reference}0`, ui)).toBe(ui.serviceUnavailable);
        expect(errorText(`service_unavailable:${reference}:extra`, ui)).toBe(`${ui.serviceUnavailable} ${ui.supportCode} ${reference}`);
    });

    it('uses the messages it is given', () => {
        expect(errorText('not_found', en.ui)).toBe(en.ui.notFound);
    });
});

describe('t', () => {
    afterEach(() => { lang.set('it'); });

    it('starts in Italian with every UI key', () => {
        const messages = get(t);
        expect(get(lang)).toBe('it');
        expect(Object.keys(messages).sort()).toEqual(Object.keys(ui).sort());
        expect(messages.notFound).toBe(ui.notFound);
    });

    it('follows the selected language', () => {
        lang.set('en');
        expect(get(t).notFound).toBe(en.ui.notFound);
    });

    it('lists the eleven supported languages', () => {
        expect(Object.keys(languages)).toEqual(['it', 'en', 'es', 'zh', 'hi', 'ar', 'fr', 'bn', 'pt', 'ru', 'de']);
    });
});

describe('date and time formatting', () => {
    it('formats dates in UTC with the Gregorian calendar', () => {
        expect(dateFormatter('it').format(new Date('2031-04-27T00:30:00Z'))).toBe('27/04/2031');
        expect(dateFormatter('de').format(new Date('2031-04-27T23:30:00Z'))).toBe('27.04.2031');
        expect(dateFormatter('zh').format(new Date('2031-04-27T12:00:00Z'))).toBe('2031/04/27');
        expect(dateFormatter('ar').resolvedOptions().calendar).toBe('gregory');
    });

    it('falls back to Italian for unknown or missing languages', () => {
        const italian = dateFormatter('it').resolvedOptions().locale;
        expect(dateFormatter('xx').resolvedOptions().locale).toBe(italian);
        expect(dateFormatter(null).resolvedOptions().locale).toBe(italian);
        expect(dateFormatter(undefined).resolvedOptions().locale).toBe(italian);
    });

    it('prefixes the weekday', () => {
        expect(formatDate('2031-04-27', 'it')).toBe('domenica · 27/04/2031');
        expect(formatDate('2031-04-27', 'en')).toBe('Sunday · 27/04/2031');
    });

    it('formats times without shifting them', () => {
        expect(formatTime('18:30', 'it')).toBe('18:30');
        expect(formatTime('09:05:00', 'it')).toBe('9:05');
        expect(formatTime('18:30', null)).toBe('18:30');
    });

    it('formats times with the same regional locale as dates', () => {
        expect(intlLocale('en')).toBe('en-GB');
        expect(intlLocale('pt')).toBe('pt-PT');
        expect(intlLocale('xx')).toBe('it-IT');
        expect(intlLocale('toString')).toBe('it-IT');
        expect(formatTime('18:30', 'en')).toBe('18:30');
    });
});
