import { Component, inject, OnInit, signal, ChangeDetectionStrategy } from '@angular/core';
import { form, FormRoot, max, min, required, validate, FieldTree, ValidationError, FormField } from '@angular/forms/signals';
import { SettingsService, SettingsTestingService } from './settings.service';
import { NotificationService } from '@app/services/notification.service';
import { debounceTime, filter, finalize, switchMap, map, catchError, EMPTY, firstValueFrom } from 'rxjs';
import { toObservable, takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { parseErrorMessage } from '@shared/error-utils';
import { englishOnly, soundCloudUrl } from '@shared/validators/form.validators';
import { SettingsFormModel } from '@app/pages/settings-view/models/settings-form.model';
import { WebSocketService } from '@app/services/websocket.service';

import { MatIcon } from '@angular/material/icon';
import { MatButton, MatIconButton } from '@angular/material/button';
import { MatInput } from '@angular/material/input';
import { MatFormField, MatHint, MatLabel, MatError } from '@angular/material/form-field';
import { MatDivider } from '@angular/material/divider';
import { SettingsSwitchCardComponent } from './components/settings-switch-card/settings-switch-card.component';
import { FallbackProxiesComponent } from './components/network-section/components/fallback-proxies/fallback-proxies.component';
import { AdbDeviceListComponent } from './components/adb-section/components/adb-device-list/adb-device-list.component';

@Component({
    selector: 'app-settings-view',
    imports: [
        FormRoot,
        FormField,
        MatIcon,
        MatButton,
        MatIconButton,
        MatInput,
        MatFormField,
        MatHint,
        MatLabel,
        MatError,
        MatDivider,
        SettingsSwitchCardComponent,
        FallbackProxiesComponent,
        AdbDeviceListComponent
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
            validate(f.soundcloud.profileUrl, (ctx) => {
                const testErr = this.soundcloudUrlTestError();
                return (testErr && !ctx.state.dirty()) ? { kind: 'testFailed', message: testErr } : null;
            });

            min(f.soundcloud.syncInterval, 1, { message: 'Interval must be at least 1 minute' });
            max(f.soundcloud.syncInterval, 1440, { message: 'Interval cannot exceed 24 hours' });

            required(f.downloads.outputPath, { message: 'Output path is required' });
            min(f.downloads.maxConcurrent, 1, { message: 'Must have at least 1 concurrent download' });
            max(f.downloads.maxConcurrent, 100, { message: 'Maximum 10 concurrent downloads allowed' });
            required(f.downloads.namingTemplate, { message: 'Naming template is required' });

            min(f.system.limitPerPage, 1, { message: 'Limit must be at least 1' });
            max(f.system.limitPerPage, 500, { message: 'Limit cannot exceed 500' });
            min(f.system.maxRetries, 0, { message: 'Max retries cannot be negative' });
            max(f.system.maxRetries, 20, { message: 'Max retries cannot exceed 20' });

            validate(f.network.proxyUrl, (ctx) => {
                const val = ctx.value();
                const useProxy = ctx.valueOf(f.network.useProxy);
                if (useProxy && !val) {
                    return { kind: 'required', message: 'Proxy URL is required when proxy is enabled' };
                }
                if (val) {
                    const match = /^(socks5|http|https):\/\/[a-zA-Z0-9\-_.:@]+$/i.test(val);
                    if (!match) {
                        return { kind: 'pattern', message: 'Invalid proxy URL (e.g. socks5://127.0.0.1:1080)' };
                    }
                }
                const testErr = this.proxyUrlTestError();
                return (testErr && !ctx.state.dirty()) ? { kind: 'testFailed', message: testErr } : null;
            });
        }, {
            submission: {
                action: async () => {
                    if (this.settingsForm().valid()) {
                        const payload = this.settingsForm().value();
                        await firstValueFrom(this._api.updateSettings(payload));
                        this.settingsForm().reset(payload);
                    }
                    return [];
                }
            }
        });

    constructor() {
        toObservable(this.settingsForm().value)
            .pipe(
                debounceTime(1000),
                filter(() => this.settingsForm().valid() && this.settingsForm().dirty()),
                switchMap((payload) =>
                    this._api.updateSettings(payload).pipe(
                        map(() => payload),
                        catchError(() => EMPTY)
                    )
                ),
                takeUntilDestroyed()
            )
            .subscribe(payload => this.settingsForm().reset(payload));

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
            .pipe(finalize(() => this.isTesting.set(false)))
            .subscribe({
                next: () => {
                    this.soundcloudUrlTestError.set(null);
                    this.settingsForm.soundcloud.profileUrl().reset();
                    this.settingsForm.soundcloud.profileUrl().reloadValidation();
                },
                error: (err) => {
                    const errMsg = parseErrorMessage(err, 'Invalid SoundCloud configuration');
                    this.soundcloudUrlTestError.set(errMsg);
                    this.settingsForm.soundcloud.profileUrl().reset();
                    this.settingsForm.soundcloud.profileUrl().reloadValidation();
                }
            });
    }

    protected testProxy() {
        this.isNetworkTesting.set(true);
        const proxy = this.settingsForm.network.proxyUrl().value();

        this._testing.testProxy([proxy])
            .pipe(finalize(() => this.isNetworkTesting.set(false)))
            .subscribe({
                next: (res) => {
                    if (res.valid) {
                        this.proxyUrlTestError.set(null);
                        this.settingsForm.network.proxyUrl().reset();
                        this.settingsForm.network.proxyUrl().reloadValidation();
                        this._notification.success('Proxy connection successful');
                    } else {
                        const err = res.error || 'Proxy is not able to reach SoundCloud API';
                        this.proxyUrlTestError.set(err);
                        this.settingsForm.network.proxyUrl().reset();
                        this.settingsForm.network.proxyUrl().reloadValidation();
                        this._notification.error(err);
                    }
                },
                error: (err) => {
                    const errMsg = parseErrorMessage(err, 'Proxy verification failed');
                    this.proxyUrlTestError.set(errMsg);
                    this.settingsForm.network.proxyUrl().reset();
                    this.settingsForm.network.proxyUrl().reloadValidation();
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


    protected hasFormatError(field: FieldTree<string>): boolean {
        return field().errors().some((e: ValidationError) => e.kind !== 'testFailed');
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



