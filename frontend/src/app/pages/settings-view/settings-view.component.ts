import {ChangeDetectionStrategy, Component, inject, OnInit, signal} from '@angular/core';
import {form, FormRoot, max, min, required, validate} from '@angular/forms/signals';
import {SettingsService, SettingsTestingService} from './settings.service';
import {NotificationService} from '@app/services/notification.service';
import {finalize, firstValueFrom} from 'rxjs';
import {takeUntilDestroyed} from '@angular/core/rxjs-interop';
import {parseErrorMessage} from '@shared/error-utils';
import {englishOnly, soundCloudUrl} from '@shared/validators/form.validators';
import {SettingsFormModel} from '@app/pages/settings-view/models/settings-form.model';
import {WebSocketService} from '@app/services/websocket.service';
import {namingTemplateValidator, proxyUrlValidator, soundcloudUrlTestValidator} from './settings.validators';

import {SoundcloudSectionComponent} from './sections/soundcloud-section/soundcloud-section.component';
import {DownloadsSectionComponent} from './sections/downloads-section/downloads-section.component';
import {NetworkSectionComponent} from './sections/network-section/network-section.component';
import {AdbSectionComponent} from './sections/adb-section/adb-section.component';

@Component({
    selector: 'app-settings-view',
    imports: [
        FormRoot,
        SoundcloudSectionComponent,
        DownloadsSectionComponent,
        NetworkSectionComponent,
        AdbSectionComponent
    ],
    templateUrl: './settings-view.component.html',
    styleUrl: './settings-view.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsView implements OnInit {

    private readonly _api = inject(SettingsService);
    private readonly _testing = inject(SettingsTestingService);
    private readonly _notification = inject(NotificationService);
    private readonly _ws = inject(WebSocketService);

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
            namingTemplate: '{artist} - {title}'
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

    protected readonly connectedDevices = signal<string[]>([]);
    protected readonly isRefreshing = signal(false);

    protected readonly soundcloudUrlTestError = signal<string | null>(null);
    protected readonly proxyUrlTestError = signal<string | null>(null);

    protected readonly settingsForm =
        form(this.settingsModel, (f) => {
            englishOnly(f.soundcloud.profileUrl);
            soundCloudUrl(f.soundcloud.profileUrl);
            validate(f.soundcloud.profileUrl, soundcloudUrlTestValidator(() => this.soundcloudUrlTestError()));

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

            validate(f.network.proxyUrl, proxyUrlValidator(() => this.proxyUrlTestError()));
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

    constructor() {
        this.refreshDevices();

        this._ws
            .connect<string[]>('/api/devices/ws')
            .pipe(takeUntilDestroyed())
            .subscribe({
                next: (devices) => this.updateConnectedDevices(devices),
                error: (err) => console.error('ADB devices WebSocket error:', err),
            });
    }

    private updateConnectedDevices(devices: string[]) {
        this.connectedDevices.set(devices);

        const current = [...this.settingsForm.adb.devices().value()];
        let changed = false;

        for (const id of devices) {
            if (!current.some(d => d.deviceId === id)) {
                current.push({
                    deviceId: id,
                    remoteMusicDir: '/storage/Music',
                    enabled: true,
                });
                changed = true;
            }
        }

        if (changed) {
            this.settingsForm.adb.devices().value.set(current);
            this.settingsForm.adb.devices().markAsDirty();
        }
    }

    protected testSoundcloud() {
        this.isTesting.set(true);

        this._testing.testSoundCloud(this.settingsForm.soundcloud.profileUrl().value())
            .pipe(
                finalize(() => {
                    this.isTesting.set(false);
                    this.settingsForm.soundcloud.profileUrl().reset();
                    this.settingsForm.soundcloud.profileUrl().reloadValidation();
                })
            )
            .subscribe({
                next: () => this.soundcloudUrlTestError.set(null),
                error: (err) => {
                    const errMsg = parseErrorMessage(err, 'Invalid SoundCloud configuration');
                    this.soundcloudUrlTestError.set(errMsg);
                }
            });
    }

    protected testProxy() {
        this.isNetworkTesting.set(true);
        const proxy = this.settingsForm.network.proxyUrl().value();

        this._testing.testProxy([proxy])
            .pipe(
                finalize(() => {
                    this.isNetworkTesting.set(false);
                    this.settingsForm.network.proxyUrl().reset();
                    this.settingsForm.network.proxyUrl().reloadValidation();
                })
            )
            .subscribe({
                next: (res) => {
                    if (res.valid) {
                        this.proxyUrlTestError.set(null);
                        this._notification.success('Proxy connection successful');
                    } else {
                        const err = res.error || 'Proxy is not able to reach SoundCloud API';
                        this.proxyUrlTestError.set(err);
                        this._notification.error(err);
                    }
                },
                error: (err) => {
                    const errMsg = parseErrorMessage(err, 'Proxy verification failed');
                    this.proxyUrlTestError.set(errMsg);
                    this._notification.error(errMsg);
                }
            });
    }

    protected refreshDevices() {
        this.isRefreshing.set(true);
        this._api.getAdbDevices()
            .pipe(finalize(() => this.isRefreshing.set(false)))
            .subscribe({
                next: (devices) => this.updateConnectedDevices(devices),
                error: (err) => console.error('Failed to get connected ADB devices:', err)
            });
    }

    protected removeDevice(index: number) {
        this.settingsForm.adb.devices().value.update(current => current.filter((_, i) => i !== index));
        this.settingsForm.adb.devices().markAsDirty();
    }

    public ngOnInit() {
        this._api.getSettings()
            .subscribe(res => this.settingsForm().reset({
                soundcloud: res.soundcloud,
                downloads: res.downloads,
                network: res.network,
                adb: res.adb ?? { enabled: true, autoSync: true, devices: [] },
                system: res.system
            }));
    }
}



