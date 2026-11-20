<script lang="ts">
 import { t } from '#lib/i18n.ts';
</script>

<svelte:head>
 <title>{$t.privacyTitle} · Frizzi</title>
</svelte:head>

<article class="form-shell">
 <h1>{$t.privacyTitle}</h1>
 <p><strong>{$t.privacyCommercial}</strong></p>
 <p>{$t.privacyUse}</p>
 <p>{$t.privacyCookies}</p>
 <p>{$t.privacyRetention}</p>
 <p>{$t.privacyTelemetry}</p>
 <p>{$t.privacyProviders}</p>
</article>

<style>
 h1 { font-size: clamp(1.5rem, 4vw, 2rem); margin: 0 0 0.75rem; }
 p { margin: 0.75rem 0; }
</style>
