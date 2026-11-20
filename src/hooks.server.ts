import type { Handle, HandleServerError } from '@sveltejs/kit/hooks';
export const handle: Handle = async ({event,resolve})=>{
 const response=await resolve(event);
 response.headers.set('Referrer-Policy','no-referrer');
 response.headers.set('X-Content-Type-Options','nosniff');
 response.headers.set('X-Frame-Options','DENY');
 if(event.url.pathname!=='/')response.headers.set('Cache-Control','no-store');
 return response;
};

export const handleError: HandleServerError = async () => {
 const api = process.env.FRIZZI_INTERNAL_API;
 if (api) {
  const response = await fetch(new URL('/api/telemetry', api), {
   method: 'POST',
   headers: { 'Content-Type': 'application/json' },
   body: JSON.stringify('web_render'),
   signal: AbortSignal.timeout(500)
  }).catch(() => null);
  if (response?.ok) return;
 }
 console.error(JSON.stringify({ event: 'web_render_failed' }));
};
