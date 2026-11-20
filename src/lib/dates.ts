export const fieldSizes = { day: 2, month: 2, year: 4 };
export type DatePart = keyof typeof fieldSizes;

/** Day, month and year in the order a formatter prints them. */
export function dateOrder(parts: Intl.DateTimeFormatPart[]): DatePart[] {
    return parts.filter((part): part is Intl.DateTimeFormatPart & {type: DatePart} =>
        part.type === 'day' || part.type === 'month' || part.type === 'year').map(part => part.type);
}

/**
 * Reads a typed date in the field order of the user's locale and returns it as
 * `YYYY-MM-DD`, or `''` when it is not a real calendar date. ISO dates are always
 * accepted, eight digits without separators are split by field size, and native
 * digits (Arabic-Indic, Persian, Bengali, Devanagari) are read as ASCII.
 */
export function readDate(raw: string, order: DatePart[]): string {
    const normalized = raw.trim().replace(/[‎‏؜]/g, '').replace(/[٠-٩۰-۹০-৯०-९]/g, digit => {
        const start = ['٠','۰','০','०'].find(zero => digit >= zero && digit <= String.fromCharCode(zero.charCodeAt(0) + 9))!;
        return String(digit.charCodeAt(0) - start.charCodeAt(0));
    });
    let iso = normalized;
    if (!/^\d{4}-\d{2}-\d{2}$/.test(iso)) {
        let offset = 0;
        const parts = /^\d{8}$/.test(normalized)
            ? order.map(part => {
                const partValue = normalized.slice(offset, offset + fieldSizes[part]);
                offset += fieldSizes[part];
                return partValue;
            })
            : normalized.split(/[\/\.\s-]/);
        if (parts.length !== 3 || parts.some((part, i) =>
            !/^\d+$/.test(part) || (order[i] === 'year' ? part.length !== 4 : part.length > 2))) return '';
        const [year, month, day] = (['year','month','day'] as const).map(part =>
            parts[order.indexOf(part)].padStart(fieldSizes[part], '0'));
        iso = `${year}-${month}-${day}`;
    }
    if (iso.startsWith('0000-')) return '';
    const date = new Date(`${iso}T12:00:00Z`);
    return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === iso ? iso : '';
}
