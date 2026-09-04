import { FieldContext } from '@angular/forms/signals';

export const namingTemplateValidator = (ctx: FieldContext<string>) => {
    const val = ctx.value();
    if (!val) return null;

    const withoutValid = val.replaceAll('{artist}', '').replaceAll('{title}', '');

    if (withoutValid.includes('{') || withoutValid.includes('}')) {
        return {
            kind: 'invalidPlaceholder',
            message: 'Only {artist} and {title} are allowed. Remove any other { or } characters.',
        };
    }

    return null;
};
