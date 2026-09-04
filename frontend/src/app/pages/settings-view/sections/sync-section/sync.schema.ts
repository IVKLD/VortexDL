import { required, schema, validate } from '@angular/forms/signals';
import { WebDavSettings } from '../../models/settings-form.model';

export const webdavSchema = schema<WebDavSettings>((f) => {
    required(f.baseUrl, { message: 'Server URL is required' });
    validate(f.baseUrl, (ctx) => {
        const val = ctx.value();
        if (val && !/^https?:\/\/.+$/i.test(val)) {
            return { kind: 'pattern', message: 'Invalid WebDAV URL (e.g. https://webdav.pcloud.com)' };
        }
        return null;
    });

    required(f.username, { message: 'Username is required' });
    required(f.password, { message: 'Password is required' });
});

export const localPathSchema = schema<string>((f) => {
    required(f, { message: 'Path is required' });
});

export const restoreUrlSchema = schema<string>((f) => {
    required(f, { message: 'URL is required' });
});
