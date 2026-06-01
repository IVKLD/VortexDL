import {ChangeDetectionStrategy, Component} from '@angular/core';
import {MatButton} from '@angular/material/button';
import {MatSidenav, MatSidenavContainer, MatSidenavContent} from '@angular/material/sidenav';
import {RouterLink, RouterLinkActive} from '@angular/router';
import {LogoComponent} from '../logo/logo';
import {ActiveDownloadsComponent} from '../active-downloads/active-downloads';
import {MatIcon} from '@angular/material/icon';
import {Header} from "@shared/components/header/header";
import {PlayerComponent} from '@shared/components/player/player';

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
    ],
    templateUrl: './sidebar.html',
    styleUrl: './sidebar.scss',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class Sidebar {
    protected readonly navItems = SIDEBAR_NAV_ITEMS;
}
