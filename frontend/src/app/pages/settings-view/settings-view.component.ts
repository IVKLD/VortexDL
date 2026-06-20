import {ChangeDetectionStrategy, Component, inject, OnInit, signal, computed} from '@angular/core';
import {form, FormRoot, max, min, required} from '@angular/forms/signals';
import {SettingsService, SettingsTestingService} from './settings.service';
import {MatSnackBar} from '@angular/material/snack-bar';
import {finalize, firstValueFrom} from "rxjs";
import {parseErrorMessage} from '@shared/error-utils';
import {englishOnly, soundCloudUrl} from "@shared/validators/form.validators";
import {SoundcloudSectionComponent} from "./components/soundcloud-section/soundcloud-section.component";
import {DownloadsSectionComponent} from "./components/downloads-section/downloads-section.component";
import {AdbSectionComponent} from "./components/adb-section/adb-section.component";
import {NetworkSettingsComponent} from "./components/network-section/network-section.component";
import {UserSettingsDto} from "@app/pages/settings-view/models/user-settings.dto";
import {SettingsFormModel, SyncMode} from "@app/pages/settings-view/models/settings-form.model";
import {UserSettingsRdo} from "@app/pages/settings-view/models/user-settings.rdo";
import {HeaderTemplateDirective} from '@shared/components/bricks/header/header-template.directive';
import {SettingsSaveButtonComponent} from './components/settings-save-button.component';

@Component({
    selector: 'app-settings-view',
    imports: [
        FormRoot,
        SoundcloudSectionComponent, DownloadsSectionComponent, AdbSectionComponent,
        NetworkSettingsComponent, HeaderTemplateDirective, SettingsSaveButtonComponent
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
    private readonly _testing = inject(SettingsTestingService);
    private readonly _snack = inject(MatSnackBar);

    protected readonly saveButtonDisabled = computed(() => this.settingsForm().invalid() || this.isTesting() || this.isNetworkTesting() || !this.settingsForm().dirty());

    protected readonly settingsForm =
        form(this.settingsModel, (f) => {
                required(f.soundcloud.profileUrl, {message: 'Profile URL is required'});
                englishOnly(f.soundcloud.profileUrl);
                soundCloudUrl(f.soundcloud.profileUrl);

                min(f.soundcloud.syncInterval, 1, {message: 'Interval must be at least 1 minute'});
                max(f.soundcloud.syncInterval, 1440, {message: 'Interval cannot exceed 24 hours'});

                required(f.downloads.outputPath, {message: 'Output path is required'});
                min(f.downloads.maxConcurrent, 1, {message: 'Must have at least 1 concurrent download'});
                max(f.downloads.maxConcurrent, 100, {message: 'Maximum 10 concurrent downloads allowed'});
                required(f.downloads.namingTemplate, {message: 'Naming template is required'});

                min(f.system.limitPerPage, 1, {message: 'Limit must be at least 1'});
                max(f.system.limitPerPage, 500, {message: 'Limit cannot exceed 500'});
                min(f.system.maxRetries, 0, {message: 'Max retries cannot be negative'});
                max(f.system.maxRetries, 20, {message: 'Max retries cannot exceed 20'});
            },
            {
                submission: {
                    action: async () => {
                        const form = this.settingsForm().value();
                        const payload: UserSettingsDto = {
                            soundcloud: form.soundcloud,
                            downloads: form.downloads,
                            adb: form.adb,
                            network: {
                                useProxy: form.network.useProxy,
                                proxyUrl: form.network.proxyUrl,
                                fallbackProxies: form.network.fallbackProxies
                            },
                            limitPerPage: form.system.limitPerPage,
                            maxRetries: form.system.maxRetries
                        };

                        try {
                            await firstValueFrom(this._api.updateSettings(payload));

                            this.settingsForm().reset();
                        } catch (error) {
                            console.error('Ошибка при сохранении:', error);
                        }
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
                            useProxy: res.network?.useProxy,
                            proxyUrl: res.network?.proxyUrl,
                            fallbackProxies: res.network?.fallbackProxies
                        },
                        adb: res.adb ?? {enabled: true, autoSync: true, devices: []},
                        system: {
                            limitPerPage: res.limitPerPage,
                            maxRetries: res.maxRetries
                        }
                    });
                }
            });
    }

    protected testSoundcloud() {
        this.isTesting.set(true);

        this._testing.testSoundCloud(this.settingsForm.soundcloud.profileUrl().value())
            .pipe(finalize(() => this.isTesting.set(false)))
            .subscribe();
    }

    protected testProxy() {
        this.isNetworkTesting.set(true);
        const proxy = this.settingsForm.network.proxyUrl().value();

        this._testing.testProxy([proxy])
            .pipe(finalize(() => this.isNetworkTesting.set(false)))
            .subscribe({
                next: (res) => {
                    const result = res.results[0];
                    if (result && result.valid) {
                        this._snack.open('Proxy connection successful', 'OK');
                    } else {
                        const err = result?.error || 'Proxy is not able to reach SoundCloud API';
                        this._snack.open(err, 'Close');
                    }
                },
                error: (err) => {
                    this._snack.open(parseErrorMessage(err, 'Proxy verification failed'), 'Close');
                }
            });
    }

    protected readonly form = form;
}
