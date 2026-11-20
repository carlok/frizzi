<script lang="ts">
 import { tick } from 'svelte';
 import { Plus, X } from '@lucide/svelte';
 import { formatDate, lang, t } from '#lib/i18n.ts';
 import LocationField from './LocationField.svelte';
 import TimeSelect from './TimeSelect.svelte';
 import DateField from './DateField.svelte';
 import { slotDraft } from '#lib/slots.ts';
 import type { LocationGeo, SlotDraft } from '#lib/types.ts';
 export let title=''; export let description=''; export let location='';
 export let locationGeo: LocationGeo | null = null;
 export let slots:SlotDraft[]=[slotDraft()];
 export let allow_multiple_choices=true; export let timezone='Europe/Rome';
 export let editing=false;
 export let votingDeadline='';
 let addButton: HTMLButtonElement;

 async function addDate() {
   const previous = slots.at(-1);
   let date = '';
   if (previous?.date) {
     // Date inputs hold calendar days. UTC keeps the increment independent of DST.
     const next = new Date(`${previous.date}T00:00:00Z`);
     next.setUTCDate(next.getUTCDate() + 1);
     if (Number.isFinite(next.getTime()) && next.getUTCFullYear() <= 9999) {
       date = next.toISOString().slice(0, 10);
     }
   }
   slots = [...slots, slotDraft({ date, start: previous?.start || '', end: previous?.end || '' })];
   await tick();
   document.getElementById(`date-${slots.length - 1}`)?.focus();
 }

 // Focus moves to the previous row's remove button, or to Add when a single row is left.
 async function removeDate(index: number) {
   slots = slots.filter((_, other) => other !== index);
   await tick();
   (slots.length > 1 && document.getElementById(`remove-${slots[Math.max(0, index - 1)].key}`) || addButton)?.focus();
 }
</script>
<div class="section-heading"><h2>{$t.detail}</h2><span class="required">* {$t.required}</span></div>
<div class="form-grid">
 <label>{$t.title} *<input name="title" required maxlength="255" bind:value={title} placeholder={$t.titleHint}/></label>
 <LocationField bind:location bind:locationGeo/>
 <label class="wide">{$t.description} <span>({$t.optional})</span><textarea name="description" maxlength="10000" bind:value={description} rows="2"></textarea></label>
</div>
<div class="divider"></div>
<div class="section-heading"><div><h2>{$t.dates}</h2><p>{editing?$t.datesChangedHint:$t.datesText}</p></div></div>
{#each slots as slot,i (slot.key)}<div class="slot-row">
 <div class="slot-date"><DateField name={`date-${i}`} label={$t.date} required bind:value={slot.date}/></div>
 <TimeSelect name={`start-${i}`} label={$t.start} required bind:value={slot.start}/>
 <TimeSelect name={`end-${i}`} label={$t.end} bind:value={slot.end}/>
 {#if slots.length>1}<button type="button" class="remove" id={`remove-${slot.key}`} aria-label={`${$t.remove} · ${slot.date?formatDate(slot.date,$lang):`${$t.date} ${i+1}`}`} on:click={()=>removeDate(i)}><X size={18} aria-hidden="true"/></button>{/if}
</div>{/each}
<button class="text-button icon-button" type="button" disabled={slots.length>=100} bind:this={addButton} on:click={addDate}><Plus size={17} aria-hidden="true"/>{$t.add}</button>
<label class="check"><input name="multiple" type="checkbox" bind:checked={allow_multiple_choices}/>{$t.multiple}</label>
<div class="form-grid">
 <label>{$t.timezone}<input name="timezone" required maxlength="64" bind:value={timezone}/></label>
 <div>
  <DateField name="voting_deadline" label={`${$t.votingDeadline} (${$t.optional})`} describedBy="deadline-hint" bind:value={votingDeadline}/>
  <small class="fine block mt-1" id="deadline-hint">{$t.deadlineHint}</small>
 </div>
</div>
