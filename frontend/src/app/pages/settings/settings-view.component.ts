import { ChangeDetectionStrategy, Component, inject, OnInit, signal } from '@angular/core';
import { MatIcon } from '@angular/material/icon';
import { form, FormRoot } from '@angular/forms/signals';
import { SettingsService } from './settings.service';
import { MatButton } from "@angular/material/button";
import { SettingsFormModel } from "@app/pages/settings/models/settings-form.model";
import { finalize } from "rxjs";
import { englishOnly, required, soundCloudUrl } from "@shared/validators/form.validators";
import { UserSettingsRdo } from "@app/pages/settings/models/user-settings.rdo";
import { UserSettingsDto } from "@app/pages/settings/models/user-settings.dto";
import { SoundcloudSectionComponent } from "./components/soundcloud-section/soundcloud-section.component";
import { DownloadsSectionComponent } from "./components/downloads-section/downloads-section.component";
import { SystemSectionComponent } from "./components/system-section/system-section.component";

@Component({
    selector: 'app-settings-view',
    imports: [
        MatIcon, FormRoot, MatButton,
        SoundcloudSectionComponent, DownloadsSectionComponent, SystemSectionComponent
    ],
    templateUrl: './settings-view.component.html',
    styleUrl: './settings-view.component.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SettingsView implements OnInit {
    private readonly _api = inject(SettingsService);

    protected readonly isLoading = signal(true);
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
        }
    });

    protected readonly settingsForm =
        form(this.settingsModel, (f) => {
            required(f.soundcloud.profileUrl, 'Profile URL is required');
            englishOnly(f.soundcloud.profileUrl);
            soundCloudUrl(f.soundcloud.profileUrl);

            // required(f.downloads.outputPath, 'Output path is required');
            // required(f.downloads.namingTemplate, 'Naming template is required');
        },
            {
                submission: {
                    action: async (data) => {
                        const val = data().value();
                        const payload: UserSettingsDto = {
                            soundcloud: val.soundcloud,
                            downloads: val.downloads,
                            limitPerPage: val.system.limitPerPage,
                            maxRetries: val.system.maxRetries
                        };

                        this._api.updateSettings(payload).subscribe()
                    }
                },
            });

    public ngOnInit() {
        this._api.getSettings()
            .pipe(finalize(() =>
                this.isLoading.set(false)
            ))
            .subscribe({
                next: (res: UserSettingsRdo) => {
                    this.settingsForm().reset({
                        soundcloud: res.soundcloud,
                        downloads: res.downloads,
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

}
