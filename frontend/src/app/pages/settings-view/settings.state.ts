import { Injectable, inject, signal } from '@angular/core';
import { form, max, min, required, validate } from '@angular/forms/signals';
import { SettingsService } from './settings.service';
import { firstValueFrom } from 'rxjs';
import { englishOnly, soundCloudUrl } from '@shared/validators/form.validators';
import { SettingsFormModel } from './models/settings-form.model';
import { namingTemplateValidator } from './settings.validators';

@Injectable({ providedIn: 'root' })
export class SettingsState {
    private readonly _api = inject(SettingsService);

    public readonly settingsModel = signal<SettingsFormModel>({
        soundcloud: { profileUrl: '', syncInterval: 60, autoSync: true },
        downloads: { outputPath: './downloads', maxConcurrent: 3, namingTemplate: '{artist} - {title}' },
        system: { limitPerPage: 100, maxRetries: 5 },
        network: { useProxy: false, proxyUrl: '', fallbackProxies: [] },
        adb: { enabled: true, autoSync: true, devices: [] },
    });

    public readonly settingsForm = form(this.settingsModel, (f) => {
        englishOnly(f.soundcloud.profileUrl);
        soundCloudUrl(f.soundcloud.profileUrl);

        min(f.soundcloud.syncInterval, 1, { message: 'Interval must be at least 1 minute' });
        max(f.soundcloud.syncInterval, 1440, { message: 'Interval cannot exceed 24 hours' });

        required(f.downloads.outputPath, { message: 'Output path is required' });
        min(f.downloads.maxConcurrent, 1, { message: 'Must have at least 1 concurrent download' });
        max(f.downloads.maxConcurrent, 100, { message: 'Maximum 10 concurrent downloads allowed' });
        required(f.downloads.namingTemplate, { message: 'Naming template is required' });
        validate(f.downloads.namingTemplate, namingTemplateValidator);

        min(f.system.limitPerPage, 1, { message: 'Limit must be at least 1' });
        max(f.system.limitPerPage, 500, { message: 'Limit cannot exceed 500' });
        min(f.system.maxRetries, 0, { message: 'Max retries cannot be negative' });
        max(f.system.maxRetries, 20, { message: 'Max retries cannot exceed 20' });

        validate(f.network.proxyUrl, (ctx) => {
            const val = ctx.value();
            if (val && !/^(socks5|http|https):\/\/[a-zA-Z0-9\-_.:@]+$/i.test(val)) {
                return { kind: 'pattern', message: 'Invalid proxy URL (e.g. socks5://127.0.0.1:1080)' };
            }
            return null;
        });
    }, {
        submission: {
            action: async () => {
                if (this.settingsForm().valid() && this.settingsForm().dirty()) {
                    const payload = this.settingsForm().value();
                    await firstValueFrom(this._api.updateSettings(payload));
                    this.settingsForm().reset(payload);
                }
                return [];
            }
        }
    });

    public loadSettings(): void {
        this._api.getSettings().subscribe({
            next: (res) => {
                this.settingsForm().reset({
                    soundcloud: res.soundcloud,
                    downloads: res.downloads,
                    network: res.network,
                    adb: res.adb ?? { enabled: true, autoSync: true, devices: [] },
                    system: res.system,
                });
            },
        });
    }
}
