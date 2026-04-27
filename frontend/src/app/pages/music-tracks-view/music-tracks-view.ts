import {ChangeDetectionStrategy, Component, computed, inject, OnInit, ViewChild, viewChild} from '@angular/core';
import {MatPaginator} from '@angular/material/paginator';
import {MatSort} from '@angular/material/sort';
import {MusicTracksViewService, Track} from "./music-tracks-view.service";
import {MusicCard} from "./music-card/music-card";
import {MusicTracksViewState} from "./music-tracks-view.state";
import {
    AutoSizeVirtualScrollStrategy, FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent
} from "@rx-angular/template/virtual-scrolling";
import {DownloadTrackingService} from "../../services/download-tracking.service";

@Component({
    selector: 'app-music-tracks-view',
    imports: [
        MusicCard,
        RxVirtualScrollViewportComponent,
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
    ],
    providers: [MusicTracksViewService],
    templateUrl: './music-tracks-view.html',
    styleUrl: './music-tracks-view.scss',
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class MusicTracksView implements OnInit {
    private readonly _api = inject(MusicTracksViewService);
    private readonly _state = inject(MusicTracksViewState);
    private readonly _downloadTracking = inject(DownloadTrackingService);

    protected readonly tracks = computed(() => {
        const tracks = this._state.getTracks();
        return Array.isArray(tracks) ? [...tracks] : [];
    });

    private readonly viewport = viewChild(RxVirtualScrollViewportComponent)

    protected deleteMusic(track: Track) {
        this._api.delete(track.id).subscribe({
            next: () => {
                this._state.removeTrack(track);
            }
        });
    }

    ngOnInit() {
        this._api.getAll().subscribe({
            next: tracks => {
                this._state.setTracks = tracks;
            }
        });
    }
}