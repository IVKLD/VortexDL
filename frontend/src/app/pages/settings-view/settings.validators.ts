import { FieldContext } from '@angular/forms/signals';

export const soundcloudUrlTestValidator = (getError: () => string | null) => {
    return (ctx: FieldContext<string>) => {
        const testErr = getError();
        return (testErr && !ctx.state.dirty()) ? { kind: 'testFailed', message: testErr } : null;
    };
};

export const proxyUrlValidator = (
    getError: () => string | null
) => {
    return (ctx: FieldContext<string>) => {
        const val = ctx.value();
        if (val) {
            const match = /^(socks5|http|https):\/\/[a-zA-Z0-9\-_.:@]+$/i.test(val);
            if (!match) {
                return { kind: 'pattern', message: 'Invalid proxy URL (e.g. socks5://127.0.0.1:1080)' };
            }
        }
        const testErr = getError();
        return (testErr && !ctx.state.dirty()) ? { kind: 'testFailed', message: testErr } : null;
    };
};

export const namingTemplateValidator = (ctx: FieldContext<string>) => {
    const val = ctx.value();
    if (!val) return null;

    const withoutValid = val.replaceAll('{artist}', '').replaceAll('{title}', '');
    
    if (withoutValid.includes('{') || withoutValid.includes('}')) {
        return {
            kind: 'invalidPlaceholder',
            message: 'Only {artist} and {title} are allowed. Remove any other { or } characters.'
        };
    }

    return null;
};
