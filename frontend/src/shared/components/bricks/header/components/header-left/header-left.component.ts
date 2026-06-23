import {Component, inject, input} from '@angular/core';
import { MusicTracksViewState } from '@app/pages/music-tracks-view/music-tracks-view.state';
import { HeaderConfig, HeaderFeature } from '../../header.types';

@Component({
    selector: 'app-header-left',
    imports: [],
    templateUrl: './header-left.component.html',
    styleUrl: './header-left.component.scss',
    })
export class HeaderLeft {
    public readonly config = input<HeaderConfig>();
    protected readonly musicState = inject(MusicTracksViewState);
    protected readonly Feature = HeaderFeature;

    protected hasFeature(feature: HeaderFeature): boolean {
        return this.config()?.features.includes(feature) ?? false;
    }
}
