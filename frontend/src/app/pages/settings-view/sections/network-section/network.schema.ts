import { schema, validate } from '@angular/forms/signals';
import { NetworkSettings } from '../../models/settings-form.model';

export const networkSchema = schema<NetworkSettings>((f) => {
    validate(f.proxyUrl, (ctx) => {
        const val = ctx.value();
        if (val && !/^(socks5|http|https):\/\/[a-zA-Z0-9\-_.:@]+$/i.test(val)) {
            return { kind: 'pattern', message: 'Invalid proxy URL (e.g. socks5://127.0.0.1:1080)' };
        }
        return null;
    });
});
