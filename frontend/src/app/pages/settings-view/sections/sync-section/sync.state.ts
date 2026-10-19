import { computed, inject, Injectable, signal } from '@angular/core';
import { form } from '@angular/forms/signals';
import { SettingsService } from '../../settings.service';
import { SettingsState } from '../../settings.state';
import { SyncAction, SyncProviderPayload, SyncProviderType } from '../../models/sync.model';
import { finalize } from 'rxjs';
import { localPathSchema, restoreUrlSchema } from './sync.schema';

@Injectable()
export class SyncState {
    private readonly _api = inject(SettingsService);
    private readonly _settingsState = inject(SettingsState);

    public readonly isExporting = signal(false);
    public readonly isImporting = signal(false);
    public readonly selectedProvider = signal<SyncProviderType>(SyncProviderType.WebDav);
    public readonly localPath = signal<string>('');
    public readonly restoreUrl = signal<string>('');

    public readonly localPathForm = form(this.localPath, localPathSchema);
    public readonly restoreUrlForm = form(this.restoreUrl, restoreUrlSchema);

    public readonly canExport = computed(() => {
        if (this.isExporting() || this.isImporting()) return false;
        if (this.selectedProvider() === SyncProviderType.Url) return false;
        return this.isCurrentProviderValid();
    });

    public readonly canImport = computed(() => {
        if (this.isExporting() || this.isImporting()) return false;
        return this.isCurrentProviderValid();
    });

    private isCurrentProviderValid(): boolean {
        switch (this.selectedProvider()) {
            case SyncProviderType.WebDav:
                return this._settingsState.settingsForm.webdav().valid();
            case SyncProviderType.Local:
                return this.localPathForm().valid();
            case SyncProviderType.Url:
                return this.restoreUrlForm().valid();
        }
    }

    private getProviderPayload(): SyncProviderPayload | null {
        switch (this.selectedProvider()) {
            case SyncProviderType.WebDav: {
                const f = this._settingsState.settingsForm.webdav;
                return {
                    webDav: {
                        baseUrl: f.baseUrl().value(),
                        remoteDir: f.remoteDir().value(),
                        username: f.username().value(),
                        password: f.password().value(),
                    },
                };
            }
            case SyncProviderType.Local:
                return {
                    local: {
                        path: this.localPath().trim(),
                    },
                };
            case SyncProviderType.Url:
                return {
                    url: {
                        url: this.restoreUrl().trim(),
                    },
                };
        }
    }

    public exportBackup(): void {
        if (this.selectedProvider() === SyncProviderType.Url) return;
        const payload = this.getProviderPayload();
        if (!payload) return;

        this.isExporting.set(true);
        this._api
            .syncBackup(SyncAction.Export, payload)
            .pipe(finalize(() => this.isExporting.set(false)))
            .subscribe();
    }

    public importBackup(): void {
        const payload = this.getProviderPayload();
        if (!payload) return;

        this.isImporting.set(true);
        this._api
            .syncBackup(SyncAction.Import, payload)
            .pipe(finalize(() => this.isImporting.set(false)))
            .subscribe({
                next: () => this._settingsState.loadSettings(),
            });
    }
}
