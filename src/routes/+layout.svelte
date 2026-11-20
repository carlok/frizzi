<script lang="ts">
 import { resolve } from '$app/paths';
 import '../style.css';
 import { onMount } from 'svelte';
 import { track } from '#lib/telemetry.ts';
 import { Sparkles, Globe } from '@lucide/svelte';
 import { lang, languages, t } from '#lib/i18n.ts';
 let ready=false;
 onMount(()=>{
    try { const saved=localStorage.getItem('locale'); if(saved && saved in languages) lang.set(saved); } catch { /* Browser privacy mode can disable storage. */ }
    let reported=false;
    const report=()=>{
      if(reported)return;
      reported=true;
      track('browser_runtime');
    };
    window.addEventListener('error',report);
    window.addEventListener('unhandledrejection',report);
    ready=true;
    return ()=>{
      window.removeEventListener('error',report);
      window.removeEventListener('unhandledrejection',report);
    };
 });
 $: if (ready && typeof document!=='undefined') {
    document.documentElement.lang=$lang || 'it';
    document.documentElement.dir=$lang==='ar'?'rtl':'ltr';
    try { localStorage.setItem('locale',$lang || 'it'); } catch { /* Keep the current language without persistence. */ }
 }
</script>
<svelte:head><title>Frizzi · {$t.tagline}</title><meta name="description" content={$t.intro}/></svelte:head>
<header data-ready={ready}>
 <a class="brand" href={resolve('')} aria-label="Frizzi"><span class="brand-icon"><Sparkles aria-hidden="true"/></span> frizzi<span class="brand-dot">.</span></a>
 <label class="language"><Globe size={18} aria-hidden="true"/><span class="sr-only">{$t.language}</span><select bind:value={$lang}>{#each Object.entries(languages) as [code,name]}<option value={code}>{name}</option>{/each}</select></label>
</header>
<main><slot/></main><footer>frizzi<span class="brand-dot">.</span><span class="footer-tagline">{$t.tagline}</span><a class="privacy-link" href={resolve('/privacy')}>{$t.privacyTitle}</a></footer>
