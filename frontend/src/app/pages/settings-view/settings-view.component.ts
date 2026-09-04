import { Component, inject, OnInit } from '@angular/core';
import { FormRoot } from '@angular/forms/signals';
import { SettingsState } from './settings.state';
import { SoundcloudSectionComponent } from './sections/soundcloud-section/soundcloud-section.component';
import { DownloadsSectionComponent } from './sections/downloads-section/downloads-section.component';
import { NetworkSectionComponent } from './sections/network-section/network-section.component';
import { AdbSectionComponent } from './sections/adb-section/adb-section.component';
import { SyncSectionComponent } from './sections/sync-section/sync-section.component';

@Component({
    selector: 'app-settings-view',
    imports: [
        FormRoot,
        SoundcloudSectionComponent,
        DownloadsSectionComponent,
        NetworkSectionComponent,
        AdbSectionComponent,
        SyncSectionComponent,
    ],
    templateUrl: './settings-view.component.html',
    styleUrl: './settings-view.component.scss',
})
export class SettingsView implements OnInit {
    protected readonly state = inject(SettingsState);

    public ngOnInit(): void {
        this.state.loadSettings();
    }
}
