import { AbstractControl, ValidationErrors, ValidatorFn } from '@angular/forms';

export function urlValidator(allowedProtocols: string[] = ['http:', 'https:']): ValidatorFn {
  return (control: AbstractControl): ValidationErrors | null => {
    const value = control.value;

    if (!value) {
      return null;
    }

    try {
      const url = new URL(value);

      if (allowedProtocols.length > 0 && !allowedProtocols.includes(url.protocol)) {
        return { invalidUrl: { reason: 'unsupported_protocol', protocol: url.protocol } };
      }

      return null;
    } catch {
      return { invalidUrl: true };
    }
  };
}
