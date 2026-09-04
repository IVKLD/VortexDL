import { Injectable, inject, signal } from '@angular/core';
import { apply, form } from '@angular/forms/signals';
import { SettingsService } from './settings.service';
import { firstValueFrom } from 'rxjs';
import { SettingsFormModel } from './models/settings-form.model';
import { soundcloudSchema } from './sections/soundcloud-section/soundcloud.schema';
import { downloadsSchema, systemSchema } from './sections/downloads-section/downloads.schema';
import { networkSchema } from './sections/network-section/network.schema';
import { webdavSchema } from './sections/sync-section/sync.schema';

@Injectable({ providedIn: 'root' })
export class SettingsState {
    private readonly _api = inject(SettingsService);

    public readonly settingsModel = signal<SettingsFormModel>({
        soundcloud: { profileUrl: '', syncInterval: 60, autoSync: true },
        downloads: { outputPath: './downloads', maxConcurrent: 3, namingTemplate: '{artist} - {title}' },
        system: { limitPerPage: 100, maxRetries: 5 },
        network: { useProxy: false, proxyUrl: '', fallbackProxies: [] },
        adb: { enabled: false, autoSync: false, devices: [] },
        webdav: { baseUrl: '', remoteDir: 'vortexdl', username: '', password: '' },
    });

    public readonly settingsForm = form(this.settingsModel, (f) => {
        apply(f.soundcloud, soundcloudSchema);
        apply(f.downloads, downloadsSchema);
        apply(f.system, systemSchema);
        apply(f.network, networkSchema);
        apply(f.webdav, webdavSchema);
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
                    adb: res.adb,
                    system: res.system,
                    webdav: res.webdav,
                });
            },
        });
    }
}
