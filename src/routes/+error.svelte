<script lang="ts">
 import { resolve } from '$app/paths';
 import { page } from '$app/state';
 import { t } from '#lib/i18n.ts';
 $: missing=page.status===404;
</script>

<svelte:head><title>{missing?$t.pageNotFound:$t.errorTitle} · Frizzi</title></svelte:head>

<section class="center-card">
 <h1>{missing?$t.pageNotFound:$t.errorTitle}</h1>
 <p>{missing?$t.notFound:$t.serviceUnavailable}</p>
 <a href={resolve('')}>{$t.back}</a>
</section>
