import { inject, Injectable } from "@angular/core";
import { MatDialog } from "@angular/material/dialog";
import { MusicDetailModal } from "@app/pages/music-tracks-view/music-detail-modal/music-detail-modal";
import { MusicTracksViewState } from "@app/pages/music-tracks-view/music-tracks-view.state";
import { MusicTracksService } from "@app/pages/music-tracks-view/music-tracks.service";
import { PlayerService } from "@app/services/player.service";
import {
  DialogDeleteTrack,
  DialogDeleteTrackData,
} from "@shared/components/dialogs/delete-track/delete-track";
import { AudioFormat, PlayableTrack } from "@shared/models/music-track.model";

@Injectable({
  providedIn: "root",
})
export class TrackActionsService {
  private readonly _dialog = inject(MatDialog);
  private readonly _api = inject(MusicTracksService);
  private readonly _tracksState = inject(MusicTracksViewState);
  private readonly _player = inject(PlayerService);

  public download(track: PlayableTrack): void {
    const format = track.format || AudioFormat.MP3;
    const link = document.createElement("a");
    link.href = `/api/stream/${track.id}`;
    link.download = `${track.artist} - ${track.title}.${format}`;
    link.click();
  }

  public delete(ids: number[]): void {
    this._api.delete(ids).subscribe({
      next: () => {
        this._tracksState.removeTracks(ids);
        const currentId = this._player.currentTrack()?.id;
        if (currentId && ids.includes(currentId)) {
          this._player.next();
        }
        this._player.removeFromQueue(ids);
      },
    });
  }

  public deleteWithDialog(data: { title: string; id: number }): void {
    const dialog = this._dialog.open<DialogDeleteTrack, DialogDeleteTrackData>(
      DialogDeleteTrack,
      {
        data: { title: data.title },
      },
    );

    dialog.afterClosed().subscribe((result) => {
      if (result) {
        this.delete([data.id]);
      }
    });
  }

  public openDetails(track: PlayableTrack): void {
    const fullTrack =
      this._tracksState.tracks().find((t) => t.id === track.id) ?? track;
    this._dialog.open(MusicDetailModal, {
      data: fullTrack,
      maxWidth: "500px",
      width: "100%",
    });
  }

  public openSource(url: string): void {
    window.open(url, "_blank");
  }
}
