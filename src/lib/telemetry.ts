import { resolve } from '$app/paths';

type Interaction =
    | 'vote_calendar' | 'vote_list' | 'vote_both'
    | 'location_suggested' | 'location_manual'
    | 'share_native' | 'share_copy'
    | 'create_duplicate_date' | 'create_time_range' | 'create_timezone'
    | 'browser_network' | 'browser_invalid_response' | 'browser_clipboard' | 'browser_runtime';

export function track(action: Interaction) {
    void fetch(resolve('api/telemetry'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(action),
        credentials: 'omit',
        referrerPolicy: 'no-referrer',
        keepalive: true
    }).catch(() => {});
}
