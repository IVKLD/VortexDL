import { Component, inject } from '@angular/core';
import { MatButton } from '@angular/material/button';
import { MatSidenav, MatSidenavContainer, MatSidenavContent } from '@angular/material/sidenav';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { LogoComponent } from '../../logo/logo';
import { ActiveDownloadsComponent } from '../../active-downloads/active-downloads';
import { MatIcon } from '@angular/material/icon';
import { Header } from "@shared/components/bricks/header/header";
import { PlayerComponent } from '@shared/components/player/player';
import { DownloadTrackingService } from '@app/services/download-tracking.service';
import { MatDialog } from '@angular/material/dialog';
import { ActiveDownloadErrorsComponent } from '../../active-downloads/components/active-download-errors/active-download-errors.component';
import { ErrorsDialogComponent, ErrorsDialogResult } from '../../active-downloads/components/active-download-errors/errors-dialog/errors-dialog.component';
import { PlayerService } from '@app/services/player.service';
import { SidebarNotifications } from '../../sidebar-notifications/sidebar-notifications';

interface SidebarNavItem {
    path: string;
    icon: string;
    label: string;
    exact?: boolean;
}

const SIDEBAR_NAV_ITEMS: SidebarNavItem[] = [
    {
        path: '/',
        icon: 'dashboard',
        label: 'Dashboard',
        exact: true,
    },
    {
        path: '/musics',
        icon: 'library_music',
        label: 'Library',
    },
    {
        path: '/search',
        icon: 'travel_explore',
        label: 'Search',
    },
    {
        path: '/settings',
        icon: 'settings',
        label: 'Settings',
    },
];

@Component({
    selector: 'app-sidebar',
    imports: [
        RouterLinkActive,
        RouterLink,
        LogoComponent,
        ActiveDownloadsComponent,
        MatIcon,
        MatButton,
        MatSidenavContainer,
        MatSidenav,
        MatSidenavContent,
        Header,
        PlayerComponent,
        ActiveDownloadErrorsComponent,
        SidebarNotifications,
    ],
    templateUrl: './sidebar.html',
    styleUrl: './sidebar.scss',
})
export class Sidebar {
    private readonly dialog = inject(MatDialog);
    protected readonly tracking = inject(DownloadTrackingService);
    protected readonly player = inject(PlayerService);
    protected readonly navItems = SIDEBAR_NAV_ITEMS;

    protected openErrorsDialog() {
        const dialogRef = this.dialog.open(ErrorsDialogComponent, {
            data: this.tracking.errors(),
            width: '600px',
            maxWidth: '90vw'
        });

        dialogRef.afterClosed().subscribe(result => {
            if (result === ErrorsDialogResult.Clear) {
                this.tracking.clearError();
            }
        });
    }
}
