import {Injectable, signal} from "@angular/core";
import {Track} from "./music-tracks-view.service";

@Injectable({
    providedIn: 'root',
})
export class MusicTracksViewState {
    private readonly _tracks = signal<Track[]>([]);

    public get getTracks() {
        return this._tracks;
    }

    public set setTracks(value: Track[]) {
        this._tracks.set(value)
    }

    public addTrack(music: Track) {
        this._tracks.update(data => [...data, music])
    }

    public removeTrack(music: Track) {
        this._tracks.update(data => data.filter(item => item.filename !== music.filename))
    }
}