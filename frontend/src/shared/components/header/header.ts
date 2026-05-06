import { ChangeDetectionStrategy, Component, inject, signal, computed } from "@angular/core";
import { MatIconButton, MatButton } from "@angular/material/button";
import { MatDialog } from "@angular/material/dialog";
import { DownloadDialogComponent } from "./download-dialog/download-dialog.component";
import { MatIcon } from "@angular/material/icon";
import { MatMenu, MatMenuItem, MatMenuTrigger } from "@angular/material/menu";
import { MusicTracksViewState, MusicSortOption } from "@app/pages/music-tracks-view/music-tracks-view.state";
import { Router, NavigationEnd, ActivatedRoute } from "@angular/router";
import { MatFormField, MatPrefix } from "@angular/material/form-field";
import { MatInput } from "@angular/material/input";
import { FormsModule } from "@angular/forms";
import { HeaderFeature, HeaderConfig } from "./header.types";
import { filter, map } from "rxjs";
import { toSignal } from "@angular/core/rxjs-interop";
import { RouteData } from "@app/app.routes";

@Component({
    selector: "app-header",
    imports: [
        MatIcon,
        MatIconButton,
        MatButton,
        MatMenu,
        MatMenuItem,
        MatMenuTrigger,
        MatFormField,
        MatInput,
        MatPrefix,
        FormsModule,
    ],
    templateUrl: "./header.html",
    styleUrl: "./header.scss",
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Header {
    private readonly _dialog = inject(MatDialog);
    private readonly _router = inject(Router);
    private readonly _route = inject(ActivatedRoute);
    protected readonly musicState = inject(MusicTracksViewState);
    protected readonly Feature = HeaderFeature;
    protected readonly SortOption = MusicSortOption;

    private readonly _headerConfig$ = this._router.events.pipe(
        filter(event => event instanceof NavigationEnd),
        map(() => {
            let route = this._route.root;
            while (route.firstChild) {
                route = route.firstChild;
            }
            const data: RouteData = route.snapshot.data;
            return data.header;
        })
    );

    protected readonly config = toSignal(this._headerConfig$, {
        initialValue: this.getInitialHeaderConfig()
    });

    private getInitialHeaderConfig(): HeaderConfig | undefined {
        const data: RouteData = this._route.root.snapshot.firstChild?.data || {};
        return data.header;
    }

    protected hasFeature(feature: HeaderFeature): boolean {
        return this.config()?.features.includes(feature) ?? false;
    }

    protected openDownloadDialog() {
        this._dialog.open(DownloadDialogComponent, {
            maxWidth: '450px',
            width: '100%',
            autoFocus: 'input'
        });
    }

    protected setSort(option: MusicSortOption) {
        this.musicState.setSortOption(option);
    }

    protected onSearch(query: string) {
        this.musicState.setSearchQuery(query);
    }
}
