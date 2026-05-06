import { ChangeDetectionStrategy, Component, inject, OnInit } from '@angular/core';
import { MusicTracksViewService, Track } from "./music-tracks-view.service";
import { MusicCard } from "./music-card/music-card";
import { MusicTracksViewState } from "./music-tracks-view.state";
import {
    FixedSizeVirtualScrollStrategy,
    RxVirtualFor,
    RxVirtualScrollViewportComponent
} from "@rx-angular/template/virtual-scrolling";
import { MatDialog } from "@angular/material/dialog";
import { MusicDetailModal } from "./music-detail-modal/music-detail-modal";
import { MatIcon } from "@angular/material/icon";

@Component({
    selector: 'app-music-tracks-view',
    imports: [
        MusicCard,
        RxVirtualScrollViewportComponent,
        RxVirtualFor,
        FixedSizeVirtualScrollStrategy,
        MatIcon,
    ],
    providers: [MusicTracksViewService],
    templateUrl: './music-tracks-view.html',
    styleUrl: './music-tracks-view.scss',
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class MusicTracksView implements OnInit {
    private readonly _api = inject(MusicTracksViewService);
    private readonly _state = inject(MusicTracksViewState);
    private readonly _dialog = inject(MatDialog);

    protected readonly tracks = this._state.sortedTracks;

    protected deleteMusic(track: Track) {
        this._api.delete(track.id).subscribe({
            next: () => {
                this._state.removeTrack(track);
            }
        });
    }

    protected openDetail(track: Track) {
        this._dialog.open(MusicDetailModal, {
            data: track,
            maxWidth: '500px',
            width: '100%'
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