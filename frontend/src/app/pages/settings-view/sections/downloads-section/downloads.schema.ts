import { max, min, required, schema, validate } from '@angular/forms/signals';
import { DownloadSettings, SystemSettings } from '../../models/settings-form.model';
import { namingTemplateValidator } from '../../settings.validators';

export const downloadsSchema = schema<DownloadSettings>((f) => {
    required(f.outputPath, { message: 'Output path is required' });
    min(f.maxConcurrent, 1, { message: 'Must have at least 1 concurrent download' });
    max(f.maxConcurrent, 100, { message: 'Maximum 10 concurrent downloads allowed' });
    required(f.namingTemplate, { message: 'Naming template is required' });
    validate(f.namingTemplate, namingTemplateValidator);
});

export const systemSchema = schema<SystemSettings>((f) => {
    min(f.limitPerPage, 1, { message: 'Limit must be at least 1' });
    max(f.limitPerPage, 500, { message: 'Limit cannot exceed 500' });
    min(f.maxRetries, 0, { message: 'Max retries cannot be negative' });
    max(f.maxRetries, 20, { message: 'Max retries cannot exceed 20' });
});
