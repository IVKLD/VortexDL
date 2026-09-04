import { max, min, schema } from '@angular/forms/signals';
import { latinOnly, soundCloudUrl } from '@shared/validators/form.validators';
import { SoundcloudSettings } from '../../models/settings-form.model';

export const soundcloudSchema = schema<SoundcloudSettings>((f) => {
    latinOnly(f.profileUrl);
    soundCloudUrl(f.profileUrl);

    min(f.syncInterval, 1, { message: 'Interval must be at least 1 minute' });
    max(f.syncInterval, 1440, { message: 'Interval cannot exceed 24 hours' });
});
