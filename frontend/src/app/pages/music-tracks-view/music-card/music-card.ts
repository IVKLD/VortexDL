import {ChangeDetectionStrategy, Component, computed, input, output} from "@angular/core";
import {Track} from "../music-tracks-view.service";
import {MatIconButton} from "@angular/material/button";
import {NgOptimizedImage} from "@angular/common";
import {MatIcon} from "@angular/material/icon";

@Component({
    selector: "app-music-card",
    imports: [
        MatIconButton,
        NgOptimizedImage,
        MatIcon,
    ],
    templateUrl: "./music-card.html",
    styleUrl: "./music-card.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MusicCard {
    public track = input.required<Track>()
    public onDeleteMusic = output()
    public onTrackClick = output()

    public displayTitle = computed(() => {
        const item = this.track();
        return item.filename.replace(`.${item.format}`, '').replace(/_/g, ' ');
    });
}
