import {ChangeDetectionStrategy, Component, computed, input, output} from "@angular/core";
import {Track} from "../music-tracks-view.service";
import {MatButton, MatIconButton} from "@angular/material/button";
import {LucideTrash} from "@lucide/angular";
import {MatPrefix} from "@angular/material/input";

@Component({
    selector: "app-music-card",
    imports: [
        MatButton,
        LucideTrash,
        MatIconButton,
        MatPrefix
    ],
    templateUrl: "./music-card.html",
    styleUrl: "./music-card.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MusicCard {
    public track = input.required<Track>()
    public onDeleteMusic = output()

    public displayTitle = computed(() => {
        const item = this.track();
        return item.filename.replace(`.${item.format}`, '').replace(/_/g, ' ');
    });
}
