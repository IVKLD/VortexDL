import { ChangeDetectionStrategy, Component, inject, OnInit, signal } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { form, FormRoot, max, min, required } from '@angular/forms/signals';
import { SettingsService } from './settings.service';
import { MatButton } from "@angular/material/button";
import { finalize } from "rxjs";
import { englishOnly, soundCloudUrl } from "@shared/validators/form.validators";
import { SoundcloudSectionComponent } from "./components/soundcloud-section/soundcloud-section.component";
import { DownloadsSectionComponent } from "./components/downloads-section/downloads-section.component";
import { AdbSectionComponent } from "./components/adb-section/adb-section.component";
import { NetworkSettingsComponent } from "./components/network-section/network-section.component";
import { UserSettingsDto } from "@app/pages/settings-view/models/user-settings.dto";
import { SettingsFormModel, SyncMode } from "@app/pages/settings-view/models/settings-form.model";
import { UserSettingsRdo } from "@app/pages/settings-view/models/user-settings.rdo";

@Component({
    selector: 'app-settings-view',
    imports: [
        MatIcon, FormRoot, MatButton,
        SoundcloudSectionComponent, DownloadsSectionComponent, AdbSectionComponent,
        NetworkSettingsComponent
    ],
    templateUrl: './settings-view.component.html',
    styleUrl: './settings-view.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsView implements OnInit {
    protected readonly isTesting = signal(false);
    protected readonly isNetworkTesting = signal(false);
    protected readonly settingsModel = signal<SettingsFormModel>({
        soundcloud: {
            profileUrl: '',
            syncInterval: 60,
            autoSync: true
        },
        downloads: {
            outputPath: './downloads',
            maxConcurrent: 3,
            namingTemplate: '{artist} - {title}',
            syncMode: SyncMode.Silent
        },
        system: {
            limitPerPage: 100,
            maxRetries: 5
        },
        network: {
            useProxy: false,
            proxyUrl: '',
            fallbackProxies: []
        },
        adb: {
            enabled: true,
            autoSync: true,
            devices: []
        }
    });
    private readonly _api = inject(SettingsService);
    protected readonly settingsForm =
        form(this.settingsModel, (f) => {
            required(f.soundcloud.profileUrl, { message: 'Profile URL is required' });
            englishOnly(f.soundcloud.profileUrl);
            soundCloudUrl(f.soundcloud.profileUrl);

            min(f.soundcloud.syncInterval, 1, { message: 'Interval must be at least 1 minute' });
            max(f.soundcloud.syncInterval, 1440, { message: 'Interval cannot exceed 24 hours' });

            required(f.downloads.outputPath, { message: 'Output path is required' });
            min(f.downloads.maxConcurrent, 1, { message: 'Must have at least 1 concurrent download' });
            max(f.downloads.maxConcurrent, 10, { message: 'Maximum 10 concurrent downloads allowed' });
            required(f.downloads.namingTemplate, { message: 'Naming template is required' });

            min(f.system.limitPerPage, 1, { message: 'Limit must be at least 1' });
            max(f.system.limitPerPage, 500, { message: 'Limit cannot exceed 500' });
            min(f.system.maxRetries, 0, { message: 'Max retries cannot be negative' });
            max(f.system.maxRetries, 20, { message: 'Max retries cannot exceed 20' });
        },
            {
                submission: {
                    action: async () => {
                        const val = this.settingsForm().value();
                        const payload: UserSettingsDto = {
                            soundcloud: val.soundcloud,
                            downloads: val.downloads,
                            adb: val.adb,
                            network: {
                                useProxy: val.network.useProxy,
                                proxyUrl: val.network.proxyUrl,
                                fallbackProxies: val.network.fallbackProxies
                            },
                            limitPerPage: val.system.limitPerPage,
                            maxRetries: val.system.maxRetries
                        };

                        this._api.updateSettings(payload).subscribe()
                    }
                },
            });

    public ngOnInit() {
        this._api.getSettings()
            .subscribe({
                next: (res: UserSettingsRdo) => {
                    this.settingsForm().reset({
                        soundcloud: res.soundcloud,
                        downloads: res.downloads,
                        network: {
                            useProxy: res.network?.useProxy ?? false,
                            proxyUrl: res.network?.proxyUrl ?? '',
                            fallbackProxies: res.network?.fallbackProxies ?? []
                        },
                        adb: res.adb ?? { enabled: true, autoSync: true, devices: [] },
                        system: {
                            limitPerPage: res.limitPerPage,
                            maxRetries: res.maxRetries
                        }
                    });
                }
            });
    }

    protected testSoundcloudUrl() {
        this.isTesting.set(true);

        this._api.testSoundCloudUrl(this.settingsForm.soundcloud.profileUrl().value())
            .pipe(finalize(() => this.isTesting.set(false)))
            .subscribe();
    }

    protected testProxy() {
        this.isNetworkTesting.set(true);

        this._api.testProxy(this.settingsForm.network.proxyUrl().value())
            .pipe(finalize(() => this.isNetworkTesting.set(false)))
            .subscribe();
    }

}
