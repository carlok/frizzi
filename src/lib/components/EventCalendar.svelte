<script lang="ts">
 import { ChevronLeft, ChevronRight } from '@lucide/svelte';
 import { formatDate, formatTime, intlLocale, lang, t } from '#lib/i18n.ts';
 import { firstWeekday } from '#lib/dates.ts';
 import type { Slot } from '#lib/types.ts';

 export let slots: Slot[];
 export let selected: string[] = [];
 export let confirmedId: string | null = null;
 export let month = '';
 export let interactive = true;
 export let busy = false;
 export let choose: (id: string) => void;
 /** When the same dates are listed as controls below, the calendar is a pointer shortcut only:
  *  hidden from assistive technology and out of the tab order, so each date is announced once. */
 export let mirrored = false;
 /** Legend for the marked days: the person's choices, or the date the organizer is looking at. */
 export let markedLabel = '';

 function monthDays(month: string, firstDay: number) {
   const date = new Date(`${month}-01T12:00:00Z`);
   const leading = (date.getUTCDay() - firstDay + 7) % 7;
   date.setUTCMonth(date.getUTCMonth() + 1, 0);
   const days = date.getUTCDate();
   return Array.from({length: Math.ceil((leading + days) / 7) * 7}, (_, index) => {
     const day = index - leading + 1;
     return day > 0 && day <= days ? `${month}-${String(day).padStart(2, '0')}` : null;
   });
 }

 $: language = $lang || 'it';
 $: locale = intlLocale(language);
 $: months = [...new Set(slots.map(slot => slot.date.slice(0, 7)))].sort();
 $: shownMonth = months.includes(month) ? month : months[0];
 $: monthIndex = months.indexOf(shownMonth);
 $: days = shownMonth ? monthDays(shownMonth, firstWeekday(locale)) : [];
 $: slotsByDay = new Map(slots.map(slot => [slot.date, slot]));
 $: weekdays = Array.from({length: 7}, (_, index) => {
   const weekday = new Date(Date.UTC(2024, 0, 7 + (firstWeekday(locale) + index) % 7));
   return new Intl.DateTimeFormat(locale, {weekday:'short', timeZone:'UTC'}).format(weekday);
 });
 $: monthTitle = shownMonth ? new Intl.DateTimeFormat(locale, {
   month:'long', year:'numeric', timeZone:'UTC', calendar:'gregory'
 }).format(new Date(`${shownMonth}-01T12:00:00Z`)) : '';
 $: numbers = new Intl.NumberFormat(locale);
</script>

{#if shownMonth}
 <section class="event-calendar" aria-label={mirrored ? undefined : $t.eventCalendar} aria-hidden={mirrored || undefined}>
  <div class="calendar-month">
   <button type="button" aria-label={$t.previousMonth} disabled={monthIndex<=0} tabindex={mirrored ? -1 : undefined}
     on:click={()=>month=months[monthIndex-1]}><ChevronLeft size={18} class="rtl:rotate-180" aria-hidden="true"/></button>
   <strong aria-live={mirrored ? undefined : 'polite'}>{monthTitle}</strong>
   <button type="button" aria-label={$t.nextMonth} disabled={monthIndex>=months.length-1} tabindex={mirrored ? -1 : undefined}
     on:click={()=>month=months[monthIndex+1]}><ChevronRight size={18} class="rtl:rotate-180" aria-hidden="true"/></button>
  </div>
  <div class="calendar-grid">
   {#each weekdays as weekday}
    <span class="calendar-weekday" aria-hidden="true">{weekday}</span>
   {/each}
   {#each days as day}
    {@const slot = day ? slotsByDay.get(day) : undefined}
    {@const pressed = slot ? selected.includes(slot.id) : false}
    {@const confirmed = slot?.id === confirmedId}
    {@const dayNumber = day ? numbers.format(Number(day.slice(-2))) : ''}
    {#if day && slot}
     {@const label = `${formatDate(day,language)} · ${formatTime(slot.start,language)}${slot.end ? ' – '+formatTime(slot.end,language) : ''}${confirmed ? ' · '+$t.confirmedDate : ''}`}
     {#if interactive}
      <button type="button" class="calendar-day proposed" class:marked={pressed} class:confirmed
        data-calendar-date={day} aria-label={label} aria-pressed={pressed} title={label} disabled={busy} tabindex={mirrored ? -1 : undefined}
        on:click={()=>choose(slot.id)}>{dayNumber}</button>
     {:else}
      <span class="calendar-day proposed" class:marked={pressed} class:confirmed
        data-calendar-date={day} title={label}>
       <span aria-hidden="true">{dayNumber}</span>
       <span class="sr-only">{label}{pressed ? ' · '+$t.selectedDates : ''}</span>
      </span>
     {/if}
    {:else}
     <span class="calendar-day calendar-empty" aria-hidden="true">{dayNumber}</span>
    {/if}
   {/each}
  </div>
  <ul class="calendar-legend">
   <li class="calendar-key proposed">{$t.proposedDates}</li>
   {#if interactive || selected.length}<li class="calendar-key marked">{markedLabel || $t.selectedDates}</li>{/if}
   {#if confirmedId}<li class="calendar-key confirmed">{$t.confirmedDate}</li>{/if}
  </ul>
 </section>
{/if}
