<script lang="ts">
 import { CalendarDays } from '@lucide/svelte';
 import { dateFormatter, lang, t } from '#lib/i18n.ts';

 export let name: string;
 export let label: string;
 export let value = '';
 export let required = false;
 export let describedBy: string | undefined = undefined;

 const example = new Date('2031-04-27T12:00:00Z');
 const fieldSizes = { day: 2, month: 2, year: 4 };
 type DatePart = keyof typeof fieldSizes;
 let input: HTMLInputElement;
 let picker: HTMLInputElement;
 let draft = '';
 let syncedValue: string | undefined;
 let syncedFormatter: Intl.DateTimeFormat | undefined;

 $: formatter = dateFormatter($lang);
 $: dateParts = formatter.formatToParts(example);
 $: order = dateParts.filter((part): part is Intl.DateTimeFormatPart & {type: DatePart} =>
   part.type === 'day' || part.type === 'month' || part.type === 'year').map(part => part.type);
 $: placeholder = dateParts.map(part => {
   if (part.type === 'day') return 'DD';
   if (part.type === 'month') return 'MM';
   if (part.type === 'year') return 'YYYY';
   return part.value;
 }).join('');
 $: if (value !== syncedValue || formatter !== syncedFormatter) {
   draft = value ? formatter.format(new Date(`${value}T12:00:00Z`)) : '';
   syncedValue = value;
   syncedFormatter = formatter;
 }
 $: input?.setCustomValidity(draft.trim() && !value ? $t.invalidDate : '');

 function readDate(raw: string) {
   const normalized = raw.trim().replace(/[\u200e\u200f\u061c]/g, '').replace(/[٠-٩۰-۹০-৯०-९]/g, digit => {
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

 function setDraft(raw: string) {
   draft = raw;
   syncedValue = readDate(raw);
   value = syncedValue;
 }

 function usePicker(iso: string) {
   syncedValue = readDate(iso);
   value = syncedValue;
   draft = value ? formatter.format(new Date(`${value}T12:00:00Z`)) : '';
 }

 function showCalendar() {
   if (picker.showPicker) {
     try { picker.showPicker(); return; } catch { /* The text field stays available. */ }
   }
   input.focus();
 }
</script>

<label for={name}>{label}{required ? ' *' : ''}</label>
<div class="date-entry">
 <input id={name} {name} type="text" inputmode="numeric" autocomplete="off"
  {required} {placeholder} aria-describedby={describedBy} bind:this={input} value={draft}
  on:input={(event) => setDraft(event.currentTarget.value)}
  on:blur={() => { if (value) usePicker(value); }}/>
 <button type="button" class="date-picker-button" aria-label={`${label}: ${$t.eventCalendar}`} on:click={showCalendar}>
  <CalendarDays size={17} aria-hidden="true"/>
 </button>
 <input class="native-date" type="date" tabindex="-1" aria-hidden="true" min="0001-01-01" max="9999-12-31"
  value={value} bind:this={picker} on:change={(event) => usePicker(event.currentTarget.value)}/>
</div>

<style>
 .date-entry { position: relative; }
 .date-entry input[type="text"] { padding-inline-end: 42px; }
 .date-picker-button {
   position: absolute;
   inset-block: 1px;
   inset-inline-end: 1px;
   width: 38px;
   display: grid;
   place-items: center;
   border: 0;
   border-radius: 6px;
   color: #436f49;
   background: transparent;
   cursor: pointer;
 }
 .date-picker-button:hover { background: #edf4e8; }
 .date-entry .native-date {
   position: absolute;
   inset-inline-end: 0;
   inset-block-end: 0;
   margin: 0;
   width: 1px;
   height: 1px;
   padding: 0;
   opacity: 0;
   pointer-events: none;
 }
</style>
